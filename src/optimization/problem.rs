use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashSet;
use std::ops::Deref;
use std::rc::{Rc, Weak};

use rand::{SeedableRng, thread_rng};
use rand::rngs::StdRng;

use crate::{DETERMINISTIC_MODE, Instance, Orientation, PartType, SheetType};
use crate::core::cost::Cost;
use crate::core::entities::layout::Layout;
use crate::core::entities::node::Node;
use crate::core::insertion::insertion_blueprint::InsertionBlueprint;
use crate::optimization::rr::cache_updates::CacheUpdates;
use crate::optimization::solutions::sendable_solution::SendableSolution;
use crate::optimization::solutions::problem_solution::ProblemSolution;
use crate::optimization::solutions::solution::Solution;
use crate::util::assertions;
use crate::util::macros::{rb, rbm};

pub struct Problem<'a> {
    instance: &'a Instance,
    parttype_qtys: Vec<usize>,
    sheettype_qtys: Vec<usize>,
    layouts: Vec<Rc<RefCell<Layout<'a>>>>,
    empty_layouts: Vec<Rc<RefCell<Layout<'a>>>>,
    unchanged_layouts: HashSet<usize>,
    unchanged_layouts_solution_id: Option<usize>,
    random: StdRng,
    counter_layout_id: usize,
    counter_solution_id: usize,
}

impl<'a> Problem<'a> {
    pub fn new(instance: &'a Instance) -> Self {
        let parttype_qtys = instance.parts().iter().map(|(_, qty)| *qty).collect::<Vec<_>>();
        let sheettype_qtys = instance.sheets().iter().map(|(_, qty)| *qty).collect::<Vec<_>>();
        let layouts = Vec::new();
        let empty_layouts = Vec::new();
        let unchanged_layouts = HashSet::new();
        let unchanged_layouts_solution_id = None;
        let random = match DETERMINISTIC_MODE {
            true => StdRng::seed_from_u64(0),
            false => StdRng::from_rng(thread_rng()).unwrap()
        };
        let counter_layout_id = 0;
        let counter_solution_id = 0;

        let mut problem = Problem {
            instance,
            parttype_qtys,
            sheettype_qtys,
            layouts,
            empty_layouts,
            unchanged_layouts,
            unchanged_layouts_solution_id,
            random,
            counter_layout_id,
            counter_solution_id,
        };

        //Initiate the empty layouts
        for (sheettype, _) in instance.sheets() {
            if sheettype.fixed_first_cut_orientation().is_none() || sheettype.fixed_first_cut_orientation().unwrap() == Orientation::Horizontal {
                let id = problem.next_layout_id();
                problem.empty_layouts.push(Rc::new(RefCell::new(
                    Layout::new(sheettype, Orientation::Horizontal, id))));
            }
            if sheettype.fixed_first_cut_orientation().is_none() || sheettype.fixed_first_cut_orientation().unwrap() == Orientation::Vertical {
                let id = problem.next_layout_id();
                problem.empty_layouts.push(Rc::new(RefCell::new(
                    Layout::new(sheettype, Orientation::Vertical, id))));
            }
        }

        problem
    }

    pub fn implement_insertion_blueprint(&mut self, blueprint: &InsertionBlueprint<'a>) -> (CacheUpdates<'a, Weak<RefCell<Node<'a>>>>, bool) {
        let blueprint_layout = blueprint.layout().as_ref().unwrap().upgrade().unwrap();

        let blueprint_creates_new_layout = self.empty_layouts.iter().any(|e| Rc::ptr_eq(e, &blueprint_layout));

        let cache_updates = match blueprint_creates_new_layout {
            false => {
                let mut cache_updates = CacheUpdates::new(Rc::downgrade(&blueprint_layout));

                blueprint_layout.borrow_mut().implement_insertion_blueprint(blueprint, &mut cache_updates, self.instance);
                self.layout_has_changed(rb!(blueprint_layout).id());


                cache_updates
            }
            true => {
                let copy = rb!(blueprint_layout).create_deep_copy(self.next_layout_id());
                //Create a copy of the insertion blueprint and map it to the copy of the layout
                let mut insertion_bp_copy = blueprint.clone();
                //Modify so the blueprint so the original node maps to the respective node of the copied layout
                let modified_original_node = rb!(copy.top_node()).children().first().unwrap().clone();
                insertion_bp_copy.set_original_node(Rc::downgrade(&modified_original_node));
                //wrap the copied layout
                let copy = Rc::new(RefCell::new(copy));
                insertion_bp_copy.set_layout(Rc::downgrade(&copy));
                self.register_layout(copy.clone());

                //Search the layout again in the problem, to please the borrow checker
                let copy = self.layouts.iter().find(|l| Rc::ptr_eq(l, &copy)).unwrap();

                debug_assert!(assertions::all_weak_references_alive(rb!(copy).sorted_empty_nodes()));
                let mut cache_updates = CacheUpdates::new(Rc::downgrade(&copy));
                rbm!(copy).implement_insertion_blueprint(&insertion_bp_copy, &mut cache_updates, self.instance);
                debug_assert!(assertions::all_weak_references_alive(rb!(copy).sorted_empty_nodes()));

                cache_updates
            }
        };

        self.register_part(blueprint.parttype(), 1);

        (cache_updates, blueprint_creates_new_layout)
    }

