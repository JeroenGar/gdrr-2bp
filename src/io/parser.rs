use std::sync::Arc;

use serde_json::json;

use crate::{Instance, JsonInstance, Orientation, PartType, SheetType};
use crate::core::entities::sendable_layout::SendableLayout;
use crate::core::insertion::node_blueprint::NodeBlueprint;
use crate::io::json_format::{JsonCP, JsonCPNode, JsonCPNodeType, JsonOrientation, JsonSolution};
use crate::io::json_format::JsonCPNodeType::Structure;
use crate::optimization::config::Config;
use crate::optimization::solutions::sendable_solution::SendableSolution;
use crate::Rotation::Default;

pub fn generate_instance(json_instance: &mut JsonInstance, config: &Config) -> Instance {
    let mut part_id = 0;
    let mut parts = Vec::new();
    for json_part in json_instance.parttypes.iter_mut() {
        json_part.reference = Some(part_id);
        let parttype = PartType::new(
            part_id,
            json_part.length,
            json_part.height,
            if config.rotation_allowed { None } else { Some(Default) },
        );
        let demand = json_part.demand;
        parts.push((parttype, demand));
        part_id += 1;
    }

    let mut sheet_id = 0;
    let mut sheets = Vec::new();
    for json_sheet in json_instance.sheettypes.iter_mut() {
        json_sheet.reference = Some(sheet_id);
        let sheettype = SheetType::new(
            sheet_id,
            json_sheet.length,
            json_sheet.height,
            json_sheet.cost,
            None,
        );
        let stock = match json_sheet.stock {
            Some(stock) => stock,
            None => usize::MAX
        };
        sheets.push((sheettype, stock));
        sheet_id += 1;
    }

    Instance::new(parts, sheets)
}

pub fn generate_json_solution(json_instance: &JsonInstance, solution: &SendableSolution) -> JsonSolution {
    let name = json_instance.name.clone();
    let sheettypes = json_instance.sheettypes.clone();
    let parttypes = json_instance.parttypes.clone();

    let mut cutting_patterns = solution.layouts().iter().map(
        |l| { convert_layout_to_json_cp(l) }
    ).collect::<Vec<JsonCP>>();

    JsonSolution{
        name,
        sheettypes,
        parttypes,
        cutting_patterns
    }
}

pub fn convert_layout_to_json_cp(layout: &SendableLayout) -> JsonCP {
    let object = layout.sheettype_id();
    let root = convert_node_bp_to_json_cp_node(layout.top_node());

    JsonCP{
        object,
        root
    }
}

pub fn convert_node_bp_to_json_cp_node(node: &NodeBlueprint) -> JsonCPNode {
    let mut children = Vec::new();
    for child in node.children() {
        children.push(convert_node_bp_to_json_cp_node(child));
    }
    let length = node.width();
    let height = node.height();

    let node_type = match (node.parttype_id(),node.children().is_empty()){
        (None,true) => JsonCPNodeType::Leftover,
        (None, false) => JsonCPNodeType::Structure,
        (Some(_),true) => JsonCPNodeType::Item,
        (Some(_), false) => {
            panic!("This should not happen")
        },
    };

    let orientation = match (&node_type, node.next_cut_orient()){
        (JsonCPNodeType::Structure,Orientation::Horizontal) => Some(JsonOrientation::H),
        (JsonCPNodeType::Structure,Orientation::Vertical) => Some(JsonOrientation::V),
        (_,_) => None,
    };

    let item = match &node_type {
        JsonCPNodeType::Item => Some(node.parttype_id().unwrap()),
        _ => None,
    };

    JsonCPNode{
        length,
        height,
        orientation,
        node_type,
        item,
        children
    }
}