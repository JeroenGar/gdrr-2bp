use generational_arena::{Arena, Index};
use rand::{SeedableRng, thread_rng};
use rand::rngs::SmallRng;

use crate::{DETERMINISTIC_MODE, Instance, Orientation};
use crate::core::cost::Cost;
use crate::core::entities::layout::Layout;
use crate::core::insertion::insertion_blueprint::InsertionBlueprint;
use crate::core::layout_index::LayoutIndex;
use crate::optimization::rr::cache_updates::CacheUpdates;
use crate::optimization::solutions::problem_solution::ProblemSolution;
use crate::optimization::solutions::sendable_solution::SendableSolution;
use crate::optimization::solutions::solution::Solution;
use crate::util::assertions;

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
    rng: SmallRng,
    changed_layouts: Vec<usize>,
    solution_id_changed_layouts: Option<usize>,
    solution_id_counter: usize,
    layout_id_counter: usize,
}

impl<'a> Problem<'a> {
    pub fn new(instance: &'a Instance) -> Self {
        let parttype_qtys = instance.parts().iter().map(|(_, qty)| *qty).collect::<Vec<_>>();
        let sheettype_qtys = instance.sheets().iter().map(|(_, qty)| *qty).collect::<Vec<_>>();
        let random = match DETERMINISTIC_MODE {
            true => SmallRng::seed_from_u64(0),
            false => SmallRng::from_rng(thread_rng()).unwrap()
        };

        let mut problem = Problem {
            instance,
            parttype_qtys,
            sheettype_qtys,
            layouts : Arena::new(),
            empty_layouts : Vec::new(),
            changed_layouts : Vec::new(),
            solution_id_changed_layouts : None,
            rng: random,
            solution_id_counter : 0,
            layout_id_counter : 0,
        };

        //Initiate the empty layouts
        for (sheettype, _) in instance.sheets() {
            match sheettype.fixed_first_cut_orientation() {
                Some(orientation) => {
                    let empty_layout = Layout::new(problem.next_layout_id(), sheettype, orientation);
                    problem.empty_layouts.push(empty_layout);
                }
                None => {
                    let empty_layout_h = Layout::new(problem.next_layout_id(), sheettype, Orientation::Horizontal);
                    let empty_layout_v = Layout::new(problem.next_layout_id(), sheettype, Orientation::Vertical);
                    problem.empty_layouts.extend([empty_layout_h, empty_layout_v]);
                }
            }
        }
        problem
    }

    /// Modifies the problem by inserting an part according to the InsertionBlueprint.
    /// It returns which updates should be made to the InsertionOptionCache and whether or not a new layout was created.
    pub fn implement_insertion_blueprint(&mut self, blueprint: &InsertionBlueprint<'a>) -> CacheUpdates<Index> {
        self.register_part(blueprint.parttype().id(), 1);

        match blueprint.layout_index() {
            LayoutIndex::Existing(index) => {
                let blueprint_layout = &mut self.layouts[*index];
                let mut cache_updates = CacheUpdates::new(*blueprint.layout_index());
                blueprint_layout.implement_insertion_blueprint(blueprint, &mut cache_updates, self.instance);

                let blueprint_layout_id = blueprint_layout.id();
                self.layout_has_changed(blueprint_layout_id);

                cache_updates
            }
            LayoutIndex::Empty(index) => {
                let next_layout_id = self.next_layout_id();
                let empty_layout = &self.empty_layouts[*index];

                //Create a copy of the empty layout and register it
                let empty_layout_clone = empty_layout.clone_with_id(next_layout_id);
                let clone_index = self.register_layout(empty_layout_clone);

                //Implement the blueprint
                let mut cache_updates = CacheUpdates::new(LayoutIndex::Existing(clone_index));
                self.layouts[clone_index].implement_insertion_blueprint(blueprint, &mut cache_updates, self.instance);

                cache_updates
            }
        }
    }

