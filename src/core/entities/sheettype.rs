use std::hash::{Hash, Hasher};
use std::sync::atomic::AtomicUsize;

#[derive(Debug)]
pub struct SheetType{
    width: u64,
    height: u64,
    value : u64,
}


impl SheetType{
    pub fn new (width: u64, height: u64, value: u64) -> SheetType{
        SheetType{
            width,
            height,
            value
        }
    }

    pub fn width(&self) -> u64{
        self.width
    }

    pub fn height(&self) -> u64{
        self.height
    }

    pub fn value(&self) -> u64{
        self.value
    }
}

