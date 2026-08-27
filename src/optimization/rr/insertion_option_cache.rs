use itertools::Itertools;
use std::ops::Range;

use crate::core::entities::layout::Layout;
use crate::core::entities::node::{Node, NodeKey};
use crate::core::entities::parttype::PartType;
use crate::core::insertion::insertion_option::InsertionOption;
use crate::core::layout_index::LayoutIndex;
use crate::core::rotation::Rotation;
use crate::optimization::instance::Instance;
use crate::optimization::problem::Problem;
use crate::optimization::rr::cache_updates::IOCUpdates;

/// A cache for InsertionOptions during the recreate phase
/// It allows very fast lookup of all InsertionOptions that are valid for a given node or a given parttype
/// It is kept up-to-date throughout the recreate phase, by receiving updates about which nodes are removed or added

pub struct InsertionOptionCache<'a> {
    options: Vec<CachedInsertionOption<'a>>,
    option_node_ranges: Vec<((LayoutIndex, NodeKey), Range<usize>)>,
    option_node_keys: Vec<u32>,
    option_parttype_map: Vec<Vec<u32>>,
}

impl<'a: 'b, 'b> InsertionOptionCache<'a> {
    pub fn new(instance: &Instance) -> Self {
        Self {
            options: Vec::new(),
            option_node_ranges: Vec::new(),
            option_node_keys: Vec::new(),
            option_parttype_map: (0..instance.parts().len())
                .map(|_| Vec::new())
                .collect_vec(),
        }
    }

