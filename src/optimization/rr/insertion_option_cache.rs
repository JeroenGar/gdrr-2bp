use generational_arena::Index;
use itertools::Itertools;
use slotmap::{new_key_type, SlotMap};

use crate::core::entities::layout::Layout;
use crate::core::entities::node::Node;
use crate::core::entities::parttype::PartType;
use crate::core::insertion::insertion_option::InsertionOption;
use crate::core::layout_index::LayoutIndex;
use crate::core::rotation::Rotation;
use crate::optimization::instance::Instance;
use crate::optimization::problem::Problem;
use crate::optimization::rr::cache_updates::IOCUpdates;
use crate::util::multi_map::MultiMap;

/// A cache for InsertionOptions during the recreate phase
/// It allows very fast lookup of all InsertionOptions that are valid for a given node or a given parttype
/// It is kept up-to-date throughout the recreate phase, by receiving updates about which nodes are removed or added

pub struct InsertionOptionCache<'a> {
    options: SlotMap<InsertionOptionKey, InsertionOption<'a>>,
    option_node_map: MultiMap<(LayoutIndex, Index), InsertionOptionKey>,
    option_parttype_map: Vec<Vec<InsertionOptionKey>>,
}

impl<'a : 'b, 'b> InsertionOptionCache<'a> {
    pub fn new(instance: &Instance) -> Self {
        Self {
            options: SlotMap::with_key(),
            option_node_map: MultiMap::new(),
            option_parttype_map: (0..instance.parts().len()).map(|_| Vec::new()).collect_vec(),
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
                        if let Some(insertion_option) = InsertionOptionCache::generate_insertion_option(
                            empty_node,
                            parttype,
                            *layout_i,
                            *empty_node_i,
                        ) {
                            self.insert_option(insertion_option);
                        }
                    }
                }
            }
        }
    }

    pub fn add_for_node<I>(&mut self, node_i: &Index, node: &Node, layout_i: &LayoutIndex, parttypes: I)
        where I: Iterator<Item=&'b &'a PartType> {
        if node.parttype().is_none() && node.children().is_empty() {
            for parttype in parttypes.into_iter() {
                let insertion_option =
                    InsertionOptionCache::generate_insertion_option(node, parttype, *layout_i, *node_i);
                if let Some(insertion_option) = insertion_option {
                    self.insert_option(insertion_option);
                }
            }
        }
    }

    pub fn remove_for_node(&mut self, layout_i: &LayoutIndex, node_i: &Index) {
        let node_key = (*layout_i, *node_i);
        match self.option_node_map.remove_all(&node_key) {
            Some(option_keys) => {
                for option_key in option_keys {
                    let parttype_id = self.options[option_key].parttype().id();
                    let options = &mut self.option_parttype_map[parttype_id];
                    let index = options.iter().position(|key| *key == option_key).unwrap();
                    options.swap_remove(index);
                    self.options.remove(option_key).expect("Insertion option missing");
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

    fn insert_option(&mut self, insertion_option: InsertionOption<'a>) {
        let node_key = (
            *insertion_option.layout_index(),
            *insertion_option.original_node_index(),
        );
        let parttype_id = insertion_option.parttype().id();
        let option_key = self.options.insert(insertion_option);

        self.option_node_map.insert(node_key, option_key);
        self.option_parttype_map[parttype_id].push(option_key);
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

    pub fn get_for_parttype(
        &self,
        parttype: &PartType,
    ) -> impl ExactSizeIterator<Item = &InsertionOption<'a>> {
        self.option_parttype_map[parttype.id()]
            .iter()
            .map(|key| &self.options[*key])
    }

    pub fn get_for_node(
        &self,
        node_i: &Index,
        layout_i: &LayoutIndex,
    ) -> impl Iterator<Item = &InsertionOption<'a>> {
        self.option_node_map
            .get(&(*layout_i, *node_i))
            .into_iter()
            .flatten()
            .map(|key| &self.options[*key])
    }

    pub fn is_empty(&self) -> bool {
        let is_empty = self.options.is_empty();
        debug_assert_eq!(is_empty, self.option_parttype_map.iter().all(Vec::is_empty));
        debug_assert_eq!(is_empty, self.option_node_map.is_empty());
        is_empty
    }
}

new_key_type! {
    struct InsertionOptionKey;
}
