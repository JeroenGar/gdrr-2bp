pub struct Config {
    avg_nodes_removed: usize,
    blink_rate: f32,
    run_time_ms: usize,
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
    pub fn run_time_ms(&self) -> usize {
        self.run_time_ms
    }
    pub fn leftover_valuation_power(&self) -> f32 {
        self.leftover_valuation_power
    }
    pub fn history_length(&self) -> usize {
        self.history_length
    }
}