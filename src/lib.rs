use std::cmp::Ordering;
use std::time::Instant;
use once_cell::sync::Lazy;
use crate::core::cost::Cost;

pub mod util;
pub mod io;
pub mod optimization;
pub mod core;


pub static EPOCH: Lazy<Instant> = Lazy::new(Instant::now);
pub const COST_COMPARATOR: fn(&Cost, &Cost) -> Ordering = |a: &Cost, b: &Cost| {
    match a.part_area_excluded.cmp(&b.part_area_excluded) {
        Ordering::Equal => a.leftover_value.partial_cmp(&b.leftover_value).unwrap().reverse(),
        other => other
    }
};
pub const DETERMINISTIC_MODE: bool = false; //fixes seed
