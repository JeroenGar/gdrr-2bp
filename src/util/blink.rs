use std::ops::Range;
use rand::Rng;
use rand::rngs::SmallRng;

/// Christiaens, J., & Vanden Berghe, G. (2020). Slack Induction by String Removals for Vehicle Routing Problems. (https://lirias.kuleuven.be/retrieve/510989)
///
/// It basically selects the lowest value from a list of entries,
/// but with a certain chance to skip over ('blink') values.
///
/// Example:
/// A blink chance of 1% means that 99% of the time, the lowest value will be selected,
/// 0.99% of the time, the second lowest value will be selected, and so on.

pub fn select_lowest_entry(entries: &Vec<usize>, blink_chance: f32, rand: &mut SmallRng) -> usize {
    let mut lowest_value = usize::MAX;
    let mut selected_index = 0;

    for (i, entry) in entries.iter().enumerate() {
        if *entry < lowest_value && rand.random::<f32>() > blink_chance {
            lowest_value = *entry;
            selected_index = i;
        }
    }
    return selected_index;
}

pub fn select_lowest_in_range(range: Range<usize>, blink_chance: f32, rand: &mut SmallRng) -> usize {
    let range_end = range.end;
    for i in range {
        if rand.random::<f32>() > blink_chance {
            return i;
        }
    }
    return range_end - 1;
}

