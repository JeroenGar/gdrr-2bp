use crate::core::orientation::Orientation;
use crate::core::size::Size;

pub struct PartType{
    id: usize,
    width: u64,
    height: u64,
    fixed_orientation: Orientation,
    size: Size,
    rotated_size: Size,
}

impl PartType{
    pub fn new (id: usize, width: u64, height: u64, fixed_orientation: Orientation) -> PartType{
        PartType{
            id: id,
            width: width,
            height: height,
            fixed_orientation: fixed_orientation,
            size: Size::new(width, height),
            rotated_size: Size::new(height, width),
        }
    }
}