use crate::core::cost::Cost;
use std::cmp::Ordering;

pub mod core;
pub mod io;
pub mod optimization;
pub mod util;

pub const COST_COMPARATOR: fn(&Cost, &Cost) -> Ordering =
    |a: &Cost, b: &Cost| match a.part_area_excluded.cmp(&b.part_area_excluded) {
        Ordering::Equal => a
            .leftover_value
            .partial_cmp(&b.leftover_value)
            .unwrap()
            .reverse(),
        other => other,
    };
