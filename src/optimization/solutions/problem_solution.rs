use std::ops::Deref;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::cost::Cost;
use crate::core::entities::layout::Layout;
use crate::Instance;
use crate::optimization::problem::Problem;
use crate::optimization::solutions::solution::Solution;
use crate::util::assertions;
use crate::util::macros::{rb};

#[derive(Debug, Clone)]
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

        for layout in problem.layouts() {
            let layout_ref = rb!(layout);
            let layout_id = layout_ref.id();
            if problem.unchanged_layouts().contains(&layout_id) {
                layouts.insert(layout_id, prev_solution.layouts.get(&layout_id).unwrap().clone());
            } else {
                layouts.insert(layout_id, Rc::new(layout_ref.create_deep_copy(layout_id)));
            }
        }

        debug_assert!(layouts.iter().all(|(_id, l)| {
            let top_node = l.as_ref().top_node();
            let top_node = rb!(top_node);
            assertions::children_nodes_fit(top_node.deref())
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

        for layout in problem.layouts() {
            let layout_ref = rb!(layout);
            layouts.insert(layout_ref.id(), Rc::new(layout_ref.create_deep_copy(layout_ref.id())));
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
