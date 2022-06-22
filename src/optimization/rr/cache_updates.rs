use std::cell::RefCell;
use std::rc::Weak;
use crate::core::entities::layout::Layout;

pub struct CacheUpdates<'a,T> {
    invalidated: Vec<T>,
    new_entries: Vec<T>,
    layout: Weak<RefCell<Layout<'a>>>,
}

impl<'a,T> CacheUpdates<'a, T> {
    pub fn new(layout : Weak<RefCell<Layout<'a>>>) -> Self{
        CacheUpdates {
            invalidated: Vec::new(),
            new_entries: Vec::new(),
            layout
        }
    }

    /*pub fn new(invalidated: Vec<T>, new_entries: Vec<T>) -> Self {
        Self { invalidated, new_entries }
    }*/

    pub fn add_invalidated(&mut self, item: T) {
        self.invalidated.push(item);
    }

    pub fn add_new(&mut self, item: T) {
        self.new_entries.push(item);
    }

    pub fn invalidated(&self) -> &Vec<T> {
        &self.invalidated
    }

    pub fn new_entries(&self) -> &Vec<T> {
        &self.new_entries
    }


    pub fn layout(&self) -> &Weak<RefCell<Layout<'a>>> {
        &self.layout
    }
}