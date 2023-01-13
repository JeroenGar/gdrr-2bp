use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Weak;
use generational_arena::Index;

use crate::core::entities::node::Node;
use crate::core::layout_index::LayoutIndex;

pub struct CacheUpdates<T> {
    invalidated: Vec<T>,
    new_entries: Vec<T>,
    layout: LayoutIndex,
}

impl<T> CacheUpdates<T> {
    pub fn new(layout: LayoutIndex) -> Self {
        CacheUpdates {
            invalidated: Vec::new(),
            new_entries: Vec::new(),
            layout,
        }
    }

    pub fn add_invalidated(&mut self, item: T) {
        self.invalidated.push(item);
    }

    pub fn add_new(&mut self, item: T) {
        self.new_entries.push(item);
    }

    pub fn extend_new(&mut self, items: Vec<T>) {
        self.new_entries.extend(items);
    }

    pub fn invalidated(&self) -> &Vec<T> {
        &self.invalidated
    }

    pub fn new_entries(&self) -> &Vec<T> {
        &self.new_entries
    }

    pub fn layout(&self) -> &LayoutIndex {
        &self.layout
    }
}


impl<T : Debug> Debug for CacheUpdates<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CacheUpdates {{ invalidated: {:#?}, new_entries: {:#?} }}",
               &self.invalidated,
               &self.new_entries
        )
    }
}