    pub fn update_cache(
        &mut self,
        cache_updates: &IOCUpdates,
        parttypes: &Vec<&'a PartType>,
        problem: &Problem,
    ) {
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

    pub fn clear(&mut self) {
        self.options.clear();
        self.option_node_ranges.clear();
        self.option_node_keys.clear();
        self.option_parttype_map.iter_mut().for_each(Vec::clear);
    }

    pub fn add_for_parttypes<'c>(
        &mut self,
        parttypes: &[&'a PartType],
        layouts: impl Iterator<Item = (LayoutIndex, &'c Layout<'a>)>,
    ) where
        'a: 'c,
    {
        //sort by decreasing area
        let mut sorted_parttypes = parttypes.to_vec();
        sorted_parttypes.sort_by(|a, b| a.area().cmp(&b.area()).reverse());

        if sorted_parttypes.is_empty() {
            return;
        }
        let smallest_parttype_area = sorted_parttypes.last().unwrap().area();

        for (layout_i, layout) in layouts {
            let sorted_empty_nodes = layout.sorted_empty_nodes();
            let mut starting_index = 0;

            for empty_node_i in sorted_empty_nodes.iter() {
                let empty_node = &layout.nodes()[*empty_node_i];
                let empty_node_area = empty_node.area();
                if smallest_parttype_area > empty_node_area {
                    //The smallest parttype is larger than this node, there are no possible insertion options left.
                    break;
                }
                while empty_node_area < sorted_parttypes[starting_index].area() {
                    starting_index += 1;
                }
                let option_range_start = self.option_node_keys.len();
                for parttype in sorted_parttypes[starting_index..].iter().copied() {
                    if let Some(insertion_option) = InsertionOptionCache::generate_insertion_option(
                        empty_node,
                        parttype,
                        layout_i,
                        *empty_node_i,
                    ) {
                        self.insert_option(insertion_option);
                    }
                }
                if option_range_start != self.option_node_keys.len() {
                    self.option_node_ranges.push((
                        (layout_i, *empty_node_i),
                        option_range_start..self.option_node_keys.len(),
                    ));
                }
            }
        }
        self.option_node_ranges
            .sort_unstable_by_key(|(node_key, _)| *node_key);
    }

    pub fn add_for_node<I>(
        &mut self,
        node_i: &NodeKey,
        node: &Node,
        layout_i: &LayoutIndex,
        parttypes: I,
    ) where
        I: Iterator<Item = &'b &'a PartType>,
    {
        if node.parttype().is_none() && !node.has_children() {
            let option_range_start = self.option_node_keys.len();
            for parttype in parttypes {
                let insertion_option = InsertionOptionCache::generate_insertion_option(
                    node, parttype, *layout_i, *node_i,
                );
                if let Some(insertion_option) = insertion_option {
                    self.insert_option(insertion_option);
                }
            }
            if option_range_start != self.option_node_keys.len() {
                let node_key = (*layout_i, *node_i);
                let option_range = option_range_start..self.option_node_keys.len();
                match self
                    .option_node_ranges
                    .binary_search_by_key(&node_key, |(key, _)| *key)
                {
                    Ok(index) => {
                        debug_assert!(self.option_node_ranges[index].1.is_empty());
                        self.option_node_ranges[index].1 = option_range;
                    }
                    Err(index) => self
                        .option_node_ranges
                        .insert(index, (node_key, option_range)),
                }
            }
        }
    }

    pub fn remove_for_node(&mut self, layout_i: &LayoutIndex, node_i: &NodeKey) {
        let node_key = (*layout_i, *node_i);
        let Ok(node_range_index) = self
            .option_node_ranges
            .binary_search_by_key(&node_key, |(key, _)| *key)
        else {
            return;
        };
        let option_range = std::mem::take(&mut self.option_node_ranges[node_range_index].1);
        for node_position in option_range {
            let option_index = self.option_node_keys[node_position] as usize;
            self.remove_option(option_index);
        }
    }

    pub fn remove_all_for_layout(&mut self, layout_i: &LayoutIndex, layout: &Layout) {
        let sorted_empty_nodes = layout.sorted_empty_nodes();
        for empty_node_i in sorted_empty_nodes.iter() {
            self.remove_for_node(layout_i, empty_node_i);
        }
    }

    fn insert_option(&mut self, insertion_option: InsertionOption<'a>) {
        let parttype_id = insertion_option.parttype().id();
        let parttype_position = self.option_parttype_map[parttype_id].len();
        let node_position = self.option_node_keys.len();
        let option_index = self.options.len();
        let stored_option_index =
            u32::try_from(option_index).expect("insertion option cache exceeds u32 indices");
        self.options.push(CachedInsertionOption {
            option: insertion_option,
            parttype_position,
            node_position,
        });

        self.option_parttype_map[parttype_id].push(stored_option_index);
        self.option_node_keys.push(stored_option_index);
    }

    fn remove_option(&mut self, option_index: usize) {
        let parttype_id = self.options[option_index].option.parttype().id();
        let parttype_position = self.options[option_index].parttype_position;
        let node_position = self.options[option_index].node_position;
        debug_assert!(u32::try_from(option_index).is_ok());
        let stored_option_index = option_index as u32;
        debug_assert_eq!(self.option_node_keys[node_position], stored_option_index);

        let parttype_options = &mut self.option_parttype_map[parttype_id];
        debug_assert_eq!(parttype_options[parttype_position], stored_option_index);
        let removed_index = parttype_options.swap_remove(parttype_position);
        debug_assert_eq!(removed_index, stored_option_index);
        if let Some(&moved_index) = parttype_options.get(parttype_position) {
            self.options[moved_index as usize].parttype_position = parttype_position;
        }

        let last_option_index = self.options.len() - 1;
        debug_assert!(u32::try_from(last_option_index).is_ok());
        let last_stored_option_index = last_option_index as u32;
        self.options.swap_remove(option_index);
        if option_index != last_option_index {
            let moved_option = &self.options[option_index];
            let moved_parttype_id = moved_option.option.parttype().id();
            debug_assert_eq!(
                self.option_node_keys[moved_option.node_position],
                last_stored_option_index
            );
            debug_assert_eq!(
                self.option_parttype_map[moved_parttype_id][moved_option.parttype_position],
                last_stored_option_index,
            );
            self.option_node_keys[moved_option.node_position] = stored_option_index;
            self.option_parttype_map[moved_parttype_id][moved_option.parttype_position] =
                stored_option_index;
        }
    }

    fn generate_insertion_option(
        node: &Node,
        parttype: &'a PartType,
        layout_i: LayoutIndex,
        node_i: NodeKey,
    ) -> Option<InsertionOption<'a>> {
        match parttype.fixed_rotation() {
            Some(fixed_rotation) => match node.insertion_possible(parttype, *fixed_rotation) {
                true => Some(InsertionOption::new(
                    layout_i,
                    node_i,
                    parttype,
                    Some(*fixed_rotation),
                )),
                false => None,
            },
            None => {
                if node.insertion_possible(parttype, Rotation::Default)
                    || node.insertion_possible(parttype, Rotation::Rotated)
                {
                    Some(InsertionOption::new(layout_i, node_i, parttype, None))
                } else {
                    None
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
            .map(|key| &self.options[*key as usize].option)
    }

    pub fn get_for_node(
        &self,
        node_i: &NodeKey,
        layout_i: &LayoutIndex,
    ) -> impl Iterator<Item = &InsertionOption<'a>> {
        let node_key = (*layout_i, *node_i);
        let option_range = self
            .option_node_ranges
            .binary_search_by_key(&node_key, |(key, _)| *key)
            .map(|index| self.option_node_ranges[index].1.clone())
            .unwrap_or_default();
        self.option_node_keys[option_range]
            .iter()
            .map(|key| &self.options[*key as usize].option)
    }

    pub fn is_empty(&self) -> bool {
        let is_empty = self.options.is_empty();
        debug_assert_eq!(is_empty, self.option_parttype_map.iter().all(Vec::is_empty));
        debug_assert_eq!(
            is_empty,
            self.option_node_ranges
                .iter()
                .all(|(_, range)| range.is_empty())
        );
        is_empty
    }
}

struct CachedInsertionOption<'a> {
    option: InsertionOption<'a>,
    parttype_position: usize,
    node_position: usize,
}
