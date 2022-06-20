use std::collections::{LinkedList};
use indexmap::IndexMap;
use crate::{Instance, PartType, SheetType};
use crate::core::entities::layout::Layout;

pub struct Problem<'a> {
    instance : &'a Instance,
    parttype_qtys : IndexMap<&'a PartType, usize>,
    sheettype_qtys : IndexMap<&'a SheetType, usize>,
    layouts : LinkedList<Layout>,
    empty_layouts : Vec<Layout>,
    random : rand::rngs::ThreadRng,

}