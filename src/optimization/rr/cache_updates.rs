use std::fmt::Debug;
use generational_arena::Index;
use crate::core::layout_index::LayoutIndex;

pub struct CacheUpdates<T> {
    invalidated: Vec<T>,
    new_entries: Vec<T>,
    layout_i: LayoutIndex,
}

impl<T> CacheUpdates<T> {
    pub fn new(layout_i: LayoutIndex) -> Self {
        CacheUpdates {
            invalidated: vec![],
            new_entries: vec![],
            layout_i,
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

    pub fn layout_index(&self) -> &LayoutIndex {
        &self.layout_i
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