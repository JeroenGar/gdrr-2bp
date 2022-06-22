use std::ops::Range;
use rand::Rng;
use rand::rngs::ThreadRng;

pub fn select_lowest(range : Range<usize>, blink_chance : f64, rand : &mut ThreadRng) -> usize {
    let range_end = range.end;
    for i in range {
        if rand.gen::<f64>() > blink_chance {
            return i;
        }
    }
    return range_end;
}