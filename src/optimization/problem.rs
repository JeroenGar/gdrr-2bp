use std::cell::RefCell;
use std::collections::HashSet;
use std::ops::Deref;
use std::rc::Rc;

use generational_arena::{Arena, Index};
use rand::{SeedableRng, thread_rng};
use rand::rngs::StdRng;

use crate::{DETERMINISTIC_MODE, Instance, Orientation, PartType, SheetType};
use crate::core::cost::Cost;
use crate::core::entities::layout::Layout;
use crate::core::insertion::insertion_blueprint::InsertionBlueprint;
use crate::core::layout_index::LayoutIndex;
use crate::optimization::rr::cache_updates::CacheUpdates;
use crate::optimization::solutions::problem_solution::ProblemSolution;
use crate::optimization::solutions::sendable_solution::SendableSolution;
use crate::optimization::solutions::solution::Solution;
use crate::util::assertions;
use crate::util::macros::{rb, rbm};

/// Problem is the main representation of the optimization problem.
/// A Problem is based on an Instance and contains a collection of Layouts.
/// Its main purpose is to be easily modifiable
/// It can create a snapshot of itself in the form of a ProblemSolution and use these to restore itself to a prior state.
pub struct Problem<'a> {
    instance: &'a Instance,
    parttype_qtys: Vec<usize>,
    sheettype_qtys: Vec<usize>,
    layouts: Arena<Layout<'a>>,
    empty_layouts: Vec<Layout<'a>>,
    random: StdRng,
    unchanged_layouts: HashSet<Index>,
    solution_id_unchanged_layouts: Option<usize>,
    solution_id_counter: usize,
}

impl<'a> Problem<'a> {
    pub fn new(instance: &'a Instance) -> Self {
        let parttype_qtys = instance.parts().iter().map(|(_, qty)| *qty).collect::<Vec<_>>();
        let sheettype_qtys = instance.sheets().iter().map(|(_, qty)| *qty).collect::<Vec<_>>();
        let layouts = Arena::new();
        let empty_layouts = Vec::new();
        let unchanged_layouts = HashSet::new();
        let solution_id_unchanged_layouts = None;
        let random = match DETERMINISTIC_MODE {
            true => StdRng::seed_from_u64(0),
            false => StdRng::from_rng(thread_rng()).unwrap()
        };
        let solution_id_counter = 0;

        let mut problem = Problem {
            instance,
            parttype_qtys,
            sheettype_qtys,
            layouts,
            empty_layouts,
            unchanged_layouts,
            solution_id_unchanged_layouts,
            random,
            solution_id_counter
        };

        //Initiate the empty layouts
        for (sheettype, _) in instance.sheets() {
            match sheettype.fixed_first_cut_orientation() {
                Some(orientation) => {
                    problem.empty_layouts.push(
                        Layout::new(sheettype, orientation)
                    );
                }
                None => {
                    problem.empty_layouts.extend(
                        [
                            Layout::new(sheettype, Orientation::Horizontal),
                            Layout::new(sheettype, Orientation::Vertical)
                        ]
                    );
                }
            }
        }

        problem
    }

    /// Modifies the problem by inserting an part according to the InsertionBlueprint.
    /// It returns which updates should be made to the InsertionOptionCache and whether or not a new layout was created.
    pub fn implement_insertion_blueprint(&mut self, blueprint: &InsertionBlueprint<'a>) -> CacheUpdates<'a, Index> {
        self.register_part(blueprint.parttype(), 1);

        match blueprint.layout() {
            LayoutIndex::Existing(index) => {
                let blueprint_layout = &mut self.layouts[*index];
                let mut cache_updates = CacheUpdates::new(blueprint.layout());

                blueprint_layout.implement_insertion_blueprint(blueprint, &mut cache_updates, self.instance);
                self.layout_has_changed(blueprint_layout.id());

                cache_updates
            }
            LayoutIndex::Empty(index) => {
                let empty_layout = &self.empty_layouts[*index];

                //Create a copy of the empty layout and register it
                let copy = self.register_layout(empty_layout.clone_with_id(self.next_layout_id()));

                //Implement the blueprint
                let mut cache_updates = CacheUpdates::new(LayoutIndex::Existing(copy));
                self.layouts[copy].implement_insertion_blueprint(blueprint, &mut cache_updates, self.instance);

                cache_updates
            }
        }
    }

    pub fn remove_node(&mut self, node_index: Index, layout_index: LayoutIndex) -> Option<Layout<'a>> {
        match layout_index {
            LayoutIndex::Empty(_) => panic!("Cannot remove a node from an empty layout"),
            LayoutIndex::Existing(li) => {
                let layout = &mut self.layouts[li];
                match node_index == layout.top_node() {
                    true => {
                        //TODO: test if this scenario is possible
                        //Remove the entire layout
                        Some(self.unregister_layout(layout_index))
                    }
                    false => {
                        let removed_parts = layout.remove_node(node_index);
                        removed_parts.iter().for_each(|p| self.unregister_part(p, 1));

                        if layout.is_empty() {
                            Some(self.unregister_layout(layout_index))
                        }
                        else {
                            None
                        }
                    }
                }
            }
        }
    }

