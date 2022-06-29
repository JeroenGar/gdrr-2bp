use std::fs::File;
use std::io::{BufReader};
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

fn main() {
    let test_file = File::open("assets/1.json").unwrap();
    let config_file = File::open("assets/config.json").unwrap();

    let json_instance : JsonInstance = serde_json::from_reader(BufReader::new(test_file)).unwrap();
    let config : Config = serde_json::from_reader(BufReader::new(config_file)).unwrap();

    {
        let mut leftover_valuator_write_lock = leftover_valuator::LEFTOVER_VALUATION_POWER.write().unwrap();
        *leftover_valuator_write_lock = config.leftover_valuation_power;
    }

    let instance = parser::generate_instance(&json_instance, &config);
    timed_println!("Starting optimization of {} parts of {} different types", instance.total_part_qty(), instance.parts().len());
    let mut gdrr = GDRR::new(&instance, &config);

    gdrr.lahc();

    timed_println!("Done");

}
