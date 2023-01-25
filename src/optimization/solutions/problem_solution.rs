use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::cost::Cost;
use crate::core::entities::layout::Layout;
use crate::Instance;
use crate::optimization::problem::Problem;
use crate::optimization::solutions::solution::Solution;
use crate::util::assertions;

#[derive(Debug, Clone)]
/// ProblemSolution represents an immutable snapshot of a Problem at some point in time.
/// Its primary use is restoring a Problem to a prior state.
pub struct ProblemSolution<'a> {
    instance: &'a Instance,
    layouts: IndexMap<usize, Rc<Layout<'a>>>,
    cost: Cost,
    id: usize,
    parttype_qtys: Vec<usize>,
    sheettype_qtys: Vec<usize>,
    usage: f64,
}

impl<'a> ProblemSolution<'a> {
    pub fn new(problem: &Problem<'a>, cost: Cost, id: usize, prev_solution: &ProblemSolution<'a>) -> ProblemSolution<'a> {
        let mut layouts = IndexMap::new();

        for (_,layout) in problem.layouts() {
            let layout_id = layout.id();
            if problem.changed_layouts().contains(&layout_id) {
                layouts.insert(layout_id, Rc::new(layout.clone()));
            } else {
                let prev_solution_layout = prev_solution.layouts.get(&layout_id).expect("Unchanged layout not found in previous solution");
                layouts.insert(layout_id, prev_solution_layout.clone());
            }
        }

        debug_assert!(layouts.iter().all(|(_id, l)| {
            let top_node = l.top_node_index();
            assertions::children_nodes_fit(top_node, l.nodes())
        }));

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

    pub fn new_force_copy_all(problem: &Problem<'a>, cost: Cost, id: usize) -> ProblemSolution<'a> {
        let mut layouts = IndexMap::new();

        for (_,layout) in problem.layouts() {
            layouts.insert(layout.id(), Rc::new(layout.clone()));
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
    pub fn layouts(&self) -> &IndexMap<usize, Rc<Layout<'a>>> {
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
