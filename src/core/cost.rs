#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Cost {
    pub material_cost: u64,
    pub leftover_value: f64,
    pub part_area_included: u64,
    pub part_area_excluded: u64,
}


impl Cost {
    pub fn new(material_cost: u64, leftover_value: f64, part_area_included: u64, part_area_excluded: u64) -> Self {
        Self { material_cost, leftover_value, part_area_included, part_area_excluded }
    }

    pub fn add(&mut self, other: &Cost) {
        self.material_cost += other.material_cost;
        self.leftover_value += other.leftover_value;
        self.part_area_included += other.part_area_included;
        self.part_area_excluded += other.part_area_excluded;
    }

    pub fn subtract(&mut self, other: &Cost) {
        self.material_cost -= other.material_cost;
        self.leftover_value -= other.leftover_value;
        self.part_area_included -= other.part_area_included;
        self.part_area_excluded -= other.part_area_excluded;
    }
}