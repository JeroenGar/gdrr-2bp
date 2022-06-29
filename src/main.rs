use std::{env, thread};
use std::fs::File;
use std::io::{BufReader};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;
use once_cell::sync::Lazy;

use crate::core::{entities::parttype::PartType, leftover_valuator, orientation::Orientation};
use crate::core::entities::sheettype::SheetType;
use crate::core::rotation::Rotation;
use crate::io::json_instance::JsonInstance;
use crate::io::parser;
use crate::optimization::config::Config;
use crate::optimization::gdrr::GDRR;
use crate::optimization::instance::Instance;
use crate::util::macros::{timed_println};

pub mod util;
pub mod io;
pub mod optimization;
pub mod core;

static EPOCH : Lazy<Instant> = Lazy::new(Instant::now);
const DETERMINISTIC_MODE : bool = false; //fixes seed

fn main() {
    let args: Vec<String> = env::args().collect();
    let input = File::open(args.get(1).expect("first cmd argument needs to be path to input file")).expect("input file could not be opened");
    let config_file = File::open(args.get(2).expect("second cmd argument needs to be path to config")).expect("config file could not be opened");

    let json_instance : JsonInstance = serde_json::from_reader(BufReader::new(input)).unwrap();
    let config : Config = serde_json::from_reader(BufReader::new(config_file)).unwrap();

    {
        let mut leftover_valuator_write_lock = leftover_valuator::LEFTOVER_VALUATION_POWER.write().unwrap();
        *leftover_valuator_write_lock = config.leftover_valuation_power;
    }

    let instance = parser::generate_instance(&json_instance, &config);
    timed_println!("Starting optimization of {} parts of {} different types", instance.total_part_qty(), instance.parts().len());

    let instance = Arc::new(instance);
    let config = Arc::new(config);

    let mut handles = Vec::new();

    for i in 0..config.n_threads {
        let instance_thread = instance.clone();
        let config_thread = config.clone();
        let thread_name = format!("T{}", i);
        let handle = thread::Builder::new().name(thread_name).spawn(move || {
            let mut gdrr = GDRR::new(&instance_thread, &config_thread);
            gdrr.lahc();
        });
        handles.push(handle);
    }

    for handle in handles {
        match handle {
            Ok(handle) => handle.join().unwrap(),
            Err(e) => panic!("Error: {}", e)
        }
    }

    timed_println!("Done");
}
