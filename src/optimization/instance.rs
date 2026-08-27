use crate::core::entities::parttype::PartType;
use crate::core::entities::sheettype::SheetType;
use crate::util::assertions;

#[derive(Debug)]
/// Instance is a immutable representation of the collection of parts and sheets that need to be optimized.
pub struct Instance {
    parts: Vec<(PartType, usize)>,
    sheets: Vec<(SheetType, usize)>,
    total_part_area: u64,
    total_part_qty: usize,
}

impl Instance {
    pub fn new(parts: Vec<(PartType, usize)>, sheets: Vec<(SheetType, usize)>) -> Self {
        // The ID's of parts and sheets must match their respective indices in the vectors.
        assert!(assertions::instance_parttypes_and_sheettypes_ids_correct(
            &parts, &sheets
        ));

        let total_part_area = parts
            .iter()
            .map(|(parttype, qty)| parttype.area() * (*qty as u64))
            .sum();
        let total_part_qty = parts.iter().map(|(_, qty)| *qty).sum();

        Self {
            parts,
            sheets,
            total_part_area,
            total_part_qty,
        }
    }

    pub fn parts(&self) -> &[(PartType, usize)] {
        &self.parts
    }

    pub fn sheets(&self) -> &[(SheetType, usize)] {
        &self.sheets
    }

    pub fn parttype(&self, index: usize) -> &PartType {
        &self.parts[index].0
    }

    pub fn parttype_qty(&self, index: usize) -> Option<usize> {
        self.parts.get(index).map(|(_, qty)| *qty)
    }

    pub fn sheettype(&self, index: usize) -> &SheetType {
        &self.sheets[index].0
    }

    pub fn sheettype_qty(&self, index: usize) -> Option<usize> {
        self.sheets.get(index).map(|(_, qty)| *qty)
    }

    pub fn smallest_sheet_value(&self) -> u64 {
        self.sheets.iter().map(|(s, _)| s.value).min().unwrap()
    }

    pub fn total_part_area(&self) -> u64 {
        self.total_part_area
    }

    pub fn total_part_qty(&self) -> usize {
        self.total_part_qty
    }
}
