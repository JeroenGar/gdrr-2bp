pub struct Config {
    avg_nodes_removed: usize,
    blink_rate: f32,
    max_run_time_ms: usize,
    max_rr_iterations : usize,
    leftover_valuation_power: f32,
    history_length: usize,
}

impl Config {
    pub fn avg_nodes_removed(&self) -> usize {
        self.avg_nodes_removed
    }
    pub fn blink_rate(&self) -> f32 {
        self.blink_rate
    }
    pub fn leftover_valuation_power(&self) -> f32 {
        self.leftover_valuation_power
    }
    pub fn history_length(&self) -> usize {
        self.history_length
    }
    pub fn max_run_time_ms(&self) -> usize {
        self.max_run_time_ms
    }
    pub fn max_rr_iterations(&self) -> usize {
        self.max_rr_iterations
    }
}