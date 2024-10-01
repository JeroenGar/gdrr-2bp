use std::hash::{Hash, Hasher};
use crate::core::orientation::Orientation;

#[derive(Debug, PartialEq, Eq)]
pub struct SheetType {
    id: usize,
    width: u64,
    height: u64,
    value: u64,
    fixed_first_cut_orientation: Option<Orientation>,
    max_stages: u8,
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

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn width(&self) -> u64 {
        self.width
    }

    pub fn height(&self) -> u64 {
        self.height
    }

    pub fn value(&self) -> u64 {
        self.value
    }

    pub fn area(&self) -> u64 {
        self.width * self.height
    }

    pub fn fixed_first_cut_orientation(&self) -> Option<Orientation> {
        self.fixed_first_cut_orientation
    }

    pub fn max_stages(&self) -> u8 {
        self.max_stages
    }
}

impl Hash for SheetType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