    pub fn cost(&mut self) -> Cost {
        let mut cost = self.layouts.iter_mut()
            .fold(Cost::empty(), |acc, (_,l)| acc + l.cost(false));

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
                debug_assert!(old_solution.id() == self.solution_id_unchanged_layouts.unwrap());

                ProblemSolution::new(self, cached_cost.unwrap_or(self.cost()), id, old_solution)
            }
            None => {
                ProblemSolution::new_force_copy_all(self, cached_cost.unwrap_or(self.cost()), id)
            }
        };

        debug_assert!(assertions::problem_matches_solution(self, &solution), "{:#?},{:#?}", id, self.solution_id_unchanged_layouts);

        self.reset_unchanged_layouts(solution.id());

        solution
    }

    pub fn restore_from_problem_solution(&mut self, solution: &ProblemSolution<'a>) {
        match self.solution_id_unchanged_layouts == Some(solution.id()) {
            true => {
                //A partial restore is possible.
                let mut unchanged_layouts = vec![];
                let mut modified_layouts = vec![];
                let mut absent_layouts = vec![];
                for (id,_) in self.layouts.iter() {
                    //For all layouts in the problem, check which ones occur in the solution
                    match solution.layouts().contains_key(&id) {
                        true => {
                            //layout is present in the solution
                            match self.unchanged_layouts.contains(&id) {
                                true => {
                                    //the layout is unchanged with respect to the solution, nothing needs to change
                                    unchanged_layouts.push(id);
                                }
                                false => {
                                    //the layout has been modified, it needs to be restored
                                    modified_layouts.push(id);
                                }
                            }
                        }
                        false => {
                            //layout is not present in the solution
                            absent_layouts.push(id);
                        }
                    }
                }
                absent_layouts.iter().for_each(|id| {
                    self.layouts.remove(*id);
                });
                modified_layouts.iter().for_each(|id| {
                    let copy = solution.layouts().get(&id)
                    self.layouts.insert_with(
                        |id| (solution.layouts().get(&id).unwrap().as_ref().clone(), id)
                    );
                });


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

    pub fn restore_from_instance_solution(&mut self, _solution: &SendableSolution) {
        todo!()
    }

    pub fn usage(&self) -> f64 {
        let total_included_part_area = self.instance().parts().iter().map(
            |(parttype, qty)| { parttype.area() * (*qty - self.parttype_qtys.get(parttype.id()).unwrap()) as u64 }
        ).sum::<u64>();
        let total_used_sheet_area = self.layouts().iter().map(
            |layout| { rb!(layout).sheettype().area() }
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

    pub fn register_layout(&mut self, layout: Layout<'a>) -> Index {
        self.register_sheet(layout.sheettype(), 1);
        layout.get_included_parts().iter().for_each(
            |p| { self.register_part(p, 1) });

        self.layouts.insert(layout)
    }

    pub fn unregister_layout(&mut self, layout_index: LayoutIndex) -> Layout<'a> {
        match layout_index {
            LayoutIndex::Empty(_) => panic!("Cannot unregister empty layout"),
            LayoutIndex::Existing(li) => {
                let layout = self.layouts.remove(li).expect("Layout not found");

                self.unregister_sheet(layout.sheettype(), 1);
                layout.get_included_parts().iter().for_each(
                    |p| { self.unregister_part(p, 1) });
                layout
            }
        }
    }

    fn layout_has_changed(&mut self, l_id: usize) {
        self.unchanged_layouts.remove(&l_id);
    }

    fn reset_unchanged_layouts(&mut self, unchanged_layouts_solution_id: usize) {
        self.unchanged_layouts = self.layouts.iter().map(
            |l| rb!(l).id()).collect();
        self.solution_id_unchanged_layouts = Some(unchanged_layouts_solution_id);
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
        self.solution_id_counter += 1;
        self.solution_id_counter
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
        self.solution_id_counter
    }
}

impl<'a> PartialEq for Problem<'a> {
    fn eq(&self, other: &Problem<'a>) -> bool {
        std::ptr::eq(self, other)
    }
}