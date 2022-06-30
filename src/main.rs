use std::{env, thread};
use std::cmp::Ordering;
use std::fs::File;
use std::io::BufReader;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::mpsc::channel;
use std::time::Instant;

use once_cell::sync::Lazy;

use crate::core::{entities::parttype::PartType, leftover_valuator, orientation::Orientation};
use crate::core::cost::Cost;
use crate::core::entities::sheettype::SheetType;
use crate::core::rotation::Rotation;
use crate::io::json_instance::JsonInstance;
use crate::io::parser;
use crate::optimization::config::Config;
use crate::optimization::gdrr::GDRR;
use crate::optimization::instance::Instance;
use crate::optimization::sol_collectors::global_sol_collector::GlobalSolCollector;
use crate::optimization::sol_collectors::local_sol_collector::LocalSolCollector;
use crate::util::macros::timed_println;

pub mod util;
pub mod io;
pub mod optimization;
pub mod core;

static EPOCH: Lazy<Instant> = Lazy::new(Instant::now);
const COST_COMPARATOR: fn(&Cost, &Cost) -> Ordering = |a: &Cost, b: &Cost| {
    match a.part_area_excluded.cmp(&b.part_area_excluded) {
        Ordering::Equal => a.leftover_value.partial_cmp(&b.leftover_value).unwrap().reverse(),
        other => other
    }
};
const DETERMINISTIC_MODE: bool = false; //fixes seed

fn main() {
    let args: Vec<String> = env::args().collect();
    let input = File::open(args.get(1).expect("first cmd argument needs to be path to input file")).expect("input file could not be opened");
    let config_file = File::open(args.get(2).expect("second cmd argument needs to be path to config")).expect("config file could not be opened");

    let json_instance: JsonInstance = serde_json::from_reader(BufReader::new(input)).unwrap();
    let config: Config = serde_json::from_reader(BufReader::new(config_file)).unwrap();

    {
        let mut leftover_valuator_write_lock = leftover_valuator::LEFTOVER_VALUATION_POWER.write().unwrap();
        *leftover_valuator_write_lock = config.leftover_valuation_power;
    }

    let instance = parser::generate_instance(&json_instance, &config);
    timed_println!("Starting optimization of {} parts of {} different types", instance.total_part_qty(), instance.parts().len());

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
            let local_sol_collector = LocalSolCollector::new(rx_sync, tx_solution_report_thread);
            let mut gdrr = GDRR::new(&instance_thread, &config_thread, local_sol_collector);
            gdrr.lahc();
        });
        gdrr_thread_handlers.push(handle.expect("could not spawn thread"));
    }

    let mut global_sol_collector = GlobalSolCollector::new(instance, config, tx_syncs, rx_solution_report);

    global_sol_collector.monitor(gdrr_thread_handlers);

    timed_println!("Goodbye!");
}
