use std::hash::{Hash, Hasher};
use crate::core::orientation::Orientation;

#[derive(Debug, PartialEq, Eq)]
pub struct SheetType {
    pub id: usize,
    pub width: u64,
    pub height: u64,
    pub value: u64,
    pub fixed_first_cut_orientation: Option<Orientation>,
    pub max_stages: u8,
}

impl SheetType {
    pub fn new(id: usize, width: u64, height: u64, value: u64, fixed_first_cut_orientation: Option<Orientation>, max_stages: u8) -> SheetType {
        SheetType {
            id,
            width,
            height,
            value,
            fixed_first_cut_orientation,
            max_stages,
        }
    }

    pub fn area(&self) -> u64 {
        self.width * self.height
    }
}

impl Hash for SheetType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
