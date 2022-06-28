use crate::{Instance, JsonInstance, PartType, SheetType};
use crate::optimization::config::Config;
use crate::Rotation::Default;

pub fn generate_instance(json_instance : &JsonInstance, config : &Config) -> Instance{
    let parts = json_instance.parttypes.iter().map(|json_part| -> (PartType, usize) {
        (PartType::new(
            json_part.length,
            json_part.height,
            if config.rotation_allowed {None} else {Some(Default)}
        ), json_part.demand)
    }).collect::<Vec<_>>();

    let sheets = json_instance.sheettypes.iter().map(|json_sheet| -> (SheetType, usize) {
        (SheetType::new(
            json_sheet.length,
            json_sheet.height,
            json_sheet.cost
        ),
         match json_sheet.stock {
             Some(stock) => stock,
             None => usize::MAX
         })
    }).collect::<Vec<_>>();

    Instance::new(parts, sheets)

}