use std::cmp::Ordering;
use colored::*;
use crate::core::cost::Cost;
use crate::optimization::solutions::sendable_solution::SendableSolution;
use crate::optimization::solutions::problem_solution::ProblemSolution;
use crate::optimization::solutions::solution::Solution;
use crate::util::macros::{timed_println};

pub struct SolutionCollector<'a> {
    best_complete_solution : Option<ProblemSolution<'a>>,
    best_incomplete_solution : Option<ProblemSolution<'a>>,
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

    pub fn report_problem_solution(&mut self, solution : &ProblemSolution<'a>){
        match &self.best_incomplete_solution{
            None => {
                self.accept_solution(solution);
            },
            Some(best_incomplete_solution) => {
                if (self.cost_comparator)(&solution.cost(), &best_incomplete_solution.cost()) == Ordering::Less{
                    self.accept_solution(solution);
                }
            }
        };
    }

    fn accept_solution(&mut self, solution : &ProblemSolution<'a>){
        match solution.is_complete(){
            true => {
                self.best_incomplete_solution = None;
                self.material_limit = solution.cost().material_cost;
                timed_println!("{}: usage: {:.3}%, sheets: {:?}", "COMPLETE  ".cyan().bold(), solution.usage() * 100.0, solution.layouts().len());
                self.best_complete_solution = Some(solution.clone());
            },
            false => {
                let part_area_included_pct = (solution.instance().total_part_area() - solution.cost().part_area_excluded) as f64 / solution.instance().total_part_area() as f64 * 100.0;
                timed_println!("{}: usage: {:.3}%, sheets: {:?}, included: {:.3}%", "incomplete".bright_green(), solution.usage() * 100.0, solution.layouts().len(), part_area_included_pct);
                self.best_incomplete_solution = Some(solution.clone());
            }
        };
    }


    pub fn best_complete_solution(&self) -> &Option<ProblemSolution<'a>> {
        &self.best_complete_solution
    }
    pub fn best_incomplete_solution(&self) -> &Option<ProblemSolution<'a>> {
        &self.best_incomplete_solution
    }
    pub fn cost_comparator(&self) -> fn(&Cost, &Cost) -> Ordering {
        self.cost_comparator
    }
    pub fn material_limit(&self) -> u64 {
        self.material_limit
    }
}