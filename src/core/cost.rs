use std::ops::{Add, Sub};
use std::iter::Sum;

#[derive(Debug, Clone, PartialEq)]
pub struct Cost {
    pub material_cost: u64,
    pub leftover_value: f32,
    pub part_area_excluded: u64,
    pub part_area_included: u64,
}


impl Cost {
    pub fn new(material_cost: u64, leftover_value: f32, part_area_excluded: u64, part_area_included : u64) -> Self {
        Self { material_cost, leftover_value, part_area_excluded, part_area_included }
    }

    pub fn add(&mut self, other: &Cost) {
        self.material_cost += other.material_cost;
        self.leftover_value += other.leftover_value;
        self.part_area_excluded += other.part_area_excluded;
        self.part_area_included += other.part_area_included;
    }

    pub fn subtract(&mut self, other: &Cost) {
        self.material_cost -= other.material_cost;
        self.leftover_value -= other.leftover_value;
        self.part_area_excluded -= other.part_area_excluded;
        self.part_area_included -= other.part_area_included;
    }

    pub fn part_area_fraction_included(&self) -> f64 {
        self.part_area_included as f64 / (self.part_area_excluded + self.part_area_included) as f64
    }
}

impl Add for Cost {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            material_cost : self.material_cost + rhs.material_cost,
            leftover_value : self.leftover_value + rhs.leftover_value,
            part_area_excluded : self.part_area_excluded + rhs.part_area_excluded,
            part_area_included : self.part_area_included + rhs.part_area_included,
        }
    }
}

impl Sub for Cost {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            material_cost : self.material_cost - rhs.material_cost,
            leftover_value : self.leftover_value - rhs.leftover_value,
            part_area_excluded : self.part_area_excluded - rhs.part_area_excluded,
            part_area_included : self.part_area_included - rhs.part_area_included,
        }
    }
}

impl Sum for Cost {
    fn sum<I: Iterator<Item=Self>>(iter: I) -> Self {
        iter.fold(Self::new(0, 0.0, 0, 0), |acc, cost| acc + cost)
    }
}