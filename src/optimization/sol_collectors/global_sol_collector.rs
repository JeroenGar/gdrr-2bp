use std::{thread, time};
use std::cmp::Ordering;
use std::sync::{Arc, atomic};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

use colored::*;

use crate::core::cost::Cost;
use crate::optimization::config::Config;
use crate::optimization::instance::Instance;
use crate::optimization::solutions::sendable_solution::SendableSolution;
use crate::optimization::solutions::solution::Solution;
use crate::optimization::solutions::solution_stats::SolutionStats;
use crate::timed_println;
use crate::util::messages::{SolutionReportMessage, SyncMessage};
use crate::util::util;

const MONITOR_INTERVAL: Duration = Duration::from_millis(10);

/// Global Solution Collector
/// communicates with a set of LocalSolCollectors
/// It receives solutions and sends out sync messages (material limit lowering, terminate)

pub struct GlobalSolCollector {
    _instance: Arc<Instance>,
    config: Arc<Config>,
    best_complete_solution: Option<SendableSolution>,
    best_incomplete_solution: Option<SendableSolution>,
    best_incomplete_cost: Option<Cost>,
    cost_comparator: fn(&Cost, &Cost) -> Ordering,
    material_limit: Option<u64>,
    tx_syncs: Vec<Sender<SyncMessage>>,
    rx_solution_report: Receiver<SolutionReportMessage>,
}

impl GlobalSolCollector {
    pub fn new(_instance: Arc<Instance>,
               config: Arc<Config>,
               tx_syncs: Vec<Sender<SyncMessage>>,
               rx_solution_report: Receiver<SolutionReportMessage>,
               cost_comparator: fn(&Cost, &Cost) -> Ordering,
    ) -> Self {
        Self {
            _instance,
            config,
            best_complete_solution : None,
            best_incomplete_solution : None,
            best_incomplete_cost : None,
            cost_comparator,
            material_limit : None,
            tx_syncs,
            rx_solution_report,
        }
    }

    pub fn monitor(&mut self, gdrr_thread_handlers: Vec<thread::JoinHandle<()>>) {
        let start_time = time::Instant::now();
        let max_run_time = self.config.max_run_time.unwrap_or(usize::MAX);
        let running = Arc::new(AtomicBool::new(true));
        let r = running.clone();

        ctrlc::set_handler(move || {
            r.store(false, atomic::Ordering::SeqCst);
        }).expect("Error setting Ctrl-C handler");

        while running.load(atomic::Ordering::SeqCst) &&
            (time::Instant::now() - start_time).as_secs() < max_run_time as u64 {
            thread::sleep(MONITOR_INTERVAL);

            while let Ok(message) = self.rx_solution_report.try_recv() {
                match message {
                    SolutionReportMessage::NewCompleteSolution(thread_name, solution) => {
                        self.report_new_complete_solution(thread_name, solution);
                    }
                    SolutionReportMessage::NewIncompleteStats(thread_name, stats) => {
                        self.report_new_incomplete_cost(thread_name, stats);
                    }
                    SolutionReportMessage::NewIncompleteSolution(thread_name, solution) => {
                        self.report_new_incomplete_solution(thread_name, solution);
                    }
                }
            }
            if self.material_limit.unwrap_or(u64::MAX) == self._instance.smallest_sheet_value(){
                timed_println!("Minimum material limit reached");
                break;
            }

            if gdrr_thread_handlers.iter().all(|h| h.is_finished()) {
                timed_println!("All GDRR threads have finished execution");
                break;
            }
        }
        timed_println!("{}","Terminating global monitor".bold().red());
        //Send the termination signal to all threads
        for tx_sync in &self.tx_syncs {
            match tx_sync.send(SyncMessage::Terminate) {
                Ok(_) => {}
                Err(_) => {}
            }
        }
        //Wait for them to finish
        for handler in gdrr_thread_handlers {
            handler.join().expect("Error joining GDRR thread");
        }

        match (self.best_complete_solution.as_ref(), self.best_incomplete_cost.as_ref()) {
            (Some(_best_complete_solution), _) => {
                timed_println!("{}:\t {}",
                    "Final global solution".cyan().bold(),
                    util::solution_stats_string(self.best_complete_solution.as_ref().unwrap()));
            }
            (None, Some(_best_incomplete_cost)) => {}
            (None, None) => {
                timed_println!("{}","No Global Solution".bright_red().bold());
            }
        }
    }

    fn report_new_complete_solution(&mut self, thread_name: String, solution: SendableSolution) {
        if solution.cost().material_cost < self.material_limit.unwrap_or(u64::MAX) {
            if self.best_complete_solution.is_none()
                || solution.cost().material_cost < self.best_complete_solution.as_ref().unwrap().cost().material_cost {
                self.best_incomplete_cost = None;
                self.best_incomplete_solution = None;
                self.material_limit = Some(solution.cost().material_cost);
                timed_println!("[{}]\t{}{}", thread_name, "<complete>\t".cyan().bold(), util::solution_stats_string(&solution).cyan().bold());
                self.best_complete_solution = Some(solution.clone());

                for tx_sync in &self.tx_syncs {
                    tx_sync.send(SyncMessage::SyncMatLimit(solution.cost().material_cost)).expect("Error sending sync matlimit message");
                }
            }
        }
    }

    fn report_new_incomplete_solution(&mut self, thread_name: String, solution: SendableSolution) {
        if self.best_complete_solution.is_none() {
            if self.best_incomplete_solution.is_none()
                || (self.cost_comparator)(&solution.cost(), &self.best_incomplete_solution.as_ref().unwrap().cost()) == Ordering::Less {
                timed_println!("[{}]\t{}{}", thread_name, "<incomplete>\t".bright_green(), util::solution_stats_string(&solution));
                self.best_incomplete_solution = Some(solution.clone());
            }
        }
    }

    fn report_new_incomplete_cost(&mut self, thread_name: String, stats: SolutionStats) {
        if stats.cost.material_cost < self.material_limit.unwrap_or(u64::MAX) {
            if self.best_incomplete_cost.is_none()
                || (self.cost_comparator)(&stats.cost, &self.best_incomplete_cost.as_ref().unwrap()) == Ordering::Less {
                timed_println!("[{}]\t{}{}", thread_name, "<incomplete>\t".bright_green(), util::compact_stats_string(&stats));
                self.best_incomplete_cost = Some(stats.cost.clone());
            }
        }
    }


    pub fn best_complete_solution(&self) -> &Option<SendableSolution> {
        &self.best_complete_solution
    }
    pub fn best_incomplete_solution(&self) -> &Option<SendableSolution> {
        &self.best_incomplete_solution
    }
    pub fn best_incomplete_cost(&self) -> &Option<Cost> {
        &self.best_incomplete_cost
    }

    pub fn material_limit(&self) -> Option<u64> {
        self.material_limit
    }
}