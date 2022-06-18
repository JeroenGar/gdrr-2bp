#[derive(Debug, Clone, Copy)]
pub struct Size{
    width: u64,
    height: u64,
    area: u64
}

impl Size{
    pub fn new(width: u64, height: u64) -> Size{
        Size{
            width: width,
            height: height,
            area: width * height
        }
    }


}