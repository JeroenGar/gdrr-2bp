use std::cmp::Ordering;
use std::rc::{Rc, Weak};

use rand::Rng;
use rand::rngs::{StdRng, ThreadRng};

const DEFAULT_N_SAMPLES: usize = 3;
const DEFAULT_CHANCE_ARRAY: [f64; 3] = [0.625, 0.875, 1.0];

pub struct BiasedSampler<T> {
    entries: Vec<Weak<T>>,
    comparator: fn(&T, &T) -> Ordering,
    n_samples: usize,
    chance_vec: Vec<f64>,
}

impl<T> BiasedSampler<T> {
    pub fn new_default(entries: Vec<Weak<T>>, comparator: fn(&T, &T) -> Ordering) -> BiasedSampler<T> {
        BiasedSampler {
            entries,
            comparator,
            n_samples: DEFAULT_N_SAMPLES,
            chance_vec: DEFAULT_CHANCE_ARRAY.to_vec(),
        }
    }

    pub fn new(entries: Vec<Weak<T>>, comparator: fn(&T, &T) -> Ordering, n_samples: usize, chance_vec: Vec<f64>) -> BiasedSampler<T> {
        BiasedSampler {
            entries,
            comparator,
            n_samples,
            chance_vec,
        }
    }

    pub fn sample(&self, random: &mut StdRng) -> Option<&Weak<T>> {
        if self.entries.is_empty() {
            return None;
        }

        let mut samples: Vec<&Weak<T>> = Vec::with_capacity(self.n_samples);
        for _ in 0..self.n_samples {
            samples.push(&self.entries.get(random.gen_range(0..self.entries.len())).unwrap());
        }
        samples.sort_by(|a, b| (self.comparator)(&a.upgrade().unwrap(), &b.upgrade().unwrap()).reverse());
        let random_f64 = random.gen::<f64>();
        for i in 0..self.chance_vec.len() {
            if random_f64 <= self.chance_vec[i] {
                return Some(samples[i]);
            }
        }
        return Some(samples[self.chance_vec.len() - 1]);
    }
}