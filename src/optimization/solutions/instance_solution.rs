use std::rc::Rc;
use indexmap::IndexMap;
use crate::core::cost::Cost;
use crate::core::entities::layout::Layout;
use crate::Instance;
use crate::optimization::solutions::problem_solution::ProblemSolution;
use crate::optimization::solutions::solution::Solution;

#[derive(Debug, Clone)]
pub struct InstanceSolution<'a> {
    instance : &'a Instance,
    layouts : IndexMap<usize, Rc<Layout<'a>>>,
    cost : Cost,
    parttype_qtys : Vec<usize>,
    sheettype_qtys : Vec<usize>
}

impl<'a> InstanceSolution<'a>{
    pub fn new(problem_solution : &ProblemSolution<'a>) -> InstanceSolution<'a>{
        let instance = problem_solution.instance();
        let layouts = problem_solution.layouts().clone();
        let cost = problem_solution.cost().clone();
        let parttype_qtys = problem_solution.parttype_qtys().clone();
        let sheettype_qtys = problem_solution.sheettype_qtys().clone();

        Self {
            instance,
            layouts,
            cost,
            parttype_qtys,
            sheettype_qtys
        }
    }
}

impl<'a> Solution<'a> for InstanceSolution<'a> {
    fn cost(&self) -> &Cost {
        &self.cost
    }

    fn instance(&self) -> &Instance {
        self.instance
    }

    fn layouts(&self) -> &IndexMap<usize, Rc<Layout<'a>>> {
        &self.layouts
    }

    fn parttype_qtys(&self) -> &Vec<usize> {
        &self.parttype_qtys
    }

    fn sheettype_qtys(&self) -> &Vec<usize> {
        &self.sheettype_qtys
    }

    fn usage(&self) -> f64 {
        todo!()
    }
}