use std::hash::{Hash, Hasher};
use crate::core::rotation::Rotation;

use crate::core::size::Size;

#[derive(Debug)]
pub struct PartType {
    id: usize,
    width: u64,
    height: u64,
    fixed_rotation: Option<Rotation>,
    size: Size,
    rotated_size: Size,
}

impl PartType {
    pub fn new(id: usize, width: u64, height: u64, fixed_rotation: Option<Rotation>) -> PartType {
        PartType {
            id,
            width,
            height,
            fixed_rotation,
            size: Size::new(width, height),
            rotated_size: Size::new(height, width),
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

    pub fn fixed_rotation(&self) -> &Option<Rotation> {
        &self.fixed_rotation
    }

    pub fn size(&self) -> &Size {
        &self.size
    }

    pub fn rotated_size(&self) -> &Size {
        &self.rotated_size
    }

    pub fn area(&self) -> u64 {
        self.size.area()
    }
}

impl Hash for PartType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for PartType {
    fn eq(&self, other: &PartType) -> bool {
        self.id == other.id
    }
}

impl Eq for PartType {}