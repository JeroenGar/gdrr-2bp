use std::cmp::Ordering;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};

use colored::*;

use crate::core::cost::Cost;
use crate::optimization::solutions::problem_solution::ProblemSolution;
use crate::optimization::solutions::sendable_solution::SendableSolution;
use crate::optimization::solutions::solution::Solution;
use crate::optimization::solutions::solution_stats::SolutionStats;
use crate::util::macros::{timed_println, timed_thread_println};
use crate::util::messages::{SolutionReportMessage, SyncMessage};

pub struct LocalSolCollector<'a> {
    best_complete_solution: Option<ProblemSolution<'a>>,
    best_incomplete_solution: Option<ProblemSolution<'a>>,
    cost_comparator: fn(&Cost, &Cost) -> Ordering,
    material_limit: u64,
    rx_sync: Receiver<SyncMessage>,
    tx_solution_report: Sender<SolutionReportMessage>,
    incumbent_complete: Option<Cost>,
    incumbent_incomplete: Option<Cost>,
    terminate: bool,
    best_complete_transferred: bool,
    best_incomplete_transferred: bool
}


impl<'a> LocalSolCollector<'a> {
    pub fn new(material_limit: u64,
               rx_sync: Receiver<SyncMessage>,
               tx_solution_report: Sender<SolutionReportMessage>) -> Self {
        let best_complete_solution = None;
        let best_incomplete_solution = None;
        let cost_comparator = crate::COST_COMPARATOR;
        let incumbent_complete = None;
        let incumbent_incomplete = None;
        let terminate = false;
        let best_complete_tx = false;
        let best_incomplete_tx = false;

        Self {
            best_complete_solution,
            best_incomplete_solution,
            cost_comparator,
            material_limit,
            rx_sync,
            tx_solution_report,
            incumbent_complete,
            incumbent_incomplete,
            terminate,
            best_complete_transferred: best_complete_tx,
            best_incomplete_transferred: best_incomplete_tx,
        }
    }

    pub fn report_problem_solution(&mut self, solution: &ProblemSolution<'a>) {
        self.rx_sync();
        match &self.best_incomplete_solution {
            None => {
                if solution.cost().material_cost < self.material_limit {
                    self.accept_solution(solution);
                    self.tx_solution_report();
                }
            }
            Some(best_incomplete_solution) => {
                debug_assert!(solution.cost().material_cost < self.material_limit);
                if (self.cost_comparator)(&solution.cost(), &best_incomplete_solution.cost()) == Ordering::Less {
                    self.accept_solution(solution);
                    self.tx_solution_report();
                }
            }
        };
    }

    fn accept_solution(&mut self, solution: &ProblemSolution<'a>) {
        match solution.is_complete() {
            true => {
                self.lower_matlimit(solution.cost().material_cost);
                self.best_complete_solution = Some(solution.clone());
                self.best_complete_transferred = false;
            }
            false => {
                self.best_incomplete_solution = Some(solution.clone());
                self.best_incomplete_transferred = false;
            }
        };
    }

    fn rx_sync(&mut self) {
        while let Ok(message) = self.rx_sync.try_recv() {
            match message {
                SyncMessage::NewIncumbentIncomplete(reported_cost) => {
                    //timed_thread_println!("rx {:.3}%", reported_cost.part_area_fraction_included() * 100.0);
                    self.incumbent_incomplete = Some(reported_cost);
                }
                SyncMessage::NewIncumbentComplete(reported_cost) => {
                    timed_thread_println!("syncing matlimit: {}", reported_cost.material_cost);
                    if reported_cost.material_cost < self.material_limit {
                        self.lower_matlimit(reported_cost.material_cost);
                    }
                    self.incumbent_complete = Some(reported_cost);
                    self.incumbent_incomplete = None;
                }
                SyncMessage::Terminate => {
                    timed_thread_println!("{}", "Terminate received".red());
                    self.terminate = true;
                }
            }
        }
    }

    fn tx_solution_report(&mut self) {
        match self.best_incomplete_solution.as_ref() {
            Some(best_incomplete_solution) => {
                if !self.best_incomplete_transferred {
                    if self.incumbent_incomplete.is_none() ||
                        (self.cost_comparator)(best_incomplete_solution.cost(), &self.incumbent_incomplete.as_ref().unwrap()) == Ordering::Less {
                        let thread_name = std::thread::current().name().unwrap().parse().unwrap();
                        let cost = best_incomplete_solution.cost().clone();
                        timed_thread_println!("{}", "Sending solution stats");
                        self.tx_solution_report.send(
                            SolutionReportMessage::NewIncompleteStats(thread_name, SolutionStats::new(cost, best_incomplete_solution.usage(), best_incomplete_solution.n_layouts()))
                        ).expect("Failed to send solution report message");
                    }
                    self.best_incomplete_transferred = true;
                }
            }
            None => {}
        }
        match self.best_complete_solution.as_ref() {
            Some(best_complete_solution) => {
                if !self.best_complete_transferred {
                    if self.incumbent_complete.is_none() ||
                        best_complete_solution.cost().material_cost < self.incumbent_complete.as_ref().unwrap().material_cost {
                        let thread_name = std::thread::current().name().unwrap().parse().unwrap();
                        let sendable_solution = SendableSolution::new(best_complete_solution);
                        timed_thread_println!("{}", "Sending full solution".green());
                        self.tx_solution_report.send(
                            SolutionReportMessage::NewCompleteSolution(thread_name, sendable_solution)
                        ).expect("Failed to send solution report message");
                    }
                    self.best_complete_transferred = true;
                }
            }
            None => {}
        }
    }

    fn lower_matlimit(&mut self, material_limit: u64) {
        debug_assert!(material_limit <= self.material_limit);
        self.material_limit = material_limit;
        self.best_incomplete_solution = None;
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


    pub fn terminate(&self) -> bool {
        self.terminate
    }
}