use std::cmp::Ordering;
use std::io::{self, IsTerminal};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, atomic};
use std::thread;
use std::time::{Duration, Instant};

use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};

use crate::core::cost::Cost;
use crate::optimization::config::Config;
use crate::optimization::gdrr::OptimizationStats;
use crate::optimization::instance::Instance;
use crate::optimization::solutions::sendable_solution::SendableSolution;
use crate::optimization::solutions::solution::Solution;
use crate::optimization::solutions::solution_stats::SolutionStats;
use crate::util::messages::{SolutionReportMessage, SyncMessage};
use crate::util::util;

const MONITOR_INTERVAL: Duration = Duration::from_millis(50);
const UI_REFRESH_INTERVAL: Duration = Duration::from_millis(100);

struct RunUi {
    interactive: Option<InteractiveUi>,
    started: Instant,
}

struct InteractiveUi {
    progress: MultiProgress,
    workers: Vec<ProgressBar>,
    leader: Option<usize>,
}

impl RunUi {
    fn new(enabled: bool, n_workers: usize) -> Self {
        let interactive = (enabled && io::stderr().is_terminal()).then(|| {
            let progress = MultiProgress::with_draw_target(ProgressDrawTarget::stderr_with_hz(10));
            let workers = (0..n_workers)
                .map(|index| {
                    let worker = progress.add(ProgressBar::new_spinner());
                    worker.set_style(worker_style(false));
                    worker.set_prefix(format!("T{index}"));
                    worker.set_message("waiting for first solution");
                    worker
                })
                .collect();
            InteractiveUi {
                progress,
                workers,
                leader: None,
            }
        });
        Self {
            interactive,
            started: Instant::now(),
        }
    }

    fn update(&mut self, thread_name: &str, message: String, leading: bool) {
        if let Some(interactive) = &mut self.interactive {
            let Some(index) = thread_name
                .strip_prefix('T')
                .and_then(|index| index.parse::<usize>().ok())
                .filter(|&index| index < interactive.workers.len())
            else {
                return;
            };

            interactive.workers[index].set_message(message);
            if leading && interactive.leader != Some(index) {
                if let Some(previous) = interactive.leader {
                    interactive.workers[previous].set_style(worker_style(false));
                }
                interactive.workers[index].set_style(worker_style(true));
                interactive.leader = Some(index);
            }
        } else if leading {
            println!("[{}] [{thread_name}] {message}", self.timestamp());
        }
    }

    fn log_best_feasible(&self, thread_name: &str, message: String) {
        if let Some(interactive) = &self.interactive {
            let _ = interactive.progress.println(format!(
                "[{}] [{thread_name}] new best feasible {message}",
                self.timestamp()
            ));
        }
    }

    fn tick(&self) {
        if let Some(interactive) = &self.interactive {
            for worker in &interactive.workers {
                worker.tick();
            }
        }
    }

    fn finish(&self, message: String) {
        if let Some(interactive) = &self.interactive {
            for worker in &interactive.workers {
                worker.finish_and_clear();
            }
        }
        println!("[{}] {message}", self.timestamp());
    }

    fn timestamp(&self) -> String {
        let elapsed = self.started.elapsed().as_secs();
        format!(
            "{:02}:{:02}:{:02}",
            elapsed / 3600,
            elapsed / 60 % 60,
            elapsed % 60
        )
    }
}

fn worker_style(leading: bool) -> ProgressStyle {
    let template = if leading {
        "{spinner:.green} [{elapsed_precise}] {prefix:>3.green} {wide_msg:.green}"
    } else {
        "{spinner:.cyan} [{elapsed_precise}] {prefix:>3} {wide_msg}"
    };
    ProgressStyle::with_template(template)
        .expect("progress template is valid")
        .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
}

/// Collects solutions from worker-local collectors and synchronizes material limits.
pub struct GlobalSolCollector {
    instance: Arc<Instance>,
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
    pub fn new(
        instance: Arc<Instance>,
        config: Arc<Config>,
        tx_syncs: Vec<Sender<SyncMessage>>,
        rx_solution_report: Receiver<SolutionReportMessage>,
        cost_comparator: fn(&Cost, &Cost) -> Ordering,
    ) -> Self {
        Self {
            instance,
            config,
            best_complete_solution: None,
            best_incomplete_solution: None,
            best_incomplete_cost: None,
            cost_comparator,
            material_limit: None,
            tx_syncs,
            rx_solution_report,
        }
    }

