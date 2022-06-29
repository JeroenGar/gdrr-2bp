use std::cmp::Ordering;
use crate::core::cost::Cost;
use crate::optimization::solutions::instance_solution::InstanceSolution;
use crate::optimization::solutions::problem_solution::ProblemSolution;
use crate::optimization::solutions::solution::Solution;
use crate::util::macros::{timed_println};

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
                self.accept_solution(solution)
            },
            Some(best_incomplete_solution) => {
                if (self.cost_comparator)(&solution.cost(), &best_incomplete_solution.cost()) == Ordering::Less{
                    self.accept_solution(solution)
                }
            }
        };
    }

    pub fn report_problem_solution(&mut self, solution : &ProblemSolution<'a>){
        match &self.best_incomplete_solution{
            None => {
                self.accept_solution(InstanceSolution::new(&solution));
            },
            Some(best_incomplete_solution) => {
                if (self.cost_comparator)(&solution.cost(), &best_incomplete_solution.cost()) == Ordering::Less{
                    self.accept_solution(InstanceSolution::new(&solution));
                }
            }
        };
    }

    fn accept_solution(&mut self, solution : InstanceSolution<'a>){
        match solution.is_complete(){
            true => {
                self.best_incomplete_solution = None;
                self.material_limit = solution.cost().material_cost;
                timed_println!("New best complete solution: {:?} sheets, {:?} material value", solution.layouts().len(), solution.cost().material_cost);
                self.best_complete_solution = Some(solution);
            },
            false => {
                let part_area_included_pct = (solution.instance().total_part_area() - solution.cost().part_area_excluded) as f64 / solution.instance().total_part_area() as f64 * 100.0;
                timed_println!("New best incomplete solution: {:?} sheets, {:.3}% parts included", solution.layouts().len(), part_area_included_pct);
                self.best_incomplete_solution = Some(solution);
            }
        };
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