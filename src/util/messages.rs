use crate::core::cost::Cost;
use crate::optimization::solutions::sendable_solution::SendableSolution;
use crate::optimization::solutions::solution_stats::SolutionStats;

pub enum SyncMessage {
    SyncMatLimit(u64),
    Terminate
}

pub enum SolutionReportMessage {
    NewCompleteSolution(String, SendableSolution),
    NewIncompleteStats(String, SolutionStats),
    NewIncompleteSolution(String, SendableSolution)
}

