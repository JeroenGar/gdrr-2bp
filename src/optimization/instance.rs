use std::iter::Map;
use crate::core::entities::sheettype::SheetType;
use crate::core::entities::parttype::PartType;

#[derive(Debug)]
pub struct Instance {
    parts : Vec<(PartType,usize)>,
    sheets: Vec<(SheetType,usize)>,
}

impl Instance {
    pub fn new(mut parts: Vec<(PartType, usize)>, mut sheets: Vec<(SheetType,usize)>) -> Self {
        //Assign IDs to parttypes and sheettypes
        parts.iter_mut().enumerate().for_each(|(i, (parttype, qty))| {
            parttype.set_id(i);
        });
        sheets.iter_mut().enumerate().for_each(|(i, (sheettype, qty))| {
            sheettype.set_id(i);
        });

        Self {
            parts,
            sheets
        }
    }

    pub fn parts(&self) -> &Vec<(PartType, usize)> {
        &self.parts
    }

    pub fn sheets(&self) -> &Vec<(SheetType, usize)> {
        &self.sheets
    }
    pub fn get_parttype(&self, index : usize) -> Option<&PartType>{
        match self.parts.get(index){
            Some((parttype, _)) => Some(parttype),
            None => None
        }
    }
    pub fn get_parttype_qty(&self, index : usize) -> Option<usize>{
        match self.parts.get(index){
            Some((_, qty)) => Some(*qty),
            None => None
        }
    }
    pub fn get_sheettype(&self, index : usize) -> Option<&SheetType>{
        match self.sheets.get(index){
            Some((sheettype, _)) => Some(sheettype),
            None => None
        }
    }
    pub fn get_sheettype_qty(&self, index : usize) -> Option<usize>{
        match self.sheets.get(index){
            Some((_, qty)) => Some(*qty),
            None => None
        }
    }
}