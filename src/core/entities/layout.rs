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

use super::{parttype::PartType, sheettype::SheetType};

#[derive(Debug, Clone)]
pub struct Layout<'a> {
    id : usize,
    sheettype: &'a SheetType,
    leftover_valuation_power: f32,
    nodes: SlotMap<NodeKey, Node<'a>>,
    top_node_i: NodeKey,
    cached_cost: Option<Cost>,
    used_part_area: u64,
    sorted_empty_nodes: Vec<NodeKey>, //sorted by descending area
}

impl<'a> Layout<'a> {
    pub fn new(
        id: usize,
        sheettype: &'a SheetType,
        first_cut_orientation: Orientation,
        leftover_valuation_power: f32,
    ) -> Self {
        let mut nodes = SlotMap::with_key();
        let top_node = Node::new(0, sheettype.width(), sheettype.height(), first_cut_orientation, None);
        let top_node_i = nodes.insert(top_node);

        let mut layout = Self {
            id,
            sheettype,
            leftover_valuation_power,
            nodes,
            top_node_i,
            cached_cost: None,
            used_part_area: 0,
            sorted_empty_nodes: vec![],
        };

        //The top node cannot be modified, so we register a placeholder node to be able to insert parts
        let placeholder_node = Node::new(1, sheettype.width(), sheettype.height(), first_cut_orientation.rotate(), None);
        layout.register_node(placeholder_node, top_node_i, true);

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
        self.nodes.clone_from(&snapshot.nodes);
        self.top_node_i = snapshot.top_node_i;
        self.cached_cost.clone_from(&snapshot.cached_cost);
        self.used_part_area = snapshot.used_part_area;
        self.sorted_empty_nodes.clone_from(&snapshot.sorted_empty_nodes);
    }

