use std::cmp::Ordering;
use std::time::Instant;
use once_cell::sync::Lazy;
use crate::core::cost::Cost;

pub mod util;
pub mod io;
pub mod optimization;
pub mod core;


pub static EPOCH: Lazy<Instant> = Lazy::new(Instant::now);
/// Orders costs from best to worst: least excluded part area first, then highest leftover value.
///
/// Uses `total_cmp` so a non-finite leftover value can never abort a run (release builds use
/// `panic = "abort"`, which would discard the best solution found so far).
pub const COST_COMPARATOR: fn(&Cost, &Cost) -> Ordering = |a: &Cost, b: &Cost| {
    match a.part_area_excluded.cmp(&b.part_area_excluded) {
        Ordering::Equal => a.leftover_value.total_cmp(&b.leftover_value).reverse(),
        other => other
    }
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cost_comparator_orders_by_excluded_area_then_leftover_value() {
        let cost = |excluded, leftover| Cost::new(0, leftover, excluded, 0);
        assert_eq!(COST_COMPARATOR(&cost(0, 1.0), &cost(1, 1e30)), Ordering::Less);
        assert_eq!(COST_COMPARATOR(&cost(1, 2.0), &cost(1, 1.0)), Ordering::Less);
        assert_eq!(COST_COMPARATOR(&cost(1, 1.0), &cost(1, 1.0)), Ordering::Equal);
    }

    #[test]
    fn cost_comparator_does_not_panic_on_nan() {
        let nan = Cost::new(0, f64::NAN, 0, 0);
        let finite = Cost::new(0, 1.0, 0, 0);
        let _ = COST_COMPARATOR(&nan, &finite);
    }
}
