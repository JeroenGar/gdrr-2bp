use crate::core::orientation::Orientation;
use crate::core::size::Size;
use crate::Rotation;

pub struct PartType{
    id: usize,
    width: u64,
    height: u64,
    fixed_rotation: Option<Rotation>,
    size: Size,
    rotated_size: Size,
}

static mut ID: usize = 0;

impl PartType{
    pub fn new (width: u64, height: u64, fixed_rotation: Option<Rotation>) -> PartType{
        PartType{
            id: unsafe { ID += 1; ID },
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