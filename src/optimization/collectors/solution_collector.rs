use std::cmp::Ordering;
use crate::core::cost::Cost;
use crate::optimization::solutions::instance_solution::InstanceSolution;
use crate::optimization::solutions::problem_solution::ProblemSolution;
use crate::optimization::solutions::solution::Solution;

pub struct SolutionCollector<'a> {
    best_complete_solution : Option<InstanceSolution<'a>>,
    best_incomplete_solution : Option<InstanceSolution<'a>>,
    cost_comparator : fn(&Cost, &Cost) -> Ordering,
    material_limit : u64
}


impl<'a> SolutionCollector<'a> {
    pub fn new(cost_comparator : fn(&Cost, &Cost) -> Ordering, material_limit : u64) -> Self {
        let best_complete_solution = None;
        let best_incomplete_solution = None;

        Self {
            best_complete_solution,
            best_incomplete_solution,
            cost_comparator,
            material_limit
        }
    }

    pub fn report_instance_solution(&mut self, solution : InstanceSolution<'a>){
        match &self.best_incomplete_solution{
            None => {
                self.replace_best_incomplete_solution(solution)
            },
            Some(best_incomplete_solution) => {
                if (self.cost_comparator)(&solution.cost(), &best_incomplete_solution.cost()) == Ordering::Less{
                    self.replace_best_incomplete_solution(solution)
                }
            }
        };
        todo!()
    }

    pub fn report_problem_solution(&mut self, solution : &ProblemSolution<'a>){
        match &self.best_incomplete_solution{
            None => {
                self.replace_best_incomplete_solution(InstanceSolution::new(&solution));
            },
            Some(best_incomplete_solution) => {
                if (self.cost_comparator)(&solution.cost(), &best_incomplete_solution.cost()) == Ordering::Less{
                    self.replace_best_incomplete_solution(InstanceSolution::new(&solution));
                }
            }
        };
        todo!()
    }

    fn replace_best_incomplete_solution(&mut self, solution : InstanceSolution<'a>){
        self.best_incomplete_solution = Some(solution);

        todo!("check for complete solution")
    }


    pub fn best_complete_solution(&self) -> &Option<InstanceSolution<'a>> {
        &self.best_complete_solution
    }
    pub fn best_incomplete_solution(&self) -> &Option<InstanceSolution<'a>> {
        &self.best_incomplete_solution
    }
    pub fn cost_comparator(&self) -> fn(&Cost, &Cost) -> Ordering {
        self.cost_comparator
    }
    pub fn material_limit(&self) -> u64 {
        self.material_limit
    }
}