use crate::Cost;

pub struct SolutionStats {
    pub cost : Cost,
    pub usage : f64,
    pub n_sheets : usize,
}

impl SolutionStats {
    pub fn new(cost : Cost, usage : f64, n_sheets : usize) -> Self {
        Self { cost, usage, n_sheets }
    }
}