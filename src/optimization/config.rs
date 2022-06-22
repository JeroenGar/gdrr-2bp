pub struct Config{
    avg_nodes_removed : usize,
    blink_chance : f64
}






impl Config{
    pub fn avg_nodes_removed(&self) -> usize {
        self.avg_nodes_removed
    }


    pub fn blink_chance(&self) -> f64 {
        self.blink_chance
    }
}