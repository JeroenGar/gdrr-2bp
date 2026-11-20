/// Values a leftover (empty) node of the given area as `area^power`.
///
/// Computed in `f64`: with real-world dimensions (e.g. mm) a single sheet already values at
/// ~1e13 for `power = 2`, and the sum over all layouts needs enough precision to still
/// distinguish small leftover changes. For integral `power = 2` the result is exact as long as
/// the total stays below 2^53.
pub fn valuate(area: u64, power: f64) -> f64 {
    let area = area as f64;
    let value = if power == 2.0 {
        area * area
    } else {
        area.powf(power)
    };
    debug_assert!(value.is_finite(), "leftover value of area {area} with power {power} is not finite");
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_two_is_exact_for_mm_sheets() {
        let sheet_area = 2440 * 1220;
        assert_eq!(valuate(sheet_area, 2.0), (sheet_area as f64).powi(2));
        assert_eq!(valuate(sheet_area, 2.0) as u128, (sheet_area as u128).pow(2));
    }

    /// Regression test for https://github.com/JeroenGar/gdrr-2bp/issues/11
    #[test]
    fn small_leftover_changes_remain_visible_in_sum_over_many_sheets() {
        let sheet_value = valuate(2440 * 1220, 2.0);
        let total: f64 = (0..40).map(|_| sheet_value).sum();
        for side in [100, 50, 30, 1] {
            let delta = valuate(side * side, 2.0);
            assert_ne!(total + delta, total, "a {side}x{side} leftover change was rounded away");
        }
    }

    #[test]
    fn non_integer_power() {
        assert!((valuate(100, 1.5) - 1000.0).abs() < 1e-9);
        assert_eq!(valuate(0, 1.5), 0.0);
    }
}
