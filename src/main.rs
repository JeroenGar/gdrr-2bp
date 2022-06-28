use std::fs::File;
use std::io::{BufReader};
use indexmap::IndexMap;

use crate::core::{entities::parttype::PartType, leftover_valuator, orientation::Orientation};
use crate::core::entities::sheettype::SheetType;
use crate::core::rotation::Rotation;
use crate::io::json_instance::JsonInstance;
use crate::io::parser;
use crate::optimization::config::Config;
use crate::optimization::gdrr::GDRR;
use crate::optimization::instance::Instance;

pub mod util;
pub mod io;
pub mod optimization;
pub mod core;

fn main() {
    println!("Hello, world!");
    let test_file = File::open("assets/1.json").unwrap();
    let config_file = File::open("assets/config.json").unwrap();

    let json_instance : JsonInstance = serde_json::from_reader(BufReader::new(test_file)).unwrap();
    let config : Config = serde_json::from_reader(BufReader::new(config_file)).unwrap();

    {
        let mut leftover_valuator_write_lock = leftover_valuator::LEFTOVER_VALUATION_POWER.write().unwrap();
        *leftover_valuator_write_lock = config.leftover_valuation_power;
    }

    let instance = parser::generate_instance(&json_instance, &config);
    let mut gdrr = GDRR::new(&instance, &config);

    gdrr.lahc();

    println!("done");

}
