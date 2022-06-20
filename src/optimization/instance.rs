use std::iter::Map;
use indexmap::IndexMap;
use crate::core::entities::sheettype::SheetType;
use crate::core::entities::parttype::PartType;

#[derive(Debug)]
pub struct Instance {
    parts : IndexMap<PartType, usize>,
    sheets: IndexMap<SheetType, usize>
}

impl Instance {
    pub fn new(parts: IndexMap<PartType, usize>, sheets: IndexMap<SheetType, usize>) -> Self {
        Self { parts, sheets }
    }
}