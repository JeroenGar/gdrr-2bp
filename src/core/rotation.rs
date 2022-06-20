pub enum Rotation{
    Default,
    Rotated
}

impl Rotation {
    pub fn rotate(&self) -> Rotation {
        match self {
            Rotation::Default => Rotation::Rotated,
            Rotation::Rotated => Rotation::Default
        }
    }
}