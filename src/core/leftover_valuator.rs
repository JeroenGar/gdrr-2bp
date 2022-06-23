use crate::optimization::config::Config;

pub fn valuate(area : u64, config : &Config) -> f32 {
    //powf is approximated by the micromath crate
    f32::powf(area as f32, config.leftover_valuation_power())
}