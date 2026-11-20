use serde::{Deserialize, Serialize};

/// Contains all the configurable parameters of the algorithm

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub avg_nodes_removed: usize,
    pub blink_rate: f32,
    pub max_run_time: Option<usize>,
    #[serde(rename = "maxRRIterations")]
    pub max_rr_iterations: Option<usize>,
    pub random_seed: Option<u64>,
    pub leftover_valuation_power: f64,
    pub history_length: usize,
    pub rotation_allowed: bool,
    pub n_threads: usize,
    pub sheet_valuation_mode : SheetValuationMode,
    pub max_stages: Option<u8>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SheetValuationMode {
    Area,
    Cost
}

impl Config {
    /// Rejects parameter values that would make the algorithm panic.
    pub fn validate(&self) -> Result<(), String> {
        if !(self.leftover_valuation_power.is_finite() && self.leftover_valuation_power > 0.0) {
            return Err(format!(
                "leftoverValuationPower must be a finite number > 0, got {}",
                self.leftover_valuation_power
            ));
        }
        if self.avg_nodes_removed < 3 {
            return Err(format!("avgNodesRemoved must be at least 3, got {}", self.avg_nodes_removed));
        }
        if self.history_length == 0 {
            return Err("historyLength must be at least 1".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> Config {
        serde_json::from_str(include_str!("../../examples/config.json")).unwrap()
    }

    #[test]
    fn example_config_is_valid() {
        assert_eq!(config().validate(), Ok(()));
    }

    #[test]
    fn invalid_values_are_rejected() {
        let invalid: [fn(&mut Config); 5] = [
            |c| c.leftover_valuation_power = -1.0,
            |c| c.leftover_valuation_power = 0.0,
            |c| c.leftover_valuation_power = f64::NAN,
            |c| c.avg_nodes_removed = 2,
            |c| c.history_length = 0,
        ];
        for modify in invalid {
            let mut c = config();
            modify(&mut c);
            assert!(c.validate().is_err());
        }
    }
}
