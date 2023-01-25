use std::sync::Arc;
use crate::core::cost::Cost;
use crate::core::entities::sendable_layout::SendableLayout;
use crate::optimization::instance::Instance;
use crate::optimization::solutions::problem_solution::ProblemSolution;
use crate::optimization::solutions::solution::Solution;

/// Representation of a solution, based on ProblemSolution, but that can be sent across threads

#[derive(Debug, Clone)]
pub struct SendableSolution {
    instance: Arc<Instance>,
    layouts: Vec<SendableLayout>,
    parttype_qtys: Vec<usize>,
    sheettype_qtys: Vec<usize>,
    cost: Cost,
    usage: f64,
}

impl SendableSolution {
    pub fn new(instance: Arc<Instance>, problem_solution: &ProblemSolution) -> SendableSolution {
        debug_assert!(instance.as_ref() as *const _ == problem_solution.instance() as *const _);

        let layouts = problem_solution.layouts().iter().map(|(_id, l)| SendableLayout::new(l)).collect();
        let cost = problem_solution.cost().clone();
        let usage = problem_solution.usage();
        let parttype_qtys = problem_solution.parttype_qtys().clone();
        let sheettype_qtys = problem_solution.sheettype_qtys().clone();

        Self {
            instance,
            layouts,
            cost,
            usage,
            parttype_qtys,
            sheettype_qtys,
        }
    }

    pub fn layouts(&self) -> &Vec<SendableLayout> {
        &self.layouts
    }

    pub fn instance(&self) -> &Arc<Instance> {
        &self.instance
    }
}


impl Solution for SendableSolution {
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