    pub fn monitor(
        &mut self,
        gdrr_thread_handlers: Vec<thread::JoinHandle<OptimizationStats>>,
        show_progress: bool,
    ) -> Duration {
        let start_time = Instant::now();
        let max_run_time = self.config.max_run_time.unwrap_or(usize::MAX);
        let running = Arc::new(AtomicBool::new(true));
        let interrupt_flag = running.clone();
        let mut ui = RunUi::new(show_progress, gdrr_thread_handlers.len());
        let mut next_ui_refresh = Instant::now() + UI_REFRESH_INTERVAL;

        ctrlc::set_handler(move || {
            interrupt_flag.store(false, atomic::Ordering::SeqCst);
        })
        .expect("Error setting Ctrl-C handler");

        while running.load(atomic::Ordering::SeqCst)
            && start_time.elapsed().as_secs() < max_run_time as u64
        {
            match self.rx_solution_report.recv_timeout(MONITOR_INTERVAL) {
                Ok(message) => {
                    self.handle_message(message, &mut ui);
                    while let Ok(message) = self.rx_solution_report.try_recv() {
                        self.handle_message(message, &mut ui);
                    }
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => break,
            }
            if Instant::now() >= next_ui_refresh {
                ui.tick();
                next_ui_refresh = Instant::now() + UI_REFRESH_INTERVAL;
            }

            if self.material_limit.unwrap_or(u64::MAX) == self.instance.smallest_sheet_value()
                || gdrr_thread_handlers
                    .iter()
                    .all(|handler| handler.is_finished())
            {
                break;
            }
        }

        for tx_sync in &self.tx_syncs {
            let _ = tx_sync.send(SyncMessage::Terminate);
        }

        let mut total_iterations = 0;
        let mut total_accepted = 0;
        let mut total_improved = 0;
        let mut worker_elapsed = Duration::ZERO;
        for handler in gdrr_thread_handlers {
            let stats = handler.join().expect("Error joining GDRR thread");
            total_iterations += stats.n_iterations;
            total_accepted += stats.n_accepted;
            total_improved += stats.n_improved;
            worker_elapsed = worker_elapsed.max(stats.elapsed);
        }

        let elapsed = start_time.elapsed();
        let iterations_per_second =
            total_iterations as f64 / worker_elapsed.as_secs_f64().max(f64::EPSILON);
        let result = self
            .best_complete_solution
            .as_ref()
            .map(|solution| format!("complete {}", util::solution_stats_string(solution)))
            .or_else(|| {
                self.best_incomplete_solution
                    .as_ref()
                    .map(|solution| format!("incomplete {}", util::solution_stats_string(solution)))
            })
            .unwrap_or_else(|| "no solution".to_string());
        ui.finish(format!(
            "Finished\n  Throughput:   {iterations_per_second:.0} iter/s\n  Iterations:   {total_iterations}\n  Accepted:     {total_accepted}\n  Improvements: {total_improved}\n  Result:       {result}"
        ));
        elapsed
    }

    fn handle_message(&mut self, message: SolutionReportMessage, ui: &mut RunUi) {
        match message {
            SolutionReportMessage::NewCompleteSolution(thread_name, solution) => {
                self.report_new_complete_solution(thread_name, solution, ui);
            }
            SolutionReportMessage::NewIncompleteStats(thread_name, stats) => {
                self.report_new_incomplete_cost(thread_name, stats, ui);
            }
            SolutionReportMessage::NewIncompleteSolution(thread_name, solution) => {
                self.report_new_incomplete_solution(thread_name, solution, ui);
            }
        }
    }

    fn report_new_complete_solution(
        &mut self,
        thread_name: String,
        solution: SendableSolution,
        ui: &mut RunUi,
    ) {
        let message = format!("complete {}", util::solution_stats_string(&solution));
        let leading = solution.cost().material_cost < self.material_limit.unwrap_or(u64::MAX)
            && self
                .best_complete_solution
                .as_ref()
                .is_none_or(|best| solution.cost().material_cost < best.cost().material_cost);
        if leading {
            let material_cost = solution.cost().material_cost;
            ui.log_best_feasible(&thread_name, util::solution_stats_string(&solution));
            self.best_incomplete_cost = None;
            self.best_incomplete_solution = None;
            self.material_limit = Some(material_cost);
            self.best_complete_solution = Some(solution);

            for tx_sync in &self.tx_syncs {
                let _ = tx_sync.send(SyncMessage::SyncMatLimit(material_cost));
            }
        }
        ui.update(&thread_name, message, leading);
    }

    fn report_new_incomplete_solution(
        &mut self,
        thread_name: String,
        solution: SendableSolution,
        ui: &mut RunUi,
    ) {
        let message = format!("incomplete {}", util::solution_stats_string(&solution));
        let leading = self.best_complete_solution.is_none()
            && self.best_incomplete_solution.as_ref().is_none_or(|best| {
                (self.cost_comparator)(solution.cost(), best.cost()) == Ordering::Less
            });
        if leading {
            self.best_incomplete_solution = Some(solution);
        }
        ui.update(&thread_name, message, leading);
    }

    fn report_new_incomplete_cost(
        &mut self,
        thread_name: String,
        stats: SolutionStats,
        ui: &mut RunUi,
    ) {
        let message = format!("incomplete {}", util::compact_stats_string(&stats));
        let leading = stats.cost.material_cost < self.material_limit.unwrap_or(u64::MAX)
            && self
                .best_incomplete_cost
                .as_ref()
                .is_none_or(|best| (self.cost_comparator)(&stats.cost, best) == Ordering::Less);
        if leading {
            self.best_incomplete_cost = Some(stats.cost.clone());
        }
        ui.update(&thread_name, message, leading);
    }

    pub fn best_complete_solution(&self) -> Option<&SendableSolution> {
        self.best_complete_solution.as_ref()
    }

    pub fn best_incomplete_solution(&self) -> Option<&SendableSolution> {
        self.best_incomplete_solution.as_ref()
    }
}
