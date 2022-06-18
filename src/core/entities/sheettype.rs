pub struct SheetType{
    id: usize,
    width: u64,
    height: u64,
    value : u64,
}

static mut ID: usize = 0;

impl SheetType{
    pub fn new (width: u64, height: u64, value: u64) -> SheetType{
        SheetType{
            id: unsafe { ID += 1; ID },
            width: width,
            height: height,
            value: value,
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

