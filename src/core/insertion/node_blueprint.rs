use itertools::Itertools;
use slotmap::SlotMap;

use crate::core::cost::Cost;
use crate::core::entities::node::{Node, NodeKey};
use crate::core::entities::parttype::PartType;
use crate::core::leftover_valuator;
use crate::core::orientation::Orientation;

/// Owned tree snapshot of a layout node used for solution transfer and serialization.

#[derive(Debug, Clone)]
pub struct NodeBlueprint {
    pub width: u64,
    pub height: u64,
    pub children: Vec<NodeBlueprint>,
    pub parttype_id: Option<usize>,
    pub next_cut_orient: Orientation,
}

impl NodeBlueprint {
    pub fn new(width: u64, height: u64, parttype: Option<&PartType>, next_cut_orient: Orientation) -> Self {
        let children = Vec::new();
        let parttype_id = parttype.map(PartType::id);
        Self { width, height, children, parttype_id, next_cut_orient }
    }

    pub fn from_node(node_index: NodeKey, nodes: &SlotMap<NodeKey, Node>) -> Self {
        let node = &nodes[node_index];

        let (width, height) = (node.width(), node.height());
        let next_cut_orient = node.next_cut_orient();
        let parttype_id = node.parttype().map(PartType::id);
        let children = node.children(nodes)
            .map(|child_index| NodeBlueprint::from_node(child_index, nodes))
            .collect_vec();

        Self { width, height, parttype_id, children, next_cut_orient }
    }

    pub fn calculate_cost(&self, leftover_valuation_power: f32) -> Cost {
        if self.parttype_id.is_some() {
            Cost::empty()
        } else if self.children.is_empty() {
            Cost::new(
                0,
                leftover_valuator::valuate(self.area(), leftover_valuation_power),
                0,
                0,
            )
        } else {
            self.children
                .iter()
                .map(|child| child.calculate_cost(leftover_valuation_power))
                .sum()
        }
    }

    pub fn calculate_usage(&self) -> f64 {
        if self.parttype_id.is_some() {
            1.0
        } else if self.children.is_empty() {
            0.0
        } else {
            let usage = self.children
                .iter()
                .map(|child| child.area() as f64 * child.calculate_usage())
                .sum::<f64>() / self.area() as f64;
            debug_assert!(usage <= 1.0);
            usage
        }
    }

    pub fn is_empty(&self) -> bool {
        self.parttype_id.is_none() && self.children.is_empty()
    }

    pub fn area(&self) -> u64 {
        self.width * self.height
    }

}
