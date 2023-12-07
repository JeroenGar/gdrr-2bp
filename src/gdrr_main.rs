use std::{env, thread};
use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::channel;
use std::time::Instant;
use once_cell::sync::Lazy;

use gdrr_2bp::io::html_export::generate_solution;
use gdrr_2bp::io::json_format::JsonInstance;
use gdrr_2bp::io::parser;
use gdrr_2bp::optimization::config::Config;
use gdrr_2bp::optimization::gdrr::GDRR;
use gdrr_2bp::optimization::sol_collectors::global_sol_collector::GlobalSolCollector;
use gdrr_2bp::optimization::sol_collectors::local_sol_collector::LocalSolCollector;
use gdrr_2bp::timed_println;
use gdrr_2bp::COST_COMPARATOR;
use mimalloc::MiMalloc;

pub static EPOCH: Lazy<Instant> = Lazy::new(Instant::now);

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() {
    let args: Vec<String> = env::args().collect();
    let input_file_path = PathBuf::from(args.get(1).expect("First cmd argument needs to be path to input file"));
    let config_file_path = PathBuf::from(args.get(2).expect("Second cmd argument needs to be path to config file"));
    let json_solution_path = match args.len() > 3 {
        true => Some(PathBuf::from(args.get(3).unwrap())),
        false => {
            timed_println!("No JSON solution file path defined, not writing JSON file");
            None
        }
    };
    let html_solution_path = match args.len() > 4 {
        true => Some(PathBuf::from(args.get(4).unwrap())),
        false => {
            timed_println!("No HTML solution file path defined, not writing HTML file");
            None
        }
    };

    let input_file = File::open(&input_file_path).expect("input file could not be opened");
    let config_file = File::open(&config_file_path).expect("config file could not be opened");

    let mut json_instance: JsonInstance = serde_json::from_reader(BufReader::new(&input_file)).unwrap();
    let config: Config = serde_json::from_reader(BufReader::new(&config_file)).unwrap();

    timed_println!("Config file loaded: {}", serde_json::to_string(&config).unwrap());

    let instance = parser::generate_instance(&mut json_instance, &config);
    timed_println!("Starting optimization of {} parts of {} different types for {} seconds", instance.total_part_qty(), instance.parts().len(), config.max_run_time.unwrap_or(usize::MAX));
    timed_println!("Press Ctrl+C to terminate manually");

    let instance = Arc::new(instance);
    let config = Arc::new(config);

    let mut gdrr_thread_handlers = Vec::new();


    let mut tx_syncs = Vec::new();
    let (tx_solution_report, rx_solution_report) = channel();

    for i in 0..config.n_threads {
        let instance_thread = instance.clone();
        let config_thread = config.clone();
        let thread_name = format!("T{}", i);
        let (tx_sync, rx_sync) = channel();
        let tx_solution_report_thread = tx_solution_report.clone();
        tx_syncs.push(tx_sync);


        let handle = thread::Builder::new().name(thread_name).spawn(move || {
            let local_sol_collector = LocalSolCollector::new(instance_thread.clone(), rx_sync, tx_solution_report_thread, COST_COMPARATOR);
            let mut gdrr = GDRR::new(&instance_thread, &config_thread, local_sol_collector);
            gdrr.lahc();
        });
        gdrr_thread_handlers.push(handle.expect("could not spawn thread"));
    }

    let mut global_sol_collector = GlobalSolCollector::new(instance, config, tx_syncs, rx_solution_report, COST_COMPARATOR);

    global_sol_collector.monitor(gdrr_thread_handlers);

    let json_solution = match (global_sol_collector.best_complete_solution().as_ref(), global_sol_collector.best_incomplete_solution().as_ref()) {
        (Some(best_complete_solution), _) => {
            Some(parser::generate_json_solution(&json_instance, best_complete_solution, &config_file_path))
        }
        (None, Some(best_incomplete_solution)) => {
            Some(parser::generate_json_solution(&json_instance, best_incomplete_solution, &config_file_path))
        }
        (None, None) => {
            None
        }
    };

    if json_solution.is_some() {
        if let Some(json_solution_path) = json_solution_path {
            let mut json_file = File::create(&json_solution_path).expect("JSON solution file could not be created");
            serde_json::to_writer_pretty(&mut json_file, json_solution.as_ref().unwrap()).expect("could not write JSON solution");
            timed_println!("JSON solution written to {}", json_solution_path.display());
        }
        if let Some(html_solution_path) = html_solution_path {
            let mut html_file = File::create(&html_solution_path).expect("HTML solution file could not be created");
            write!(html_file, "{}", &generate_solution(json_solution.as_ref().unwrap())).expect("could not write HTML solution");
            timed_println!("HTML solution written to {}", html_solution_path.display());
        }
    } else {
        timed_println!("No solution available");
    }
}
