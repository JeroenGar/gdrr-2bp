use std::rc::Rc;
use indexmap::IndexMap;
use crate::core::cost::Cost;
use crate::core::entities::layout::Layout;
use crate::core::entities::sendable_layout::SendableLayout;
use crate::Instance;
use crate::optimization::solutions::problem_solution::ProblemSolution;
use crate::optimization::solutions::solution::Solution;
use crate::util::macros::{rb,rbm};

#[derive(Debug, Clone)]
pub struct SendableSolution<'a> {
    instance : &'a Instance,
    layouts : Vec<SendableLayout>,
    cost : Cost,
    usage : f64,
    parttype_qtys : Vec<usize>,
    sheettype_qtys : Vec<usize>
}

impl<'a> SendableSolution<'a>{
    pub fn new(problem_solution : &ProblemSolution<'a>) -> SendableSolution<'a>{
        let instance = problem_solution.instance();
        let layouts = problem_solution.layouts().iter().map(|(id, l)| SendableLayout::new(l)).collect();
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
            sheettype_qtys
        }
    }
}


impl<'a> Solution for SendableSolution<'a> {
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