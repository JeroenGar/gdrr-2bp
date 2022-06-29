use std::ops::Range;

use rand::Rng;
use rand::rngs::{StdRng, ThreadRng};

pub fn select_lowest_entry(entries: &Vec<usize>, blink_chance: f32, rand: &mut StdRng) -> usize {
    let mut lowest_value = usize::MAX;
    let mut selected_index = 0;

    for (i, entry) in entries.iter().enumerate() {
        if *entry < lowest_value && rand.gen::<f32>() > blink_chance {
            lowest_value = *entry;
            selected_index = i;
        }
    }
    return selected_index;
}

pub fn select_lowest_in_range(range: Range<usize>, blink_chance: f32, rand: &mut StdRng) -> usize {
    let range_end = range.end;
    for i in range {
        if rand.gen::<f32>() > blink_chance {
            return i;
        }
    }
    return range_end - 1;
}