    pub fn remove_node(&mut self, node: &Rc<RefCell<Node<'a>>>, layout: &Rc<RefCell<Layout<'a>>>) -> u64 {
        debug_assert!(assertions::node_belongs_to_layout(node, rb!(layout).deref()));
        debug_assert!(assertions::layout_belongs_to_problem(layout, self));

        self.layout_has_changed(rb!(layout).id());

        let is_top_node = Rc::ptr_eq(node, rb!(layout).top_node());

        match is_top_node {
            true => {
                //The node to remove is the root node of the layout, so the entire layout is removed
                self.unregister_layout(layout);
                rb!(layout).sheettype().value()
            }
            false => {
                {
                    let mut layout_ref = rbm!(layout);
                    layout_ref.remove_node(node);
                    let mut parts_to_release = Vec::new();
                    rb!(node).get_included_parts(&mut parts_to_release);
                    parts_to_release.iter().for_each(|p| { self.unregister_part(p, 1) });
                }

                if rb!(layout).is_empty() {
                    self.unregister_layout(layout);
                    rb!(layout).sheettype().value()
                } else {
                    0
                }
            }
        }
    }

    pub fn cost(&self) -> Cost {
        let mut cost = self.layouts.iter().fold(Cost::new(0, 0.0, 0,0), |acc, l| acc + rb!(l).cost());

        cost.part_area_excluded = self.parttype_qtys.iter().enumerate()
            .fold(0, |acc, (id, qty)| acc + self.instance().get_parttype(id).area() * (*qty as u64));

        cost.part_area_included = self.instance.total_part_area() - cost.part_area_excluded;

        cost
    }

    pub fn create_solution(&mut self, old_solution: &Option<ProblemSolution<'a>>, cached_cost: Option<Cost>) -> ProblemSolution<'a> {
        debug_assert!(cached_cost.is_none() || cached_cost.as_ref().unwrap() == &self.cost());
        let id = self.next_solution_id();
        let solution = match old_solution {
            Some(old_solution) => {
                debug_assert!(old_solution.id() == self.unchanged_layouts_solution_id.unwrap());

                ProblemSolution::new(self, cached_cost.unwrap_or(self.cost()), id, old_solution)
            }
            None => {
                ProblemSolution::new_force_copy_all(self, cached_cost.unwrap_or(self.cost()), id)
            }
        };

        debug_assert!(assertions::problem_matches_solution(self, &solution), "{:#?},{:#?}", id, self.unchanged_layouts_solution_id);

        self.reset_unchanged_layouts(solution.id());


        solution
    }

    pub fn restore_from_problem_solution(&mut self, solution: &ProblemSolution<'a>) {
        match self.unchanged_layouts_solution_id == Some(solution.id()) {
            true => {
                //A partial restore is possible.
                let mut layouts_in_problem = HashSet::new();
                for layout in self.layouts.clone().iter() {
                    //For all layouts in the problem, check which ones occur in the solution
                    let layout_id = rb!(layout).id();
                    match solution.layouts().contains_key(&layout_id) {
                        true => {
                            //layout is present in the solution
                            match self.unchanged_layouts.contains(&layout_id) {
                                true => {
                                    //the layout is unchanged with respect to the solution, nothing needs to change
                                }
                                false => {
                                    //layout was changed
                                    self.layouts.retain(|l| !Rc::ptr_eq(l, layout));
                                    let copy = solution.layouts().get(&layout_id).unwrap().create_deep_copy(layout_id);
                                    let copy = Rc::new(RefCell::new(copy));
                                    self.layouts.push(copy.clone());
                                }
                            }
                            layouts_in_problem.insert(layout_id);
                        }
                        false => {
                            //layout is not present in the solution
                            self.layouts.retain(|l| !Rc::ptr_eq(l, layout));
                        }
                    }
                }
                //Some layouts are present in the solution, but not in the problem
                for id in solution.layouts().keys() {
                    if !layouts_in_problem.contains(id) {
                        let copy = solution.layouts().get(id).unwrap().create_deep_copy(*id);
                        let copy = Rc::new(RefCell::new(copy));
                        self.layouts.push(copy.clone());
                    }
                }
            }
            false => {
                //The id of the solution does not match unchanged_layouts_solution_id, a partial restore is not possible
                self.layouts.clear();
                for (id, layout) in solution.layouts().iter() {
                    let copy = layout.create_deep_copy(*id);
                    let copy = Rc::new(RefCell::new(copy));
                    self.layouts.push(copy.clone());
                }
            }
        }

        self.parttype_qtys = solution.parttype_qtys().clone();
        self.sheettype_qtys = solution.sheettype_qtys().clone();

        self.reset_unchanged_layouts(solution.id());

        debug_assert!(assertions::problem_matches_solution(self, solution));
    }

    pub fn restore_from_instance_solution(&mut self, solution: &SendableSolution) {
        todo!()
    }

    pub fn usage(&self) -> f64 {
        let total_included_part_area = self.instance().parts().iter().map(
            |(parttype, qty)| {parttype.area() * (*qty - self.parttype_qtys.get(parttype.id()).unwrap()) as u64}
        ).sum::<u64>();
        let total_used_sheet_area = self.layouts().iter().map(
            |layout| {rb!(layout).sheettype().area()}
        ).sum::<u64>();

        total_included_part_area as f64 / total_used_sheet_area as f64
    }

    pub fn instance(&self) -> &'a Instance {
        self.instance
    }
    pub fn parttype_qtys(&self) -> &Vec<usize> {
        &self.parttype_qtys
    }
    pub fn sheettype_qtys(&self) -> &Vec<usize> {
        &self.sheettype_qtys
    }

