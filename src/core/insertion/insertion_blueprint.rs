use generational_arena::Index;

use crate::core::cost::Cost;
use crate::core::insertion::node_blueprint::NodeBlueprint;
use crate::core::layout_index::LayoutIndex;
use crate::PartType;

#[derive(Debug, Clone)]
pub struct InsertionBlueprint<'a> {
    layout_i: LayoutIndex,
    original_node_i: Index,
    replacements: Vec<NodeBlueprint>,
    parttype: &'a PartType,
    cost: Cost,
}


impl<'a> InsertionBlueprint<'a> {
    pub fn new(layout_i: LayoutIndex, original_node_i: Index, replacements: Vec<NodeBlueprint>, parttype: &'a PartType, cost: Cost) -> Self {
        Self {
            layout_i,
            original_node_i,
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

    pub fn layout_index(&self) -> &LayoutIndex {
        &self.layout_i
    }

    pub fn original_node_index(&self) -> &Index {
        &self.original_node_i
    }

}