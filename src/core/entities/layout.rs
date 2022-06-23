use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::core::{cost::Cost, insertion::insertion_blueprint::InsertionBlueprint};
use crate::core::entities::node::Node;
use crate::optimization::rr::cache_updates::CacheUpdates;
use crate::Orientation;
use crate::util::assertions;

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

    pub fn implement_insertion_blueprint(&mut self, blueprint: &InsertionBlueprint<'a>, cache_updates : &mut CacheUpdates<'a,Weak<RefCell<Node<'a>>>>)  {
        self.invalidate_caches();

        let original_node = blueprint.original_node().upgrade().unwrap();
        let mut parent_node = original_node.as_ref().borrow_mut().parent().as_ref().unwrap().upgrade().unwrap();

        //convert the NodeBlueprints to Nodes
        let mut replacements = Vec::new();
        let mut all_created_nodes = Vec::new();
        for node_blueprint in blueprint.replacements().iter() {
            let node = Node::new_from_blueprint(node_blueprint, Rc::downgrade(&parent_node), &mut all_created_nodes);
            all_created_nodes.push(Rc::downgrade(&node));
            replacements.push(node);
        }
        parent_node.as_ref().borrow_mut().replace_child(&original_node, replacements);

        //update the cache
        cache_updates.add_invalidated(Rc::downgrade(&original_node));
        all_created_nodes.iter().for_each(
            |node| {
                cache_updates.add_new(node.clone());
                self.register_node(node.clone());
            }
        );
        self.register_part(blueprint.parttype());
    }

    pub fn remove_node(&mut self, node: &Rc<RefCell<Node<'a>>>) -> Rc<RefCell<Node<'a>>> {
        self.invalidate_caches();

        let mut parent = node.as_ref().borrow().parent().as_ref().unwrap().upgrade().unwrap();
        let removed_node = parent.as_ref().borrow_mut().remove_child(node);
        removed_node
    }

    fn invalidate_caches(&mut self){
        self.cached_cost.replace(None);
        self.cached_usage.replace(None);
    }

    fn recalculate_cost(&self) {
        todo!()
    }

    fn register_node(&mut self, node: Weak<RefCell<Node>>) {
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

    pub fn get_included_parts(&self) -> Vec<&'a PartType> {
        let mut included_parts = Vec::new();
        self.top_node.as_ref().borrow().get_included_parts(&mut included_parts);

        included_parts
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

    pub fn get_sorted_empty_nodes(&self) -> &Vec<Weak<RefCell<Node<'a>>>> {
        todo!()
    }

    pub fn get_removable_nodes(&self) -> Vec<Weak<RefCell<Node<'a>>>> {
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