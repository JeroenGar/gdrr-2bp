use std::fmt::Debug;

use crate::core::entities::node::NodeKey;
use crate::core::entities::parttype::PartType;
use crate::core::insertion::insertion_blueprint::{InsertionBlueprint, InsertionShape};
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
    rotation: Option<Rotation>, // None means both rotations are checked lazily
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
        let mut append_blueprint = |shape: InsertionShape, rotation| {
            blueprints.push(InsertionBlueprint::new(
                self.layout_i,
                self.original_node_i,
                shape,
                rotation,
                self.parttype,
                original_node,
                leftover_valuation_power,
            ));
        };

        match self.rotation {
            Some(rotation) => {
                original_node.for_each_insertion_shape(
                    self.parttype,
                    rotation,
                    max_stages,
                    &mut |shape| append_blueprint(shape, rotation),
                );
            }
            None => {
                for rotation in [Rotation::Default, Rotation::Rotated] {
                    if original_node.insertion_possible(self.parttype, rotation) {
                        original_node.for_each_insertion_shape(
                            self.parttype,
                            rotation,
                            max_stages,
                            &mut |shape| append_blueprint(shape, rotation),
                        );
                    }
                }
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
