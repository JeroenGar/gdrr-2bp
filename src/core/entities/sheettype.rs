use std::hash::{Hash, Hasher};
use std::sync::atomic::AtomicUsize;
use crate::Orientation;

#[derive(Debug, PartialEq, Eq)]
pub struct SheetType {
    id: Option<usize>,
    width: u64,
    height: u64,
    value: u64,
    fixed_first_cut_orientation : Option<Orientation>
}

impl SheetType {
    pub fn new(width: u64, height: u64, value: u64, fixed_first_cut_orientation : Option<Orientation>) -> SheetType {
        SheetType {
            id: None,
            width,
            height,
            value,
            fixed_first_cut_orientation
        }
    }

    pub fn id(&self) -> usize {
        self.id.unwrap()
    }

    pub fn set_id(&mut self, id: usize) {
        self.id = Some(id);
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
}

impl Hash for SheetType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

