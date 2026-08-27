use slotmap::SlotMap;
use itertools::Itertools;
use crate::core::{
    cost::Cost,
    insertion::insertion_blueprint::{InsertionBlueprint, InsertionNodeKind},
};
use crate::core::entities::node::{Node, NodeKey};
use crate::core::orientation::Orientation;
use crate::optimization::rr::cache_updates::IOCUpdates;
use crate::util::assertions;

use super::sheettype::SheetType;

#[derive(Debug, Clone)]
pub struct Layout<'a> {
    id : usize,
    sheettype: &'a SheetType,
    leftover_valuation_power: f32,
    nodes: LayoutNodes<'a>,
    cached_cost: Option<Cost>,
}

impl<'a> Layout<'a> {
    pub fn new(
        id: usize,
        sheettype: &'a SheetType,
        first_cut_orientation: Orientation,
        leftover_valuation_power: f32,
    ) -> Self {
        let top_node = Node::new(0, sheettype.width, sheettype.height, first_cut_orientation, None);
        let nodes = LayoutNodes::new(top_node);

        let mut layout = Self {
            id,
            sheettype,
            leftover_valuation_power,
            nodes,
            cached_cost: None,
        };

        //The top node cannot be modified, so we register a placeholder node to be able to insert parts
        let placeholder_node = Node::new(1, sheettype.width, sheettype.height, first_cut_orientation.rotate(), None);
        layout.register_node(placeholder_node, layout.nodes.top_node, true);

        layout
    }

    pub fn clone_with_id(&self, id : usize) -> Self{
        Self {
            id,
            ..self.clone()
        }
    }

    pub(crate) fn restore_from(&mut self, snapshot: &Self) {
        self.id = snapshot.id;
        self.sheettype = snapshot.sheettype;
        self.leftover_valuation_power = snapshot.leftover_valuation_power;
        self.nodes.restore_from(&snapshot.nodes);
        self.cached_cost.clone_from(&snapshot.cached_cost);
    }

    pub fn implement_insertion_blueprint(
        &mut self,
        blueprint: &InsertionBlueprint<'a>,
        updates: &mut IOCUpdates,
    ) {
        let original = *blueprint.original_node_index();
        let parent = self.nodes.arena[original].parent().expect("original node has no parent");
        let insertion_nodes = blueprint.nodes(&self.nodes.arena[original]);

        //unregister the original node
        self.unregister_node(original, &mut None);

        //create and register the replacements
        let mut inserted_nodes = [None; 5];
        for (i, insertion_node) in insertion_nodes.into_iter().enumerate() {
            let Some(insertion_node) = insertion_node else {
                break;
            };
            let parent = match insertion_node.parent {
                Some(parent_i) => {
                    debug_assert!(parent_i < i);
                    inserted_nodes[parent_i].expect("insertion node parent has not been created")
                }
                None => parent,
            };
            let parttype = match insertion_node.kind {
                InsertionNodeKind::Part => Some(blueprint.parttype()),
                InsertionNodeKind::Structure | InsertionNodeKind::Empty => None,
            };
            let is_empty = insertion_node.kind == InsertionNodeKind::Empty;
            let node = Node::new(
                self.nodes.arena[parent].level() + 1,
                insertion_node.width,
                insertion_node.height,
                insertion_node.next_cut_orient,
                parttype,
            );
            let node_index = self.register_node(node, parent, is_empty);
            if is_empty {
                updates.add_new_empty(node_index);
            }
            inserted_nodes[i] = Some(node_index);
        }

        debug_assert!(assertions::children_nodes_fit(&parent, &self.nodes.arena), "{:#?}", blueprint);
        self.nodes.debug_assert_valid();
    }

