use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::collections::BinaryHeap;
use std::ops::Deref;
use std::rc::{Rc, Weak};

use crate::core::{cost::Cost, insertion::insertion_blueprint::InsertionBlueprint};
use crate::core::entities::node::Node;
use crate::optimization::config::Config;
use crate::optimization::rr::cache_updates::CacheUpdates;
use crate::Orientation;
use crate::util::assertions;
use crate::util::macros::{rb, rbm};

use super::{parttype::PartType, sheettype::SheetType};

#[derive(Debug)]
pub struct Layout<'a> {
    id: usize,
    sheettype: &'a SheetType,
    top_node: Rc<RefCell<Node<'a>>>,
    cached_cost: RefCell<Option<Cost>>,
    cached_usage: RefCell<Option<f64>>,
    sorted_empty_nodes: Vec<Weak<RefCell<Node<'a>>>>, //sorted by descending area
}

impl<'a> Layout<'a> {
    pub fn new(sheettype: &'a SheetType, first_cut_orientation: Orientation, id: usize) -> Self {
        let top_node = Node::new_top_node(sheettype.width(), sheettype.height(), first_cut_orientation);

        let mut layout = Self {
            id,
            sheettype,
            top_node: top_node.clone(),
            cached_cost: RefCell::new(None),
            cached_usage: RefCell::new(None),
            sorted_empty_nodes: Vec::new(),
        };
        layout.register_node(&top_node, true);

        layout
    }

    pub fn create_deep_copy(&self, id: usize) -> Layout<'a> {
        let sheettype = self.sheettype();
        let top_node_copy = rbm!(self.top_node).create_deep_copy(None);

        let mut copy = Layout {
            id,
            sheettype,
            top_node: top_node_copy.clone(),
            cached_cost: RefCell::new(None),
            cached_usage: RefCell::new(None),
            sorted_empty_nodes: Vec::new(),
        };
        copy.register_node(&top_node_copy, true);

        debug_assert!(assertions::all_weak_references_alive(&copy.sorted_empty_nodes));

