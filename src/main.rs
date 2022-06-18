use crate::core::{entities::parttype::PartType, orientation::Orientation};

pub mod util;
pub mod io;
pub mod optimization;
pub mod core;

fn main() {
    println!("Hello, world!");

    let parttype = PartType::new(0, 10, 10, Orientation::Default);
    println!("{:?}", parttype);
}
