#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

impl Orientation {
    pub fn rotate(&self) -> Orientation {
        match self {
            Orientation::Horizontal => Orientation::Vertical,
            Orientation::Vertical => Orientation::Horizontal
        }
    }
}