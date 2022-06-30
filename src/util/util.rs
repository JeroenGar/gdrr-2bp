use crate::optimization::solutions::problem_solution::ProblemSolution;
use crate::optimization::solutions::solution::Solution;

pub fn solution_stats_string(solution : &dyn Solution) -> String{
    format!("usage: {:.3}%, sheets: {}", solution.usage() * 100.0, solution.n_layouts())
}