        copy
    }


    pub fn implement_insertion_blueprint(&mut self, blueprint: &InsertionBlueprint<'a>, cache_updates: &mut CacheUpdates<'a, Weak<RefCell<Node<'a>>>>) {
        debug_assert!(assertions::node_belongs_to_layout(&blueprint.original_node().upgrade().unwrap(),self));
        let original_node = blueprint.original_node().upgrade().unwrap();
        let mut parent_node = rbm!(original_node).parent().as_ref().unwrap().upgrade().unwrap();

        //convert the NodeBlueprints to Nodes
        let mut replacements = Vec::new();
        let mut all_created_nodes = Vec::new();
        for node_blueprint in blueprint.replacements().iter() {
            let node = Node::new_from_blueprint(node_blueprint, Rc::downgrade(&parent_node), &mut all_created_nodes);
            replacements.push(node);
        }
        //(un)register applicable nodes
        self.unregister_node(&original_node, false);
        replacements.iter().for_each(|n| { self.register_node(&n, true) });

        Node::replace_child(&parent_node, &original_node, replacements);

        //update the cache
        cache_updates.add_invalidated(Rc::downgrade(&original_node));
        all_created_nodes.iter().for_each(
            |node| {
                cache_updates.add_new(node.clone());
            }
        );

        debug_assert!(assertions::children_nodes_fit(rb!(parent_node).deref()), "{:#?}", blueprint);
    }

    pub fn remove_node(&mut self, node: &Rc<RefCell<Node<'a>>>) {
        /*
           Scenario 1: Empty node present + other child(ren)
            -> expand existing waste piece

             ---******               ---******
                *$$$$*                  *$$$$*
                ******                  ******
                *XXXX*     ----->       *    *
                ******                  *    *
                *    *                  *    *
             ---******               ---******

             Scenario 2: No waste piece present
                -> convert Node to be removed into waste Node

             ---******               ---******
                *$$$$*                  *$$$$*
                ******    ----->        ******
                *XXXX*                  *    *
             ---******               ---******

             Scenario 3: No other children present besides waste piece
                -> convert parent into waste piece

             ---******               ---******
                *XXXX*                  *    *
                ******    ----->        *    *
                *    *                  *    *
             ---******               ---******

         */

        let mut parent_node = rb!(node).parent().as_ref().unwrap().upgrade().unwrap();
        //Check if there is an empty_node present
        let empty_node = rb!(parent_node).children().iter().find(|node: &&Rc<RefCell<Node>>| {
            rb!(node).is_empty()
        }).cloned();

        match empty_node {
            Some(empty_node) => {
                //Scenario 1 and 3

                if rb!(parent_node).children().len() > 1 || rb!(parent_node).parent().is_none() {
                    //Scenario 1 (also do this when the parent node is the root)
                    //Two children are merged into one
                    let replacement_node = match rb!(parent_node).next_cut_orient() {
                        Orientation::Horizontal => {
                            let new_height = rb!(empty_node).height() + rb!(node).height();
                            Rc::new(RefCell::new(
                                Node::new(rb!(node).width(), new_height, rb!(node).next_cut_orient())
                            ))
                        }
                        Orientation::Vertical => {
                            let new_width = rb!(empty_node).width() + rb!(node).width();
                            Rc::new(RefCell::new(
                                Node::new(new_width, rb!(node).height(), rb!(node).next_cut_orient())
                            ))
                        }
                    };
                    self.unregister_node(node, true);
                    self.unregister_node(&empty_node, false);

                    //Replace the empty node and the node to be removed with a enlarged empty node
                    Node::replace_children(&parent_node, vec![node, &empty_node], vec![replacement_node.clone()]);
                    self.register_node(&replacement_node, false);
                } else {
                    //Scenario 3: convert the parent into an empty node
                    self.unregister_node(&parent_node, true);
                    Node::clear_children(&parent_node);
                    self.register_node(&parent_node, true);
                }
            }
            None => {
                //Scenario 2: convert the node itself into an empty node
                let replacement_node = Rc::new(RefCell::new(
                    Node::new(rb!(node).width(), rb!(node).height(), rb!(node).next_cut_orient())
                ));
                self.unregister_node(node, true);
                Node::replace_child(&parent_node, node, vec![replacement_node.clone()]);
                self.register_node(&replacement_node, false);
            }
        }
    }

    fn invalidate_caches(&mut self) {
        self.cached_cost.replace(None);
        self.cached_usage.replace(None);
    }

    fn calculate_cost(&self) -> Cost {
        let mut cost = rb!(self.top_node).calculate_cost();
        cost.material_cost = self.sheettype.value();

        cost
    }

    fn calculate_usage(&self) -> f64 {
        rb!(self.top_node).calculate_usage()
    }

    fn register_node(&mut self, node: &Rc<RefCell<Node<'a>>>, recursive: bool) {
        self.invalidate_caches();

        let node_ref = rb!(node);

        //All empty nodes need to be added to the sorted empty nodes list
        if node_ref.is_empty() {
            let result = self.sorted_empty_nodes.binary_search_by(
                &(|n: &Weak<RefCell<Node<'a>>>| {
                    let n_area = rb!(n.upgrade().unwrap()).area();
                    n_area.cmp(&node_ref.area()).reverse()
                }));

            match result {
                Ok(index) => self.sorted_empty_nodes.insert(index, Rc::downgrade(node)),
                Err(index) => self.sorted_empty_nodes.insert(index, Rc::downgrade(node)),
            }
            debug_assert!(assertions::nodes_sorted_descending_area(&self.sorted_empty_nodes));
            debug_assert!(assertions::all_nodes_have_parents(&self.sorted_empty_nodes), "{:#?}", self.sorted_empty_nodes.iter().map(|n| n.upgrade().unwrap()).collect::<Vec<_>>());
        }
        if node_ref.parttype().is_some() {
            self.register_part(node_ref.parttype().unwrap());
        }
        if recursive {
            node_ref.children().iter().for_each(|child| {
                self.register_node(child, true);
            });
        }
    }

    fn unregister_node(&mut self, node: &Rc<RefCell<Node<'a>>>, recursive: bool) {
        self.invalidate_caches();

        let node_ref = rb!(node);

        //All empty nodes need to be removed from the sorted empty nodes list
        if node_ref.is_empty() {
            let lower_index = self.sorted_empty_nodes.partition_point(|n|
                { rb!(n.upgrade().unwrap()).area() > node_ref.area() });

            if Weak::ptr_eq(&self.sorted_empty_nodes[lower_index], &Rc::downgrade(node)) {
                //We have found the correct node, remove it
                self.sorted_empty_nodes.remove(lower_index);
            } else {
                let upper_index = self.sorted_empty_nodes.partition_point(|n|
                    { rb!(n.upgrade().unwrap()).area() >= node_ref.area() });

                let mut node_found = false;
                for i in lower_index..upper_index {
                    if Weak::ptr_eq(&self.sorted_empty_nodes[i], &Rc::downgrade(node)) {
                        //We have found the correct node, remove it
                        self.sorted_empty_nodes.remove(i);
                        node_found = true;
                        break;
                    }
                }
                if !node_found {
                    panic!("Node not found in sorted_empty_nodes");
                }
            }
        }
        if node_ref.parttype().is_some() {
            self.unregister_part(node_ref.parttype().unwrap());
        }
        if recursive {
            node_ref.children().iter().for_each(|child| {
                self.unregister_node(child, true);
            });
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
        rb!(self.top_node).get_included_parts(&mut included_parts);

        included_parts
    }

    pub fn is_empty(&self) -> bool {
        rb!(self.top_node).children().iter().all(
            |n| {
                rb!(n).is_empty()
            })
    }

    pub fn cost(&self) -> Cost {
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

    pub fn usage(&self) -> f64 {
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

    pub fn sorted_empty_nodes(&self) -> &Vec<Weak<RefCell<Node<'a>>>> {
        debug_assert!(assertions::nodes_sorted_descending_area(&self.sorted_empty_nodes), "{:#?}", self.sorted_empty_nodes.iter().map(|n| rb!(n.upgrade().unwrap()).area()).collect::<Vec<_>>());
        debug_assert!(assertions::all_nodes_have_parents(&self.sorted_empty_nodes), "{:#?}", self.sorted_empty_nodes.iter().map(|n| n.upgrade().unwrap()).collect::<Vec<_>>());
        &self.sorted_empty_nodes
    }

    pub fn get_removable_nodes(&self) -> Vec<Weak<RefCell<Node<'a>>>> {
        let mut nodes = Vec::new();
        rb!(self.top_node).get_all_removable_children(&mut nodes);

        nodes
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
}