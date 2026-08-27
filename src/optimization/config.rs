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
    pub leftover_valuation_power: f32,
    pub history_length: usize,
    pub rotation_allowed: bool,
    pub n_threads: usize,
    pub sheet_valuation_mode: SheetValuationMode,
    pub max_stages: Option<u8>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SheetValuationMode {
    Area,
    Cost,
}

impl Config {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.n_threads == 0 {
            return Err("nThreads must be at least 1");
        }
        if self.history_length == 0 {
            return Err("historyLength must be at least 1");
        }

        let max_avg_nodes_removed = (usize::MAX - 1) / 2 + 2;
        if !(3..=max_avg_nodes_removed).contains(&self.avg_nodes_removed) {
            return Err("avgNodesRemoved is outside its supported range");
        }
        if !(0.0..=1.0).contains(&self.blink_rate) {
            return Err("blinkRate must be between 0 and 1");
        }
        if !self.leftover_valuation_power.is_finite() {
            return Err("leftoverValuationPower must be finite");
        }

        Ok(())
    }
}
