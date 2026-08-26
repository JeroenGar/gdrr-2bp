pub fn valuate(area: u64, power: f32) -> f32 {
    let area = area as f32;
    if power == 2.0 {
        area * area
    } else {
        area.powf(power)
    }
}
