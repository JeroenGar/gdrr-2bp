use std::hash::{Hash, Hasher};
use std::sync::atomic::AtomicUsize;

use crate::core::orientation::Orientation;
use crate::core::size::Size;
use crate::Rotation;

#[derive(Debug)]
pub struct PartType {
    id: Option<usize>,
    width: u64,
    height: u64,
    fixed_rotation: Option<Rotation>,
    size: Size,
    rotated_size: Size,
}

impl PartType {
    pub fn new(width: u64, height: u64, fixed_rotation: Option<Rotation>) -> PartType {
        PartType {
            id: None,
            width,
            height,
            fixed_rotation,
            size: Size::new(width, height),
            rotated_size: Size::new(height, width),
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
        if self.id.is_some() {
            self.id.hash(state);
        } else {
            self.width.hash(state);
            self.height.hash(state);
            self.fixed_rotation.hash(state);
        }
    }
}

impl PartialEq for PartType {
    fn eq(&self, other: &PartType) -> bool {
        return if self.id.is_some() && other.id.is_some() {
            self.id == other.id
        } else {
            self.id == other.id &&
                self.width == other.width &&
                self.height == other.height &&
                self.fixed_rotation == other.fixed_rotation
        };
    }
}

impl Eq for PartType {}