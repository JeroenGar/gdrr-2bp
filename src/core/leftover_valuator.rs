use std::sync::RwLock;

use lazy_static::lazy_static;


lazy_static! {
    pub static ref LEFTOVER_VALUATION_POWER: RwLock<f32> = RwLock::new(2.0);
}

pub fn valuate(area: u64) -> f32 {
    f32::powf(area as f32, *LEFTOVER_VALUATION_POWER.read().unwrap())
}