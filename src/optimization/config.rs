use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub avg_nodes_removed: usize,
    pub blink_rate: f32,
    pub max_run_time: Option<usize>,
    #[serde(rename = "maxRRIterations")]
    pub max_rr_iterations: Option<usize>,
    pub leftover_valuation_power: f32,
    pub history_length: usize,
    pub rotation_allowed: bool,
    pub n_threads: usize,
}

impl Config {}