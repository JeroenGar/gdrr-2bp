use crate::core::cost::Cost;
use once_cell::sync::Lazy;
use std::cmp::Ordering;
use std::time::Instant;

pub mod core;
pub mod io;
pub mod optimization;
pub mod util;

pub static EPOCH: Lazy<Instant> = Lazy::new(Instant::now);
pub const COST_COMPARATOR: fn(&Cost, &Cost) -> Ordering =
    |a: &Cost, b: &Cost| match a.part_area_excluded.cmp(&b.part_area_excluded) {
        Ordering::Equal => a
            .leftover_value
            .partial_cmp(&b.leftover_value)
            .unwrap()
            .reverse(),
        other => other,
    };
