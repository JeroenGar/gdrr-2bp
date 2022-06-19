use crate::core::orientation::Orientation;
use crate::core::size::Size;

pub struct PartType{
    id: usize,
    width: u64,
    height: u64,
    fixed_orientation: Option<Orientation>,
    size: Size,
    rotated_size: Size,
}

static mut ID: usize = 0;

impl PartType{
    pub fn new (width: u64, height: u64, fixed_orientation: Option<Orientation>) -> PartType{
        PartType{
            id: unsafe { ID += 1; ID },
            width: width,
            height: height,
            fixed_orientation: fixed_orientation,
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

    pub fn size(&self) -> Size{
        self.size
    }

    pub fn rotated_size(&self) -> Size{
        self.rotated_size
    }

    pub fn fixed_orientation(&self) -> Option<Orientation> {
        self.fixed_orientation
    }
}