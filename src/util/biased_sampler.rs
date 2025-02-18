use rand::rngs::SmallRng;
use rand::Rng;

const DEFAULT_N_SAMPLES: usize = 3;
const DEFAULT_CHANCE_ARRAY: [f64; DEFAULT_N_SAMPLES] = [0.625, 0.875, 1.0];

/// BiasedSampler samples at random from a list of entries, but not uniformly.
/// There is a bias towards the highest or lowest values.
///
/// For example, the default configuration sampler will select 3 random entries of type T and sort them based on their value V.
/// Either in ascending or descending order, depending on the bias mode.
/// It will then return the first entry with a probability of 0.625, the second with a probability of 0.25 and the third with a probability of 0.125.
/// This allows us to sample at random, but with a bias.

pub struct BiasedSampler<T, V, const N: usize> where V: Ord {
    entries: Vec<(T, V)>,
    bias_mode: BiasMode,
    chance_vec: [f64; N],
}

impl<T, V> BiasedSampler<T, V, DEFAULT_N_SAMPLES> where V: Ord {
    pub fn new_default(entries: Vec<(T, V)>, bias_mode: BiasMode) -> BiasedSampler<T, V, DEFAULT_N_SAMPLES> {
        BiasedSampler {
            entries,
            bias_mode,
            chance_vec: DEFAULT_CHANCE_ARRAY,
        }
    }
}

impl<T, V, const N: usize> BiasedSampler<T, V, N> where V: Ord {
    pub fn new(entries: Vec<(T, V)>, mode: BiasMode, chance_vec: [f64; N]) -> BiasedSampler<T, V, N> {
        BiasedSampler {
            entries,
            bias_mode: mode,
            chance_vec,
        }
    }

    pub fn sample(&self, random: &mut SmallRng) -> Option<&T> {
        if self.entries.is_empty() {
            return None;
        }

        //Select N random entries
        let mut samples: [Option<&(T, V)>; N] = [None; N];
        for i in 0..N {
            samples[i] = Some(&self.entries[random.random_range(0..self.entries.len())]);
        }

        //Sort the entries based on their value (ascending or descending, depending on the mode)
        samples.sort_by(|a, b| {
            let (a, b) = (a.unwrap(), b.unwrap());
            match self.bias_mode {
                BiasMode::Low => a.1.cmp(&b.1),
                BiasMode::High => b.1.cmp(&a.1),
            }
        });

        let random_f64 = random.random::<f64>();
        for i in 0..N {
            if random_f64 <= self.chance_vec[i] {
                return Some(&samples[i].unwrap().0);
            }
        }
        return Some(&samples[N - 1].unwrap().0);
    }

    pub fn entries(&self) -> &Vec<(T, V)> {
        &self.entries
    }

    pub fn chance_vec(&self) -> &[f64; N] {
        &self.chance_vec
    }
}

pub enum BiasMode {
    Low,
    High,
}