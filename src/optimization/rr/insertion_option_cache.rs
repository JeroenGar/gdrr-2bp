use std::borrow::Borrow;
use std::cell::RefCell;
use std::io::empty;
use std::rc::{Rc, Weak};

use by_address::ByAddress;
use indexmap::IndexSet;

use crate::{PartType, Rotation};
use crate::core::entities::layout::Layout;
use crate::core::entities::node::Node;
use crate::core::insertion::insertion_blueprint::InsertionBlueprint;
use crate::core::insertion::insertion_option::InsertionOption;
use crate::optimization::problem::Problem;
use crate::optimization::rr::cache_updates::CacheUpdates;
use crate::util::multi_map::MultiMap;
use crate::util::macros::{rb,rbm};

pub struct InsertionOptionCache<'a> {
    option_node_map: MultiMap<ByAddress<Rc<RefCell<Node<'a>>>>, Rc<InsertionOption<'a>>>,
    option_parttype_map: MultiMap<&'a PartType, Rc<InsertionOption<'a>>>,
}

impl<'a : 'b, 'b> InsertionOptionCache<'a> {
    pub fn new() -> Self {
        Self {
            option_node_map: MultiMap::new(),
            option_parttype_map: MultiMap::new(),
        }
    }

    pub fn update_cache(&mut self, cache_updates: &CacheUpdates<'a, Weak<RefCell<Node<'a>>>>, parttypes: &Vec<&'a PartType>)
    {
        cache_updates.invalidated().iter().for_each(|node| {
            match node.upgrade() {
                Some(node) => {
                    self.remove_for_node(&node);
                }
                None => {}
            }
        });
        let layout = cache_updates.layout().clone();
        cache_updates.new_entries().iter().for_each(|node| {
            let node = node.upgrade().unwrap();
            self.add_for_node(&node, layout.clone(), parttypes.iter());
        });
    }

    pub fn add_for_parttypes<I>(&mut self, parttypes: I, layouts: &Vec<Rc<RefCell<Layout<'a>>>>)
        where I: Iterator<Item=&'b &'a PartType>
    {
        let mut sorted_parttypes: Vec<&&PartType> = Vec::from_iter(parttypes);
        if sorted_parttypes.is_empty() {
            return;
        }
        //sort by decreasing area
        sorted_parttypes.sort_by(|a, b| {
            a.area().cmp(&b.area()).reverse()
        });

        for layout in layouts.iter() {
            let layout_ref = rb!(layout);
            let sorted_empty_nodes = layout_ref.sorted_empty_nodes();
            let mut starting_index = 0;
            for empty_node in sorted_empty_nodes.iter() {
                let empty_node = empty_node.upgrade().unwrap();
                let empty_node_ref = rb!(empty_node);
                if sorted_parttypes.get(sorted_parttypes.len() - 1).unwrap().area() > empty_node_ref.area() {
                    //The smallest parttype is larger than this node, there are no possible insertion options left.
                    break;
                }
                for i in starting_index..sorted_parttypes.len() {
                    let parttype = *sorted_parttypes.get(i).unwrap();

                    if empty_node_ref.area() < parttype.area() {
                        //The empty node is smaller than this parttype. For the next (smaller) empty node, start searching from next index
                        starting_index = i + 1;
                    } else {
                        let insertion_option = InsertionOptionCache::generate_insertion_option(&empty_node, parttype, Rc::downgrade(layout));
                        match insertion_option {
                            Some(insertion_option) => {
                                let insertion_option = Rc::new(insertion_option);
                                self.option_node_map.insert(ByAddress(empty_node.clone()), insertion_option.clone());
                                self.option_parttype_map.insert(parttype, insertion_option.clone());
                            }
                            None => {}
                        }
                    }
                }
            }
        }
    }

    pub fn add_for_node<I>(&mut self, node: &Rc<RefCell<Node<'a>>>, layout: Weak<RefCell<Layout<'a>>>, parttypes: I)
        where I: Iterator<Item=&'b &'a PartType> {
        let node_ref = rb!(node);
        if node_ref.parttype().is_none() && node_ref.children().is_empty() {
            for parttype in parttypes.into_iter() {
                let insertion_option = InsertionOptionCache::generate_insertion_option(node, parttype, layout.clone());
                match insertion_option {
                    Some(insertion_option) => {
                        let insertion_option = Rc::new(insertion_option);
                        self.option_node_map.insert(ByAddress(node.clone()), insertion_option.clone());
                        self.option_parttype_map.insert(parttype, insertion_option.clone());
                    }
                    None => {}
                }
            }
        }
    }

    pub fn remove_for_parttype(&mut self, parttype: &'a PartType) {
        match self.option_parttype_map.get(&parttype){
            Some(options) => {
                for insert_opt in options {
                    self.option_node_map.remove(&ByAddress(insert_opt.original_node().upgrade().unwrap()), insert_opt);
                }
            }
            None => ()
        }
        self.option_parttype_map.remove_all(&parttype);
    }

    pub fn remove_for_node(&mut self, node: &Rc<RefCell<Node<'a>>>) {
        match self.option_node_map.get(&ByAddress(node.clone())) {
            Some(options) => {
                for insert_opt in options {
                    self.option_parttype_map.remove(&insert_opt.parttype(), insert_opt);
                }
            }
            None => ()
        }
        self.option_node_map.remove_all(&ByAddress(node.clone()));
    }

    pub fn remove_for_layout(&mut self, layout: &Rc<RefCell<Layout<'a>>>) {
        let layout = rb!(layout);
        let sorted_empty_nodes = layout.sorted_empty_nodes();
        for empty_node in sorted_empty_nodes.iter() {
            let empty_node = empty_node.upgrade().unwrap();
            self.remove_for_node(&empty_node);
        }
    }

    fn generate_insertion_option(node: &Rc<RefCell<Node<'a>>>, parttype: &'a PartType, layout: Weak<RefCell<Layout<'a>>>) -> Option<InsertionOption<'a>> {
        let node_ref = rb!(node);
        match parttype.fixed_rotation() {
            Some(fixed_rotation) => {
                match node_ref.insertion_possible(parttype, *fixed_rotation) {
                    true => Some(InsertionOption::new(Rc::downgrade(&node), parttype, Some(*fixed_rotation), layout)),
                    false => None
                }
            }
            None => {
                let default_possible = node_ref.insertion_possible(parttype, Rotation::Default);
                let rotated_possible = node_ref.insertion_possible(parttype, Rotation::Rotated);
                match (default_possible, rotated_possible) {
                    (true, true) => {
                        Some(InsertionOption::new(Rc::downgrade(&node), parttype, None, layout))
                    }
                    (true, false) => {
                        Some(InsertionOption::new(Rc::downgrade(&node), parttype, Some(Rotation::Default), layout))
                    }
                    (false, true) => {
                        Some(InsertionOption::new(Rc::downgrade(&node), parttype, Some(Rotation::Rotated), layout))
                    }
                    (false, false) => {
                        None
                    }
                }
            }
        }
    }

    pub fn get_for_parttype(&self, parttype: &'a PartType) -> Option<&Vec<Rc<InsertionOption<'a>>>> {
        self.option_parttype_map.get(&parttype)
    }

    pub fn get_for_node(&self, node: &Rc<RefCell<Node<'a>>>) -> Option<&Vec<Rc<InsertionOption<'a>>>> {
        self.option_node_map.get(&ByAddress(node.clone()))
    }

    pub fn is_empty(&self) -> bool {
        self.option_parttype_map.is_empty() && self.option_node_map.is_empty()
    }
}
