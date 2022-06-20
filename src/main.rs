use indexmap::IndexMap;
use crate::core::{entities::parttype::PartType, orientation::Orientation};
use crate::core::entities::sheettype::SheetType;
use crate::core::rotation::Rotation;
use crate::optimization::instance::Instance;

pub mod util;
pub mod io;
pub mod optimization;
pub mod core;

fn main() {
    println!("Hello, world!");

    let parttype = PartType::new(10, 10, Some(Rotation::Default));
    let sheettype = SheetType::new(100,100,100*100);

    let mut parts = IndexMap::new();
    let mut sheets = IndexMap::new();

    parts.insert(parttype, 100);
    sheets.insert(sheettype, 1);

    let instance = Instance::new(parts, sheets);

    dbg!(instance);

}