    pub fn remove_node(&mut self, node_index: Index, layout_index: LayoutIndex) -> Option<Layout<'a>> {
        self.layout_has_changed(self.get_layout(&layout_index).id());
        match layout_index {
            LayoutIndex::Empty(_) => panic!("Cannot remove a node from an empty layout"),
            LayoutIndex::Existing(index) => {
                let layout = &mut self.layouts[index];
                match node_index == *layout.top_node_index() {
                    true => {
                        //Remove the entire layout
                        Some(self.unregister_layout(layout_index))
                    }
                    false => {
                        let removed_part_ids = layout.remove_node(node_index);
                        for p_id in removed_part_ids {
                            self.unregister_part(p_id, 1);
                        }

                        if self.get_layout(&layout_index).is_empty() {
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
        //TODO: implement cached cost for problem

        debug_assert!(cached_cost.is_none() || cached_cost.as_ref().unwrap() == &self.cost());
        let id = self.next_solution_id();
        let cost = cached_cost.unwrap_or(self.cost());
        let solution = match old_solution {
            Some(old_solution) => {
                debug_assert!(old_solution.id() == self.solution_id_changed_layouts.unwrap());
                ProblemSolution::new(self, cost, id, old_solution)
            }
            None => {
                ProblemSolution::new_force_copy_all(self, cost, id)
            }
        };

        debug_assert!(assertions::problem_matches_solution(self, &solution), "{:#?},{:#?}", id, self.solution_id_changed_layouts);

        self.reset_changed_layouts(solution.id());

        solution
    }

    pub fn restore_from_problem_solution(&mut self, solution: &ProblemSolution<'a>) {
        match self.solution_id_changed_layouts == Some(solution.id()) {
            true => {
                //A partial restore is possible.

                //Check all the layouts in the problem and check if they are either modified, unmodified or absent in the solution.
                let mut present_layout_ids = vec![];
                let mut changed_layout_indices = vec![];
                let mut absent_layout_indices = vec![];

                for (index,layout) in self.layouts.iter() {
                    //For all layouts in the problem, check which ones occur in the solution
                    let layout_id = layout.id();
                    match solution.layouts().contains_key(&layout_id) {
                        true => {
                            //layout is present in the solution
                            if self.changed_layouts.contains(&layout_id) {
                                changed_layout_indices.push(index)
                            }
                            present_layout_ids.push(layout_id);
                        }
                        false => {
                            absent_layout_indices.push(index);
                        }
                    }
                }
                for i in absent_layout_indices{
                    self.layouts.remove(i);
                }
                for i in changed_layout_indices {
                    let layout = self.layouts.remove(i).expect("Layout should be present");
                    let copy = solution.layouts().get(&layout.id()).unwrap().as_ref().clone();
                    self.layouts.insert(copy);
                }

                //Some layouts are present in the solution, but not in the problem
                for id in solution.layouts().keys() {
                    if !present_layout_ids.contains(id) {
                        let copy = solution.layouts().get(id).unwrap().as_ref().clone();
                        self.layouts.insert(copy);
                    }
                }
            }
            false => {
                //The id of the solution does not match unchanged_layouts_solution_id, a partial restore is not possible
                self.layouts.clear();
                for (_, layout) in solution.layouts().iter() {
                    let copy = layout.as_ref().clone();
                    self.layouts.insert(copy);
                }
            }
        }

        self.parttype_qtys = solution.parttype_qtys().clone();
        self.sheettype_qtys = solution.sheettype_qtys().clone();

        debug_assert!(assertions::problem_matches_solution(self, solution));

        self.reset_changed_layouts(solution.id());
    }

    pub fn restore_from_instance_solution(&mut self, _solution: &SendableSolution) {
        todo!()
    }

    pub fn usage(&self) -> f64 {
        let total_included_part_area = self.instance().parts().iter().map(
            |(parttype, qty)| { parttype.area() * (*qty - self.parttype_qtys.get(parttype.id()).unwrap()) as u64 }
        ).sum::<u64>();
        let total_used_sheet_area = self.layouts().iter().map(
            |(_, layout)| { layout.sheettype().area() }
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

    pub fn rng(&mut self) -> &mut SmallRng {
        &mut self.rng
    }

    pub fn layouts(&self) -> &Arena<Layout<'a>> {
        &self.layouts
    }

    pub fn layouts_mut(&mut self) -> &mut Arena<Layout<'a>> {
        &mut self.layouts
    }

    pub fn get_layout(&self, layout_index: &LayoutIndex) -> &Layout<'a>{
        match layout_index{
            LayoutIndex::Existing(index) => self.layouts.get(*index).unwrap(),
            LayoutIndex::Empty(index) => self.empty_layouts.get(*index).unwrap(),
        }
    }

    pub fn register_layout(&mut self, layout: Layout<'a>) -> Index {
        self.register_sheet(layout.sheettype().id(), 1);
        layout.get_included_parts().iter().for_each(
            |p_id| {
                self.register_part(*p_id, 1);
            });
        self.layout_has_changed(layout.id());
        self.layouts.insert(layout)
    }

    pub fn unregister_layout(&mut self, layout_index: LayoutIndex) -> Layout<'a> {
        match layout_index {
            LayoutIndex::Empty(_) => panic!("Cannot unregister empty layout"),
            LayoutIndex::Existing(li) => {
                let layout = self.layouts.remove(li).expect("Layout not found");

                self.unregister_sheet(layout.sheettype().id(), 1);
                layout.get_included_parts().iter().for_each(
                    |p_id| { self.unregister_part(*p_id, 1) });
                layout
            }
        }
    }

    fn layout_has_changed(&mut self, l_id: usize) {
        self.changed_layouts.push(l_id);
    }

    fn reset_changed_layouts(&mut self, solution_id_changed_layouts: usize) {
        self.changed_layouts.clear();
        self.solution_id_changed_layouts = Some(solution_id_changed_layouts);
    }

    fn register_part(&mut self, parttype_id: usize, qty: usize) {
        self.parttype_qtys[parttype_id] -= qty;
    }

    fn unregister_part(&mut self, parttype_id: usize, qty: usize) {
        debug_assert!(self.parttype_qtys[parttype_id] + qty <= self.instance.get_parttype_qty(parttype_id).unwrap());
        self.parttype_qtys[parttype_id] += qty;
    }

    fn register_sheet(&mut self, sheettype_id: usize, qty: usize) {
        self.sheettype_qtys[sheettype_id] -= qty;
    }

    fn unregister_sheet(&mut self, sheettype_id: usize, qty: usize) {
        debug_assert!(self.sheettype_qtys[sheettype_id] + qty <= self.instance.get_sheettype_qty(sheettype_id).unwrap());
        self.sheettype_qtys[sheettype_id] += qty;
    }

    fn next_layout_id(&mut self) -> usize {
        self.layout_id_counter += 1;
        self.layout_id_counter
    }

    fn next_solution_id(&mut self) -> usize {
        self.solution_id_counter += 1;
        self.solution_id_counter
    }

    pub fn empty_layouts(&self) -> &Vec<Layout<'a>> {
        &self.empty_layouts
    }

    pub fn changed_layouts(&self) -> &Vec<usize> {
        &self.changed_layouts
    }
}

impl<'a> PartialEq for Problem<'a> {
    fn eq(&self, other: &Problem<'a>) -> bool {
        std::ptr::eq(self, other)
    }
}