    pub fn remove_node(&mut self, node_index: NodeKey) -> Vec<usize>{
        /*®
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

        let parent_node_index = self.nodes.arena[node_index].parent().expect("Cannot remove a node without a parent");
        let parent_node = &self.nodes.arena[parent_node_index];

        //Check if there is an empty_node present
        let empty_node = parent_node.last_child.filter(|child| self.nodes.arena[*child].is_empty());
        debug_assert_eq!(
            empty_node,
            parent_node.children(&self.nodes.arena).find(|child| self.nodes.arena[*child].is_empty()),
        );

        let mut removed_parts = Some(vec![]);

        match empty_node {
            Some(empty_node_index) => {
                //Scenario 1 and 3
                if parent_node.first_child != parent_node.last_child || parent_node.parent().is_none() {
                    //Scenario 1 (also do this when the parent node is the root)
                    //Two children are merged into one

                    let node = &self.nodes.arena[node_index];
                    let empty_node = &self.nodes.arena[empty_node_index];
                    let replacement_node = match parent_node.next_cut_orient() {
                        Orientation::Horizontal => {
                            let new_height = empty_node.height() + node.height();
                            Node::new(node.level(), node.width(), new_height, node.next_cut_orient(), None)
                        }
                        Orientation::Vertical => {
                            let new_width = empty_node.width() + node.width();
                            Node::new(node.level(), new_width, node.height(), node.next_cut_orient(), None)
                        }
                    };

                    //Replace the empty node and the node to be removed with a enlarged empty node
                    self.unregister_node(empty_node_index, &mut removed_parts);
                    self.unregister_node(node_index, &mut removed_parts);
                    self.register_node(replacement_node, parent_node_index, true);
                } else {
                    //Scenario 3: replace the parent with an empty node
                    let grandparent_index = parent_node.parent().expect("grandparent node needs to be present");

                    //create empty parent
                    let empty_parent_node = Node::new(parent_node.level(), parent_node.width(), parent_node.height(), parent_node.next_cut_orient(), None);

                    //replace
                    self.unregister_node(parent_node_index, &mut removed_parts);
                    self.register_node(empty_parent_node, grandparent_index, true);
                }
            }
            None => {
                //Scenario 2: convert the node itself into an empty node

                //create empty replacement node
                let node = &self.nodes.arena[node_index];
                let replacement_node = Node::new(node.level(), node.width(), node.height(), node.next_cut_orient(), None);

                //replace
                self.unregister_node(node_index, &mut removed_parts);
                self.register_node(replacement_node, parent_node_index, true);
            }
        }

        self.nodes.debug_assert_valid();

        removed_parts.unwrap()
    }

    fn invalidate_caches(&mut self) {
        self.cached_cost = None;
    }

    fn calculate_cost(&self) -> Cost {
        debug_assert!(assertions::cached_sorted_empty_nodes_correct(&self.nodes.arena, &self.nodes.empty_nodes_by_area));
        let material_cost = Cost::empty().add_material_cost(self.sheettype.value);
        self.nodes.empty_nodes_by_area.iter()
            .map(|node_index| self.nodes.arena[*node_index].calculate_cost(self.leftover_valuation_power))
            .fold(material_cost, |acc, cost| acc.add(&cost))
    }

    fn calculate_usage(&self) -> f64 {
        self.nodes.used_part_area as f64 / self.sheettype.area() as f64
    }

    fn register_node(&mut self, node: Node<'a>, parent: NodeKey, is_empty: bool) -> NodeKey {
        self.invalidate_caches();
        self.nodes.insert_child(node, parent, is_empty)
    }

    fn unregister_node(&mut self, node_index: NodeKey, removed_part_ids: &mut Option<Vec<usize>>) {
        self.invalidate_caches();
        self.nodes.remove_subtree(node_index, removed_part_ids);
    }

    pub fn included_part_ids(&self) -> Vec<usize> {
        self.nodes.arena.iter()
            .filter_map(|(_, node)| node.parttype().map(|parttype| parttype.id()))
            .collect_vec()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.arena.iter().all(|(_, n)| n.is_empty())
    }

    pub fn cost(&mut self, force_recalc: bool) -> Cost {
        let cost = match (self.cached_cost.as_ref(), force_recalc) {
            (Some(cost), false) => cost.clone(),
            _ => {
                let cost = self.calculate_cost();
                self.cached_cost = Some(cost.clone());
                cost
            }
        };
        debug_assert!(force_recalc || cost == self.cost(true));
        cost
    }

    pub fn cost_immut(&self, force_recalc: bool) -> Cost {
        let cost = match (self.cached_cost.as_ref(), force_recalc) {
            (Some(cost), false) => cost.clone(),
            _ => {
                let cost = self.calculate_cost();
                cost
            }
        };
        debug_assert!(force_recalc || cost == self.cost_immut(true));
        cost
    }

    pub fn usage(&mut self) -> f64 {
        debug_assert!(assertions::cached_used_part_area_correct(&self.nodes.arena, self.nodes.used_part_area));
        self.calculate_usage()
    }

    pub fn usage_immut(&self) -> f64 {
        debug_assert!(assertions::cached_used_part_area_correct(&self.nodes.arena, self.nodes.used_part_area));
        self.calculate_usage()
    }

    pub fn sorted_empty_nodes(&self) -> &[NodeKey] {
        debug_assert!(assertions::node_arena_valid(&self.nodes.arena, &self.nodes.top_node), "{:#?}", self.nodes.empty_nodes_by_area.iter().map(|n| &self.nodes.arena[*n]).collect_vec());
        debug_assert!(assertions::cached_sorted_empty_nodes_correct(&self.nodes.arena, &self.nodes.empty_nodes_by_area), "{:#?}", self.nodes.empty_nodes_by_area.iter().map(|n| &self.nodes.arena[*n]).collect_vec());
        &self.nodes.empty_nodes_by_area
    }

    pub fn removable_nodes(&self) -> &[NodeKey] {
        debug_assert!(assertions::cached_removable_nodes_correct(&self.nodes.arena, &self.nodes.removable_nodes));
        &self.nodes.removable_nodes
    }

    pub fn sheettype(&self) -> &'a SheetType {
        self.sheettype
    }

    pub fn leftover_valuation_power(&self) -> f32 {
        self.leftover_valuation_power
    }

    pub fn top_node_index(&self) -> &NodeKey {
        &self.nodes.top_node
    }

    pub fn nodes(&self) -> &SlotMap<NodeKey, Node<'a>> {
        &self.nodes.arena
    }

}

/// Owns a layout's mutable node topology and its derived lookup state.
///
/// Keeping mutations here prevents the arena links, root, used area, and cached node lists from drifting apart.
#[derive(Debug, Clone)]
struct LayoutNodes<'a> {
    arena: SlotMap<NodeKey, Node<'a>>,
    top_node: NodeKey,
    used_part_area: u64,
    empty_nodes_by_area: Vec<NodeKey>,
    removable_nodes: Vec<NodeKey>,
}

impl<'a> LayoutNodes<'a> {
    fn new(top_node: Node<'a>) -> Self {
        let mut arena = SlotMap::with_key();
        let top_node = arena.insert(top_node);
        Self {
            arena,
            top_node,
            used_part_area: 0,
            empty_nodes_by_area: Vec::new(),
            removable_nodes: Vec::new(),
        }
    }

    fn restore_from(&mut self, snapshot: &Self) {
        self.arena.clone_from(&snapshot.arena);
        self.top_node = snapshot.top_node;
        self.used_part_area = snapshot.used_part_area;
        self.empty_nodes_by_area.clone_from(&snapshot.empty_nodes_by_area);
        self.removable_nodes.clone_from(&snapshot.removable_nodes);
    }

    fn insert_child(&mut self, node: Node<'a>, parent: NodeKey, is_empty: bool) -> NodeKey {
        if let Some(parttype) = node.parttype() {
            self.used_part_area += parttype.area();
        }

        debug_assert!(node.level() == self.arena[parent].level() + 1);

        let parent_had_children = self.arena[parent].has_children();
        let node_index = self.arena.insert(node);

        if self.arena[node_index].parttype().is_some() {
            self.register_removable(node_index);
        }

        if is_empty {
            debug_assert!(self.arena[node_index].is_empty());
            let node_area = self.arena[node_index].area();
            let result = self.empty_nodes_by_area.binary_search_by(&|key: &NodeKey| {
                self.arena[*key].area().cmp(&node_area).reverse()
            });
            let position = result.unwrap_or_else(|position| position);
            self.empty_nodes_by_area.insert(position, node_index);
        }

        let previous_sibling = self.arena[parent].last_child;
        self.arena[node_index].parent = Some(parent);
        self.arena[node_index].previous_sibling = previous_sibling;
        match previous_sibling {
            Some(previous_sibling) => self.arena[previous_sibling].next_sibling = Some(node_index),
            None => self.arena[parent].first_child = Some(node_index),
        }
        self.arena[parent].last_child = Some(node_index);
        if !parent_had_children {
            self.register_removable(parent);
        }

        debug_assert!(assertions::node_arena_valid(&self.arena, &self.top_node));
        node_index
    }

    fn remove_subtree(&mut self, node_index: NodeKey, removed_part_ids: &mut Option<Vec<usize>>) {
        if self.arena[node_index].is_empty() {
            let node_area = self.arena[node_index].area();
            let lower_index = self.empty_nodes_by_area.partition_point(|key| self.arena[*key].area() > node_area);

            if self.empty_nodes_by_area[lower_index] == node_index {
                self.empty_nodes_by_area.remove(lower_index);
            } else {
                let upper_index = self.empty_nodes_by_area.partition_point(|key| self.arena[*key].area() >= node_area);

                let mut node_found = false;
                for position in lower_index..upper_index {
                    if self.empty_nodes_by_area[position] == node_index {
                        self.empty_nodes_by_area.remove(position);
                        node_found = true;
                        break;
                    }
                }
                if !node_found {
                    panic!("Empty node not found in sorted_empty_nodes");
                }
            }
        }

        while let Some(child) = self.arena[node_index].first_child {
            self.remove_subtree(child, removed_part_ids);
        }

        self.unregister_removable(node_index);

        let node = self.arena.remove(node_index).expect("Node to be removed does not exist");
        debug_assert!(node.first_child.is_none() && node.last_child.is_none());

        if let &Some(parttype) = node.parttype() {
            if let Some(removed_parts) = removed_part_ids {
                removed_parts.push(parttype.id());
            }
            self.used_part_area -= parttype.area();
        }

        if let Some(parent) = node.parent() {
            match node.previous_sibling {
                Some(previous_sibling) => self.arena[previous_sibling].next_sibling = node.next_sibling,
                None => self.arena[parent].first_child = node.next_sibling,
            }
            match node.next_sibling {
                Some(next_sibling) => self.arena[next_sibling].previous_sibling = node.previous_sibling,
                None => self.arena[parent].last_child = node.previous_sibling,
            }
            if !self.arena[parent].has_children() {
                self.unregister_removable(parent);
            }
        }

        debug_assert!(assertions::node_arena_valid(&self.arena, &self.top_node));
    }

    fn register_removable(&mut self, node_index: NodeKey) {
        debug_assert!(self.arena[node_index].removable_position().is_none());
        debug_assert!(self.arena[node_index].parttype().is_some() || self.arena[node_index].has_children());
        let position = self.removable_nodes.len();
        self.arena[node_index].set_removable_position(Some(position));
        self.removable_nodes.push(node_index);
    }

    fn unregister_removable(&mut self, node_index: NodeKey) {
        let Some(position) = self.arena[node_index].removable_position() else {
            return;
        };
        let removed_node = self.removable_nodes.swap_remove(position);
        debug_assert_eq!(removed_node, node_index);
        self.arena[node_index].set_removable_position(None);
        if let Some(&moved_node) = self.removable_nodes.get(position) {
            self.arena[moved_node].set_removable_position(Some(position));
        }
    }

    fn debug_assert_valid(&self) {
        debug_assert!(assertions::node_arena_valid(&self.arena, &self.top_node));
        debug_assert!(assertions::cached_sorted_empty_nodes_correct(&self.arena, &self.empty_nodes_by_area), "{:#?}", self.empty_nodes_by_area.iter().map(|key| &self.arena[*key]).collect_vec());
        debug_assert!(assertions::cached_removable_nodes_correct(&self.arena, &self.removable_nodes));
    }
}
