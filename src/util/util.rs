use crate::optimization::solutions::solution::Solution;
use crate::optimization::solutions::solution_stats::SolutionStats;

pub fn solution_stats_string(solution: &dyn Solution) -> String {
    format!(
        "(usage: {:.3}%, p_incl: {:.3}%, sheets: {}, mat: {})",
        solution.usage() * 100.0,
        solution.cost().part_area_fraction_included() * 100.0,
        solution.n_layouts(),
        solution.cost().material_cost)
}

pub fn compact_stats_string(stats: &SolutionStats) -> String {
    format!(
        "(usage: {:.3}%, p_incl: {:.3}%, sheets: {}, mat: {})",
        stats.usage * 100.0,
        stats.cost.part_area_fraction_included() * 100.0,
        stats.n_sheets,
        stats.cost.material_cost)
}