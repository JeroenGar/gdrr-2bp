use std::cmp::Ordering;
use std::rc::{Rc, Weak};

use rand::Rng;
use rand::rngs::{SmallRng};

const DEFAULT_N_SAMPLES: usize = 3;
const DEFAULT_CHANCE_ARRAY: [f64; DEFAULT_N_SAMPLES] = [0.625, 0.875, 1.0];

//TODO: add explanation of how this works

pub struct BiasedSampler<T, V, const N: usize> where V: PartialOrd {
    entries: Vec<(T,V)>,
    bias_mode: BiasMode,
    chance_vec: [f64;N],
}

impl<T,V> BiasedSampler<T,V,DEFAULT_N_SAMPLES> where V: PartialOrd{
    pub fn new_default(entries: Vec<(T,V)>, bias_mode : BiasMode) -> BiasedSampler<T,V,DEFAULT_N_SAMPLES> {
        BiasedSampler {
            entries,
            bias_mode,
            chance_vec: DEFAULT_CHANCE_ARRAY,
        }
    }
}

impl<T,V,const N: usize> BiasedSampler<T, V, N> where V: PartialOrd {
    pub fn new(entries: Vec<(T,V)>, mode: BiasMode, chance_vec: [f64;N]) -> BiasedSampler<T,V,N> {
        BiasedSampler {
            entries,
            bias_mode: mode,
            chance_vec
        }
    }

    pub fn sample(&self, random: &mut SmallRng) -> Option<&T> {
        //TODO: ensure the comparator is correct

        if self.entries.is_empty() {
            return None;
        }

        //Select N random entries
        let mut samples: Vec<&(T,V)> = Vec::with_capacity(N);
        for _ in 0..N {
            samples.push(&self.entries[random.gen_range(0..self.entries.len())]);
        }

        //Sort the entries based on their value (ascending or descending, depending on the mode)
        samples.sort_by(|a, b| match self.bias_mode {
            BiasMode::Low => a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal),
            BiasMode::High => b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal),
        });

        let random_f64 = random.gen::<f64>();
        for i in 0..N {
            if random_f64 <= self.chance_vec[i] {
                return Some(&samples[i].0);
            }
        }
        return Some(&samples[N - 1].0);
    }


    pub fn entries(&self) -> &Vec<(T,V)> {
        &self.entries
    }
    pub fn chance_vec(&self) -> &[f64;N] {
        &self.chance_vec
    }
}

pub enum BiasMode {
    Low,
    High
}