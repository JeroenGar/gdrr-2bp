use std::rc::Rc;

use slotmap::SecondaryMap;

use crate::core::cost::Cost;
use crate::core::entities::layout::Layout;
use crate::core::layout_index::LayoutKey;
use crate::optimization::instance::Instance;
use crate::optimization::problem::Problem;
use crate::optimization::solutions::solution::Solution;
use crate::util::assertions;

#[derive(Debug, Clone)]
/// ProblemSolution represents an immutable snapshot of a Problem at some point in time.
/// Its primary use is restoring a Problem to a prior state.
pub struct ProblemSolution<'a> {
    instance: &'a Instance,
    layouts: SecondaryMap<LayoutKey, Rc<Layout<'a>>>,
    cost: Cost,
    id: usize,
    parttype_qtys: Vec<usize>,
    sheettype_qtys: Vec<usize>,
    usage: f64,
}

impl<'a> ProblemSolution<'a> {
    pub fn new(problem: &Problem<'a>, cost: Cost, id: usize, mut prev_solution: ProblemSolution<'a>) -> ProblemSolution<'a> {
        for &layout_key in problem.changed_layouts() {
            match problem.layouts().get(layout_key) {
                Some(layout) => {
                    prev_solution.layouts.insert(layout_key, Rc::new(layout.clone()));
                }
                None => {
                    prev_solution.layouts.remove(layout_key);
                }
            }
        }

        debug_assert!(prev_solution.layouts.iter().all(|(_id, l)| {
            let top_node = l.top_node_index();
            assertions::children_nodes_fit(top_node, l.nodes())
        }));

        prev_solution.cost = cost;
        prev_solution.id = id;
        prev_solution.parttype_qtys = problem.parttype_qtys().clone();
        prev_solution.sheettype_qtys = problem.sheettype_qtys().clone();
        prev_solution.usage = problem.usage();
        prev_solution
    }

    pub fn new_force_copy_all(problem: &Problem<'a>, cost: Cost, id: usize) -> ProblemSolution<'a> {
        let mut layouts = SecondaryMap::with_capacity(problem.layouts().capacity());

        for (layout_key, layout) in problem.layouts() {
            layouts.insert(layout_key, Rc::new(layout.clone()));
        }

        let parttype_qtys = problem.parttype_qtys().clone();
        let sheettype_qtys = problem.sheettype_qtys().clone();

        let usage = problem.usage();

        Self {
            instance: problem.instance(),
            layouts,
            cost,
            id,
            parttype_qtys,
            sheettype_qtys,
            usage,
        }
    }


    pub fn instance(&self) -> &'a Instance {
        self.instance
    }
    pub fn layouts(&self) -> &SecondaryMap<LayoutKey, Rc<Layout<'a>>> {
        &self.layouts
    }
    pub fn id(&self) -> usize {
        self.id
    }
}

impl<'a> Solution for ProblemSolution<'a> {
    fn cost(&self) -> &Cost {
        &self.cost
    }
    fn n_layouts(&self) -> usize {
        self.layouts.len()
    }
    fn parttype_qtys(&self) -> &Vec<usize> {
        &self.parttype_qtys
    }
    fn sheettype_qtys(&self) -> &Vec<usize> {
        &self.sheettype_qtys
    }
    fn usage(&self) -> f64 {
        self.usage
    }
}
