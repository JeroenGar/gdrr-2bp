use std::rc::{Rc};

use generational_arena::{Index};
use itertools::Itertools;

use crate::{PartType, Rotation};
use crate::core::entities::layout::Layout;
use crate::core::entities::node::Node;
use crate::core::insertion::insertion_option::InsertionOption;
use crate::core::layout_index::LayoutIndex;
use crate::optimization::problem::Problem;
use crate::optimization::rr::cache_updates::IOCUpdates;
use crate::util::multi_map::MultiMap;

/// This struct functions as a cache for all InsertionOptions during the recreate phase
/// It is kept up to date by removing and adding InsertionOptions when nodes are removed or added
pub struct InsertionOptionCache<'a> {
    option_node_map: MultiMap<(LayoutIndex, Index), Rc<InsertionOption<'a>>>,
    option_parttype_map: MultiMap<&'a PartType, Rc<InsertionOption<'a>>>,
}

impl<'a : 'b, 'b> InsertionOptionCache<'a> {
    pub fn new() -> Self {
        Self {
            option_node_map: MultiMap::new(),
            option_parttype_map: MultiMap::new(),
        }
    }

    pub fn update_cache(&mut self, cache_updates: &IOCUpdates, parttypes: &Vec<&'a PartType>, problem: &Problem){
        let layout_i = cache_updates.layout_index();
        cache_updates.removed_nodes().iter().for_each(|node_i| {
            self.remove_for_node(layout_i, node_i);
        });
        let layout = problem.get_layout(layout_i);
        cache_updates.new_nodes().iter().for_each(|node_i| {
            let node = &layout.nodes()[*node_i];
            self.add_for_node(node_i, node, layout_i, parttypes.iter());
        });
    }

    pub fn add_for_parttypes(&mut self, parttypes: &[&'a PartType], layouts: &[(LayoutIndex, &Layout)])
    {
        //sort by decreasing area
        let sorted_parttypes: Vec<&&PartType> = parttypes.iter()
            .sorted_by(|a, b| a.area().cmp(&b.area()).reverse())
            .collect_vec();

        if sorted_parttypes.is_empty() {
            return;
        }

        for (layout_i, layout) in layouts {
            let sorted_empty_nodes = layout.sorted_empty_nodes();
            let mut starting_index = 0;

            for empty_node_i in sorted_empty_nodes.iter() {
                let mut generated_insertion_options = Vec::new();
                let empty_node = &layout.nodes()[*empty_node_i];
                if sorted_parttypes[sorted_parttypes.len() - 1].area() > empty_node.area() {
                    //The smallest parttype is larger than this node, there are no possible insertion options left.
                    break;
                }
                for i in starting_index..sorted_parttypes.len() {
                    let parttype = *sorted_parttypes.get(i).unwrap();

                    if empty_node.area() < parttype.area() {
                        //The empty node is smaller than this parttype. For the next (smaller) empty node, start searching from next index
                        starting_index = i + 1;
                    } else {
                        let insertion_option = InsertionOptionCache::generate_insertion_option(empty_node, parttype, *layout_i, *empty_node_i);
                        match insertion_option {
                            Some(insertion_option) => {
                                let insertion_option = Rc::new(insertion_option);
                                generated_insertion_options.push(insertion_option.clone());
                            }
                            None => {}
                        }
                    }
                }
                //update the maps
                for insertion_option in &generated_insertion_options {
                    let parttype = insertion_option.parttype();
                    self.option_parttype_map.insert(parttype, insertion_option.clone());
                }
                self.option_node_map.insert_all((*layout_i, *empty_node_i), generated_insertion_options);
            }
        }
    }

    pub fn add_for_node<I>(&mut self, node_i: &Index, node: &Node, layout_i: &LayoutIndex, parttypes: I)
        where I: Iterator<Item=&'b &'a PartType> {
        if node.parttype().is_none() && node.children().is_empty() {
            for parttype in parttypes.into_iter() {
                let insertion_option =
                    InsertionOptionCache::generate_insertion_option(node, parttype, *layout_i, *node_i);
                match insertion_option {
                    Some(insertion_option) => {
                        let insertion_option = Rc::new(insertion_option);
                        let node_key = (*layout_i, *node_i);
                        self.option_node_map.insert(node_key, insertion_option.clone());
                        self.option_parttype_map.insert(parttype, insertion_option.clone());
                    }
                    None => {}
                }
            }
        }
    }

    pub fn remove_for_node(&mut self, layout_i: &LayoutIndex, node_i: &Index) {
        let node_key = (*layout_i, *node_i);
        match self.option_node_map.remove_all(&node_key) {
            Some(options) => {
                for insert_opt in options {
                    self.option_parttype_map.remove(&insert_opt.parttype(), &insert_opt);
                }
            }
            None => ()
        }
    }

    pub fn remove_all_for_layout(&mut self, layout_i: &LayoutIndex, layout: &Layout) {
        let sorted_empty_nodes = layout.sorted_empty_nodes();
        for empty_node_i in sorted_empty_nodes.iter() {
            self.remove_for_node(layout_i, empty_node_i);
        }
    }

    fn generate_insertion_option(node: &Node, parttype: &'a PartType, layout_i: LayoutIndex, node_i: Index) -> Option<InsertionOption<'a>> {
        match parttype.fixed_rotation() {
            Some(fixed_rotation) => {
                match node.insertion_possible(parttype, *fixed_rotation) {
                    true => Some(InsertionOption::new(layout_i, node_i, parttype, Some(*fixed_rotation))),
                    false => None
                }
            }
            None => {
                let default_possible = node.insertion_possible(parttype, Rotation::Default);
                let rotated_possible = node.insertion_possible(parttype, Rotation::Rotated);
                match (default_possible, rotated_possible) {
                    (true, true) => {
                        Some(InsertionOption::new(layout_i, node_i, parttype, None))
                    }
                    (true, false) => {
                        Some(InsertionOption::new(layout_i, node_i, parttype,  Some(Rotation::Default)))
                    }
                    (false, true) => {
                        Some(InsertionOption::new(layout_i, node_i, parttype, Some(Rotation::Rotated)))
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

    pub fn get_for_node(&self, node_i: &Index, layout_i: &LayoutIndex) -> Option<&Vec<Rc<InsertionOption<'a>>>> {
        self.option_node_map.get(&(*layout_i, *node_i))
    }

    pub fn is_empty(&self) -> bool {
        self.option_parttype_map.is_empty() && self.option_node_map.is_empty()
    }
}
