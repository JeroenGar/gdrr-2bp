use std::fmt::Debug;
use generational_arena::Index;
use crate::core::layout_index::LayoutIndex;

//Insertion Option Cache Updates
pub struct IOCUpdates {
    removed_nodes: Vec<Index>,
    new_nodes: Vec<Index>,
    layout_i: LayoutIndex,
}

impl IOCUpdates {
    pub fn new(layout_i: LayoutIndex) -> Self {
        IOCUpdates {
            removed_nodes: vec![],
            new_nodes: vec![],
            layout_i,
        }
    }

    pub fn add_removed(&mut self, item: Index) {
        self.removed_nodes.push(item);
    }

    pub fn add_new(&mut self, item: Index) {
        self.new_nodes.push(item);
    }

    pub fn extend_new(&mut self, items: Vec<Index>) {
        self.new_nodes.extend(items);
    }

    pub fn removed_nodes(&self) -> &Vec<Index> {
        &self.removed_nodes
    }

    pub fn new_nodes(&self) -> &Vec<Index> {
        &self.new_nodes
    }

    pub fn layout_index(&self) -> &LayoutIndex {
        &self.layout_i
    }
}


impl Debug for IOCUpdates {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CacheUpdates {{ invalidated: {:#?}, new_entries: {:#?} }}",
               &self.removed_nodes,
               &self.new_nodes
        )
    }
}