    pub fn implement_insertion_blueprint(
        &mut self,
        blueprint: &InsertionBlueprint<'a>,
        updates: &mut IOCUpdates,
    ) {
        let original = *blueprint.original_node_index();
        let parent = self.nodes[original].parent().expect("original node has no parent");
        let insertion_nodes = blueprint.nodes(&self.nodes[original]);

        //unregister the original node
        self.unregister_node(original, &mut None);
        updates.add_removed(original);

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
                self.nodes[parent].level() + 1,
                insertion_node.width,
                insertion_node.height,
                insertion_node.next_cut_orient,
                parttype,
            );
            let node_index = self.register_node(node, parent, is_empty);
            updates.add_new(node_index);
            inserted_nodes[i] = Some(node_index);
        }

        debug_assert!(assertions::children_nodes_fit(&parent, &self.nodes), "{:#?}", blueprint);
        debug_assert!(assertions::node_arena_valid(&self.nodes, &self.top_node_i));
        debug_assert!(assertions::cached_sorted_empty_nodes_correct(&self.nodes(), &self.sorted_empty_nodes), "{:#?}", self.sorted_empty_nodes.iter().map(|n| &self.nodes[*n]).collect_vec());
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

        let parent_node_index = self.nodes[node_index].parent().expect("Cannot remove a node without a parent");
        let parent_node = &self.nodes[parent_node_index];

        //Check if there is an empty_node present
        let empty_node = parent_node.children(&self.nodes).find(|child| self.nodes[*child].is_empty());

        let mut removed_parts = Some(vec![]);

        match empty_node {
            Some(empty_node_index) => {
                //Scenario 1 and 3
                if parent_node.first_child != parent_node.last_child || parent_node.parent().is_none() {
                    //Scenario 1 (also do this when the parent node is the root)
                    //Two children are merged into one

                    let node = &self.nodes[node_index];
                    let empty_node = &self.nodes[empty_node_index];
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
                let node = &self.nodes[node_index];
                let replacement_node = Node::new(node.level(), node.width(), node.height(), node.next_cut_orient(), None);

                //replace
                self.unregister_node(node_index, &mut removed_parts);
                self.register_node(replacement_node, parent_node_index, true);
            }
        }

        debug_assert!(assertions::node_arena_valid(&self.nodes, &self.top_node_i));
        debug_assert!(assertions::cached_sorted_empty_nodes_correct(&self.nodes(), &self.sorted_empty_nodes), "{:#?}", self.sorted_empty_nodes.iter().map(|n| &self.nodes[*n]).collect_vec());

        removed_parts.unwrap()
    }

    fn invalidate_caches(&mut self) {
        self.cached_cost = None;
    }

    fn calculate_cost(&self) -> Cost {
        debug_assert!(assertions::cached_sorted_empty_nodes_correct(&self.nodes, &self.sorted_empty_nodes));
        let material_cost = Cost::empty().add_material_cost(self.sheettype.value());
        self.sorted_empty_nodes.iter()
            .map(|node_index| self.nodes[*node_index].calculate_cost(self.leftover_valuation_power))
            .fold(material_cost, |acc, cost| acc.add(&cost))
    }

    fn calculate_usage(&self) -> f64 {
        self.used_part_area as f64 / self.sheettype.area() as f64
    }

    fn register_node(&mut self, node: Node<'a>, parent: NodeKey, is_empty: bool) -> NodeKey {
        self.invalidate_caches();

        if let Some(parttype) = node.parttype() {
            self.register_part(parttype);
        }

        debug_assert!(node.level() == self.nodes[parent].level() + 1);

        let node_index = self.nodes.insert(node);

        //All empty nodes need to be added to the sorted empty nodes list
        if is_empty {
            debug_assert!(self.nodes[node_index].is_empty());
            let node_area = self.nodes[node_index].area();
            let result = self.sorted_empty_nodes.binary_search_by(
                &(|n: &NodeKey| {
                    let n_area = self.nodes[*n].area();
                    n_area.cmp(&node_area).reverse()
                })
            );

            match result {
                Ok(i) => self.sorted_empty_nodes.insert(i, node_index),
                Err(i) => self.sorted_empty_nodes.insert(i, node_index),
            }
        }

        //Configure relationship between node and parent
        let previous_sibling = self.nodes[parent].last_child;
        self.nodes[node_index].parent = Some(parent);
        self.nodes[node_index].previous_sibling = previous_sibling;
        match previous_sibling {
            Some(previous_sibling) => self.nodes[previous_sibling].next_sibling = Some(node_index),
            None => self.nodes[parent].first_child = Some(node_index),
        }
        self.nodes[parent].last_child = Some(node_index);

        debug_assert!(assertions::node_arena_valid(&self.nodes, &self.top_node_i));
        node_index
    }

    fn unregister_node(&mut self, node_index: NodeKey, removed_part_ids: &mut Option<Vec<usize>>) {
        self.invalidate_caches();

        //All empty nodes need to be removed from the sorted empty nodes list
        if self.nodes[node_index].is_empty() {
            let node = &self.nodes[node_index];
            let lower_index = self.sorted_empty_nodes.partition_point(|n|
                { self.nodes[*n].area() > node.area() });

            if self.sorted_empty_nodes[lower_index] == node_index {
                //We have found the correct node, remove it
                self.sorted_empty_nodes.remove(lower_index);
            } else {
                let upper_index = self.sorted_empty_nodes.partition_point(|n|
                    { self.nodes[*n].area() >= node.area() });

                let mut node_found = false;
                for i in lower_index..upper_index {
                    if self.sorted_empty_nodes[i] == node_index {
                        //We have found the correct node, remove it
                        self.sorted_empty_nodes.remove(i);
                        node_found = true;
                        break;
                    }
                }
                if !node_found {
                    panic!("Empty node not found in sorted_empty_nodes");
                }
            }
        }

        //unregister all children
        while let Some(child) = self.nodes[node_index].first_child {
            self.unregister_node(child, removed_part_ids);
        }

        //remove the node
        let node = self.nodes.remove(node_index).expect("Node to be removed does not exist");
        debug_assert!(node.first_child.is_none() && node.last_child.is_none());

        //unregister part
        if let &Some(parttype) = node.parttype() {
            if let Some(removed_parts) = removed_part_ids {
                removed_parts.push(parttype.id());
            }
            self.unregister_part(parttype);
        }

        //break the relationship with parent
        if let Some(parent) = node.parent() {
            match node.previous_sibling {
                Some(previous_sibling) => self.nodes[previous_sibling].next_sibling = node.next_sibling,
                None => self.nodes[parent].first_child = node.next_sibling,
            }
            match node.next_sibling {
                Some(next_sibling) => self.nodes[next_sibling].previous_sibling = node.previous_sibling,
                None => self.nodes[parent].last_child = node.previous_sibling,
            }
        }

        debug_assert!(assertions::node_arena_valid(&self.nodes, &self.top_node_i));
    }

    fn register_part(&mut self, parttype: &PartType) {
        self.used_part_area += parttype.area();
    }

    fn unregister_part(&mut self, parttype: &PartType) {
        self.used_part_area -= parttype.area();
    }

    pub fn get_included_parts(&self) -> Vec<usize> {
        self.nodes.iter()
            .map(|(_, n)| n.parttype().map(|p| p.id()))
            .flatten()
            .collect_vec()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.iter().all(|(_, n)| n.is_empty())
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

    pub fn usage(&mut self, _force_recalc: bool) -> f64 {
        debug_assert!(assertions::cached_used_part_area_correct(&self.nodes, self.used_part_area));
        self.calculate_usage()
    }

    pub fn usage_immut(&self, _force_recalc: bool) -> f64 {
        debug_assert!(assertions::cached_used_part_area_correct(&self.nodes, self.used_part_area));
        self.calculate_usage()
    }

    pub fn sorted_empty_nodes(&self) -> &Vec<NodeKey> {
        debug_assert!(assertions::node_arena_valid(&self.nodes, &self.top_node_i), "{:#?}", self.sorted_empty_nodes.iter().map(|n| &self.nodes[*n]).collect_vec());
        debug_assert!(assertions::cached_sorted_empty_nodes_correct(&self.nodes(), &self.sorted_empty_nodes), "{:#?}", self.sorted_empty_nodes.iter().map(|n| &self.nodes[*n]).collect_vec());

        &self.sorted_empty_nodes
    }

    pub fn removable_nodes(&self) -> impl Iterator<Item = NodeKey> + '_ {
        //All nodes with children or that contain a part are removable
        self.nodes.iter()
            .filter(|(_, node)| node.parttype().is_some() || node.has_children())
            .map(|(index, _)| index)
    }

    pub fn sheettype(&self) -> &'a SheetType {
        self.sheettype
    }

    pub fn leftover_valuation_power(&self) -> f32 {
        self.leftover_valuation_power
    }

    pub fn top_node_index(&self) -> &NodeKey {
        &self.top_node_i
    }

    pub fn nodes(&self) -> &SlotMap<NodeKey, Node<'a>> {
        &self.nodes
    }

    pub fn id(&self) -> usize {
        self.id
    }
}
