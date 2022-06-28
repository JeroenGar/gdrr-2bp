use std::ops::Range;

use rand::Rng;
use rand::rngs::{StdRng, ThreadRng};

pub fn select_lowest(range: Range<usize>, blink_chance: f32, rand: &mut StdRng) -> usize {
    let range_end = range.end;
    for i in range {
        if rand.gen::<f32>() > blink_chance {
            return i;
        }
    }
    return range_end;
}