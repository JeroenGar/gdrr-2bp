use std::cell::RefCell;
use std::rc::{Weak};
use generational_arena::Index;

use crate::core::cost::Cost;
use crate::core::entities::layout::Layout;
use crate::core::entities::node::Node;
use crate::core::insertion::node_blueprint::NodeBlueprint;
use crate::core::layout_index::LayoutIndex;
use crate::PartType;
use crate::util::assertions;
use crate::util::macros::{rb};

#[derive(Debug, Clone)]
pub struct InsertionBlueprint<'a> {
    layout: LayoutIndex,
    original_node: Index,
    replacements: Vec<NodeBlueprint>,
    parttype: &'a PartType,
    cost: Cost,
}


impl<'a> InsertionBlueprint<'a> {
    pub fn new(layout: LayoutIndex, original_node: Index, replacements: Vec<NodeBlueprint>, parttype: &'a PartType, cost: Cost) -> Self {
        //TODO: fix this
        //debug_assert!(rb!(original_node.upgrade().unwrap()).parent().is_some(), "{:#?}", original_node.upgrade().unwrap());
        //debug_assert!(assertions::replacements_fit(&original_node, &replacements), "{:#?}", (&original_node, &replacements));

        Self {
            layout,
            original_node,
            replacements,
            parttype,
            cost
        }
    }

    pub fn replacements(&self) -> &Vec<NodeBlueprint> {
        &self.replacements
    }
    pub fn parttype(&self) -> &'a PartType {
        self.parttype
    }
    pub fn cost(&self) -> &Cost {
        &self.cost
    }
    pub fn layout(&self) -> LayoutIndex {
        self.layout
    }
    pub fn original_node(&self) -> Index {
        self.original_node
    }


    pub fn set_original_node(&mut self, original_node: Index) {
        self.original_node = original_node;
    }
    pub fn set_replacements(&mut self, replacements: Vec<NodeBlueprint>) {
        self.replacements = replacements;
    }
    pub fn set_parttype(&mut self, parttype: &'a PartType) {
        self.parttype = parttype;
    }
}