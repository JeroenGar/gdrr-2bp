use std::hash::{Hash, Hasher};
use std::sync::atomic::AtomicUsize;

#[derive(Debug, PartialEq, Eq)]
pub struct SheetType{
    id: usize,
    width: u64,
    height: u64,
    value : u64,
}

fn get_id() -> usize {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

impl SheetType{
    pub fn new (width: u64, height: u64, value: u64) -> SheetType{
        SheetType{
            id: get_id(),
            width,
            height,
            value
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

    pub fn value(&self) -> u64{
        self.value
    }
}

impl Hash for SheetType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