    pub fn random(&mut self) -> &mut StdRng {
        &mut self.random
    }

    pub fn layouts(&self) -> &Vec<Rc<RefCell<Layout<'a>>>> {
        &self.layouts
    }

    pub fn register_layout(&mut self, layout: Rc<RefCell<Layout<'a>>>) {
        self.register_sheet(rb!(layout).sheettype(), 1);
        rb!(layout).get_included_parts().iter().for_each(
            |p| { self.register_part(p, 1) });

        self.layouts.push(layout);
    }

    pub fn unregister_layout(&mut self, layout: &Rc<RefCell<Layout<'a>>>) {
        debug_assert!(assertions::layout_belongs_to_problem(layout, self));

        self.unregister_sheet(rb!(layout).sheettype(), 1);
        rb!(layout).get_included_parts().iter().for_each(
            |p| { self.unregister_part(p, 1) });

        self.layouts.retain(|l| !Rc::ptr_eq(l, layout));
    }

    fn layout_has_changed(&mut self, l_id: usize) {
        self.unchanged_layouts.remove(&l_id);
    }

    fn reset_unchanged_layouts(&mut self, unchanged_layouts_solution_id: usize) {
        self.unchanged_layouts = self.layouts.iter().map(
            |l| rb!(l).id()).collect();
        self.unchanged_layouts_solution_id = Some(unchanged_layouts_solution_id);
    }

    fn register_part(&mut self, parttype: &'a PartType, qty: usize) {
        self.parttype_qtys[parttype.id()] -= qty;
    }

    fn unregister_part(&mut self, parttype: &'a PartType, qty: usize) {
        let id = parttype.id();
        debug_assert!(self.parttype_qtys[id] + qty <= self.instance.get_parttype_qty(id).unwrap());

        self.parttype_qtys[id] += qty;
    }

    fn register_sheet(&mut self, sheettype: &'a SheetType, qty: usize) {
        self.sheettype_qtys[sheettype.id()] -= qty;
    }

    fn unregister_sheet(&mut self, sheettype: &'a SheetType, qty: usize) {
        let id = sheettype.id();
        debug_assert!(self.sheettype_qtys[id] + qty <= self.instance.get_sheettype_qty(id).unwrap());

        self.sheettype_qtys[id] += qty;
    }

    fn next_layout_id(&mut self) -> usize {
        self.counter_layout_id += 1;
        self.counter_layout_id
    }

    fn next_solution_id(&mut self) -> usize {
        self.counter_solution_id += 1;
        self.counter_solution_id
    }


    pub fn empty_layouts(&self) -> &Vec<Rc<RefCell<Layout<'a>>>> {
        &self.empty_layouts
    }

    pub fn counter_layout_id(&self) -> usize {
        self.counter_layout_id
    }


    pub fn unchanged_layouts(&self) -> &HashSet<usize> {
        &self.unchanged_layouts
    }

    pub fn counter_solution_id(&self) -> usize {
        self.counter_solution_id
    }
}

impl<'a> PartialEq for Problem<'a> {
    fn eq(&self, other: &Problem<'a>) -> bool {
        std::ptr::eq(self, other)
    }
}