use std::cell::RefCell;
use std::collections::LinkedList;
use std::rc::{Rc, Weak};

use crate::core::{cost::Cost, insertion::insertion_blueprint::InsertionBlueprint};
use crate::core::entities::node::Node;
use crate::optimization::rr::cache_updates::CacheUpdates;
use crate::Orientation;

use super::{sheettype::SheetType, parttype::PartType};

#[derive(Debug)]
pub struct Layout<'a> {
    id: usize,
    sheettype: &'a SheetType,
    top_node: Rc<RefCell<Node<'a>>>,
    cached_cost: RefCell<Option<Cost>>,
    cached_usage: RefCell<Option<f64>>,
    sorted_empty_nodes: Vec<Weak<RefCell<Node<'a>>>>,
}


impl<'a> Layout<'a> {
    pub fn new(sheettype: &'a SheetType, first_cut_orientation: Orientation, id: usize) -> Self {
        let mut top_node = Node::new(sheettype.width(), sheettype.height(), first_cut_orientation);
        let placeholder = Node::new(sheettype.width(), sheettype.height(), first_cut_orientation.rotate());
        top_node.add_child(Rc::new(RefCell::new(placeholder)));

        let top_node = Rc::new(RefCell::new(top_node));

        Self {
            id,
            sheettype,
            top_node,
            cached_cost: RefCell::new(None),
            cached_usage: RefCell::new(None),
            sorted_empty_nodes: Vec::new(),
        }
    }

    pub fn implement_insertion_blueprint(&mut self, blueprint: &InsertionBlueprint) -> CacheUpdates<Rc<RefCell<Node<'a>>>> {
        todo!()
    }

    pub fn remove_node(&mut self, node: &Rc<RefCell<Node<'a>>>) -> Vec<&'a PartType> {
        todo!()
    }

    pub fn is_empty(&self) -> bool {
        todo!()
    }

    pub fn create_deep_copy(&self) -> Layout<'a> {
        todo!()
    }

    pub fn get_cost(&self) -> Cost {
        todo!()
    }

    pub fn get_usage(&self) -> f64 {
        todo!()
    }

    fn recalculate_cost(&self) {
        todo!()
    }

    fn register_node(&mut self, node: &Node, recursive: bool) {
        todo!()
    }

    fn deregister_node(&mut self, node: &Node, recursive: bool) {
        todo!()
    }

    fn register_part(&mut self, parttype: &PartType) {
        todo!()
    }

    fn deregister_part(&mut self, parttype: &PartType) {
        todo!()
    }

    pub fn get_sorted_empty_nodes(&self) -> &Vec<Weak<RefCell<Node<'a>>>> {
        todo!()
    }


    pub fn id(&self) -> usize {
        self.id
    }
    pub fn sheettype(&self) -> &'a SheetType {
        self.sheettype
    }
    pub fn top_node(&self) -> &Rc<RefCell<Node<'a>>> {
        &self.top_node
    }
    pub fn cached_cost(&self) -> &RefCell<Option<Cost>> {
        &self.cached_cost
    }

    pub fn sorted_empty_nodes(&self) -> &Vec<Weak<RefCell<Node<'a>>>> {
        &self.sorted_empty_nodes
    }
}