#[derive(Debug, Clone)]
pub struct Size {
    width: u64,
    height: u64,
    area: u64,
}

impl Size {
    pub fn new(width: u64, height: u64) -> Size {
        Size {
            width,
            height,
            area: width * height,
        }
    }


    pub fn width(&self) -> u64 {
        self.width
    }
    pub fn height(&self) -> u64 {
        self.height
    }
    pub fn area(&self) -> u64 {
        self.area
    }
}