use std::fmt::Debug;

use crate::core::cost::Cost;
use crate::core::entities::node::NodeKey;
use crate::core::entities::parttype::PartType;
use crate::core::insertion::insertion_blueprint::InsertionBlueprint;
use crate::core::insertion::node_blueprint::NodeBlueprint;
use crate::core::layout_index::LayoutIndex;
use crate::core::rotation::Rotation;
use crate::optimization::problem::Problem;

/// Represents the possibility to insert a parttype into a node with a certain rotation
/// Does not define how exactly, just that it is possible
///
/// InsertionOptions can generate InsertionBlueprints which define exactly how a part is inserted

#[derive(Debug, PartialEq, Eq)]
pub struct InsertionOption<'a> {
    layout_i: LayoutIndex,
    original_node_i: NodeKey,
    parttype: &'a PartType,
    rotation: Option<Rotation>, // None means both rotations are possible
}

impl<'a> InsertionOption<'a> {
    pub fn new(layout_i: LayoutIndex, original_node_i: NodeKey, parttype: &'a PartType, rotation: Option<Rotation>) -> Self {
        Self {
            layout_i,
            original_node_i,
            parttype,
            rotation,
        }
    }

    pub fn append_blueprints(
        &self,
        problem: &Problem,
        blueprints: &mut Vec<InsertionBlueprint<'a>>,
    ) {
        let layout = problem.get_layout(&self.layout_i);
        let leftover_valuation_power = layout.leftover_valuation_power();
        let original_node = &layout.nodes()[self.original_node_i];
        let max_stages = layout.sheettype().max_stages();
        let original_cost = original_node.calculate_cost(leftover_valuation_power);
        let mut append_blueprint = |replacements: Vec<NodeBlueprint>| {
            let new_cost = replacements.iter()
                .map(|replacement| replacement.calculate_cost(leftover_valuation_power))
                .sum::<Cost>();
            let insertion_cost = new_cost.subtract(&original_cost);
            blueprints.push(InsertionBlueprint::new(
                self.layout_i,
                self.original_node_i,
                replacements,
                self.parttype,
                insertion_cost,
            ));
        };

        match self.rotation {
            Some(rotation) => {
                original_node.for_each_insertion_replacement(
                    self.parttype,
                    rotation,
                    max_stages,
                    &mut append_blueprint,
                );
            }
            None => {
                original_node.for_each_insertion_replacement(
                    self.parttype,
                    Rotation::Default,
                    max_stages,
                    &mut append_blueprint,
                );
                original_node.for_each_insertion_replacement(
                    self.parttype,
                    Rotation::Rotated,
                    max_stages,
                    &mut append_blueprint,
                );
            }
        }
    }

    pub fn parttype(&self) -> &'a PartType {
        self.parttype
    }

    pub fn rotation(&self) -> Option<Rotation> {
        self.rotation
    }

    pub fn original_node_index(&self) -> &NodeKey {
        &self.original_node_i
    }

    pub fn layout_index(&self) -> &LayoutIndex {
        &self.layout_i
    }
}
