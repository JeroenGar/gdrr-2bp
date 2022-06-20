use std::hash::{Hash, Hasher};
use std::sync::atomic::AtomicUsize;
use crate::core::orientation::Orientation;
use crate::core::size::Size;
use crate::Rotation;

#[derive(Debug)]
pub struct PartType{
    id: usize,
    width: u64,
    height: u64,
    fixed_rotation: Option<Rotation>,
    size: Size,
    rotated_size: Size,
}

fn get_id() -> usize {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

impl PartType{
    pub fn new (width: u64, height: u64, fixed_rotation: Option<Rotation>) -> PartType{
        PartType{
            id: get_id(),
            width,
            height,
            fixed_rotation,
            size: Size::new(width, height),
            rotated_size: Size::new(height, width),
        }
    }

    pub fn id(&self) -> usize{
        self.id
    }

    pub fn width(&self) -> u64{
        self.width
    }

    pub fn height(&self) -> u64{
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
}

impl Hash for PartType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq<Self> for PartType {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for PartType {

}