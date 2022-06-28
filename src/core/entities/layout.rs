use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::collections::BinaryHeap;
use std::rc::{Rc, Weak};

use crate::core::{cost::Cost, insertion::insertion_blueprint::InsertionBlueprint};
use crate::core::entities::node::Node;
use crate::optimization::config::Config;
use crate::optimization::rr::cache_updates::CacheUpdates;
use crate::Orientation;
use crate::util::assertions;

use super::{parttype::PartType, sheettype::SheetType};

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

        let mut layout = Self {
            id,
            sheettype,
            top_node,
            cached_cost: RefCell::new(None),
            cached_usage: RefCell::new(None),
            sorted_empty_nodes: Vec::new(),
        };
        let mut all_nodes = vec![Rc::downgrade(&layout.top_node)];
        layout.top_node.as_ref().borrow().get_all_children(&mut all_nodes);

        all_nodes.into_iter().for_each(|node| {
            layout.register_node(node);
        });

        layout
    }

    pub fn create_deep_copy(&self, id : usize) -> Layout<'a> {
        let sheettype = self.sheettype();
        let top_node_copy = self.top_node.as_ref().borrow().create_deep_copy(None);

        let mut copy = Layout {
            id,
            sheettype,
            top_node : top_node_copy.clone(),
            cached_cost: RefCell::new(None),
            cached_usage : RefCell::new(None),
            sorted_empty_nodes : Vec::new(),
        };

        let mut all_new_nodes = vec![Rc::downgrade(&top_node_copy)];
        top_node_copy.as_ref().borrow().get_all_children(&mut all_new_nodes);

        all_new_nodes.into_iter().for_each(|node| {
            copy.register_node(node);
        });

        copy
    }


    pub fn implement_insertion_blueprint(&mut self, blueprint: &InsertionBlueprint<'a>, cache_updates: &mut CacheUpdates<'a, Weak<RefCell<Node<'a>>>>) {
        let original_node = blueprint.original_node().upgrade().unwrap();
        let mut parent_node = original_node.as_ref().borrow_mut().parent().as_ref().unwrap().upgrade().unwrap();

        //convert the NodeBlueprints to Nodes
        let mut replacements = Vec::new();
        let mut all_created_nodes = Vec::new();
        for node_blueprint in blueprint.replacements().iter() {
            let node = Node::new_from_blueprint(node_blueprint, Rc::downgrade(&parent_node), &mut all_created_nodes);
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

        debug_assert!(assertions::children_nodes_fit(&parent_node));
    }

    pub fn remove_node(&mut self, node: &Rc<RefCell<Node<'a>>>) -> Rc<RefCell<Node<'a>>> {

        //remove the node from the tree
        let mut parent_node = node.as_ref().borrow().parent().as_ref().unwrap().upgrade().unwrap();
        let removed_node = parent_node.as_ref().borrow_mut().remove_child(node);

        //unregister the released nodes and parts
        let mut removed_nodes = Vec::new();
        removed_node.as_ref().borrow().get_all_children(&mut removed_nodes);

        removed_nodes.iter().for_each(|node| {
            self.unregister_node(node);
        });

        debug_assert!(assertions::children_nodes_fit(&parent_node));

        removed_node
    }

    fn invalidate_caches(&mut self) {
        self.cached_cost.replace(None);
        self.cached_usage.replace(None);
    }

    fn calculate_cost(&self) -> Cost {
        let mut cost = self.top_node.as_ref().borrow().calculate_cost();
        cost.material_cost = self.sheettype.value();

        cost
    }

    fn calculate_usage(&self) -> f64 {
        self.top_node.as_ref().borrow().calculate_usage()
    }

    fn register_node(&mut self, node: Weak<RefCell<Node<'a>>>) {
        self.invalidate_caches();

        let node_rc = node.upgrade().unwrap();
        let node_ref = node_rc.as_ref().borrow();

        //All empty nodes need to be added to the sorted empty nodes list
        if node_ref.is_empty() {
            let result = self.sorted_empty_nodes.binary_search_by(
                &(|n : &Weak<RefCell<Node<'a>>>| {
                let n_area = n.upgrade().unwrap().as_ref().borrow().area();
                    n_area.cmp(&node_ref.area())
            }));

            match result {
                Ok(index) => self.sorted_empty_nodes.insert(index, node),
                Err(index) => self.sorted_empty_nodes.insert(index, node),
            }
        }
        if node_ref.parttype().is_some() {
            self.register_part(node_ref.parttype().unwrap());
        }
    }

    fn unregister_node(&mut self, node: &Weak<RefCell<Node<'a>>>) {
        self.invalidate_caches();

        let node_rc = node.upgrade().unwrap();
        let node_ref = node_rc.as_ref().borrow();

        //All empty nodes need to be removed from the sorted empty nodes list
        if node_ref.is_empty() {
            let lower_index = self.sorted_empty_nodes.partition_point(|n|
                {n.upgrade().unwrap().as_ref().borrow().area() < node_ref.area()});

            if Weak::ptr_eq(&self.sorted_empty_nodes[lower_index],&node){
                //We have found the correct node, remove it
                self.sorted_empty_nodes.remove(lower_index);
            }
            else{
                let upper_index = self.sorted_empty_nodes.partition_point(|n|
                    {n.upgrade().unwrap().as_ref().borrow().area() <= node_ref.area()});

                let mut node_found = false;
                for i in lower_index..upper_index{
                    if Weak::ptr_eq(&self.sorted_empty_nodes[i],&node){
                        //We have found the correct node, remove it
                        self.sorted_empty_nodes.remove(i);
                        node_found = true;
                        break;
                    }
                }
                if !node_found{
                    panic!("Node not found in sorted_empty_nodes");
                }
            }
        }
        if node_ref.parttype().is_some(){
            self.unregister_part(node_ref.parttype().unwrap());
        }
    }

    fn register_part(&mut self, parttype: &PartType) {
        self.invalidate_caches();
    }

    fn unregister_part(&mut self, parttype: &PartType) {
        self.invalidate_caches();
    }

    pub fn get_included_parts(&self) -> Vec<&'a PartType> {
        let mut included_parts = Vec::new();
        self.top_node.as_ref().borrow().get_included_parts(&mut included_parts);

        included_parts
    }

    pub fn is_empty(&self) -> bool {
        self.top_node.as_ref().borrow().is_empty()
    }

    pub fn get_cost(&self) -> Cost {
        let mut cached_cost = self.cached_cost.borrow_mut();
        match cached_cost.as_ref() {
            Some(cost) => {
                debug_assert!(*cost == self.calculate_cost());
                cost.clone()
            }
            None => {
                let cost = self.calculate_cost();
                cached_cost.replace(cost.clone());
                cost
            }
        }
    }

    pub fn get_usage(&self) -> f64 {
        let mut cached_usage = self.cached_usage.borrow_mut();
        match cached_usage.as_ref() {
            Some(usage) => {
                debug_assert!(*usage == self.calculate_usage());
                *usage
            }
            None => {
                let usage = self.calculate_usage();
                cached_usage.replace(usage);
                usage
            }
        }
    }

    pub fn get_sorted_empty_nodes(&self) -> &Vec<Weak<RefCell<Node<'a>>>> {
        debug_assert!(assertions::nodes_sorted_descending_area(&self.sorted_empty_nodes));
        &self.sorted_empty_nodes
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

    pub fn sorted_empty_nodes(&self) -> &Vec<Weak<RefCell<Node<'a>>>> {
        &self.sorted_empty_nodes
    }
}