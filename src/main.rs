use crate::core::{entities::parttype::PartType, orientation::Orientation};
use crate::core::rotation::Rotation;

pub mod util;
pub mod io;
pub mod optimization;
pub mod core;

fn main() {
    println!("Hello, world!");

    let parttype = PartType::new(10, 10, Some(Rotation::Default));
    println!("{:?}", parttype.id());
}
