use crate::core::entities::node::NodeKey;
use crate::core::layout_index::LayoutIndex;
use std::fmt::Debug;

//Insertion Option Cache Updates
pub struct IOCUpdates {
    removed_node: NodeKey,
    new_empty_nodes: [Option<NodeKey>; 2],
    layout_i: LayoutIndex,
}

impl IOCUpdates {
    pub fn new(layout_i: LayoutIndex, removed_node: NodeKey) -> Self {
        IOCUpdates {
            removed_node,
            new_empty_nodes: [None; 2],
            layout_i,
        }
    }

    pub fn add_new_empty(&mut self, item: NodeKey) {
        let slot = self
            .new_empty_nodes
            .iter_mut()
            .find(|node| node.is_none())
            .expect("insertion creates more than two empty nodes");
        *slot = Some(item);
    }

    pub fn removed_node(&self) -> &NodeKey {
        &self.removed_node
    }

    pub fn new_empty_nodes(&self) -> impl Iterator<Item = &NodeKey> {
        self.new_empty_nodes.iter().flatten()
    }

    pub fn layout_index(&self) -> &LayoutIndex {
        &self.layout_i
    }
}

impl Debug for IOCUpdates {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CacheUpdates {{ invalidated: {:#?}, new_entries: {:#?} }}",
            self.removed_node, self.new_empty_nodes
        )
    }
}
