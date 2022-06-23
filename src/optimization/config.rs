pub struct Config {
    avg_nodes_removed: usize,
    blink_rate: f64,
    run_time_ms: usize,
    leftover_valuation_power: f64,
    history_length: usize,
}

impl Config {
    pub fn avg_nodes_removed(&self) -> usize {
        self.avg_nodes_removed
    }


    pub fn blink_chance(&self) -> f64 {
        self.blink_rate
    }
}