use generational_arena::{Arena, Index};
use itertools::Itertools;

use crate::{Instance, Orientation};
use crate::core::{cost::Cost, insertion::insertion_blueprint::InsertionBlueprint};
use crate::core::entities::node::Node;
use crate::core::insertion::node_blueprint::NodeBlueprint;
use crate::optimization::rr::cache_updates::CacheUpdates;
use crate::util::assertions;

use super::{parttype::PartType, sheettype::SheetType};

#[derive(Debug, Clone)]
pub struct Layout<'a> {
    sheettype: &'a SheetType,
    nodes: Arena<Node<'a>>,
    top_node: Index,
    cached_cost: Option<Cost>,
    cached_usage: Option<f64>,
    sorted_empty_nodes: Vec<Index>, //sorted by descending area
}

impl<'a> Layout<'a> {
    pub fn new(sheettype: &'a SheetType, first_cut_orientation: Orientation) -> Self {
        let mut nodes = Arena::new();
        let top_node = nodes.insert(Node::new(sheettype.width(), sheettype.height(), first_cut_orientation, None));

        let mut layout = Self {
            sheettype,
            nodes,
            top_node,
            cached_cost: None,
            cached_usage: None,
            sorted_empty_nodes: vec![],
        };

        let placeholder_node = Node::new(sheettype.width(), sheettype.height(), first_cut_orientation.rotate(), None);
        layout.register_node(placeholder_node, top_node);

        layout
    }

    pub fn implement_insertion_blueprint(&mut self, blueprint: &InsertionBlueprint<'a>, cache_updates: &mut CacheUpdates<'a, Index>, instance: &'a Instance) {
        let original = blueprint.original_node();
        let parent = self.nodes[original].parent().expect("original node has no parent");

        //unregister the original node
        self.unregister_node(original, None);
        cache_updates.add_invalidated(original);

        //create and register the replacements
        let mut all_created_nodes = vec![];
        for replacement in blueprint.replacements() {
            self.implement_node_blueprint(parent, replacement, &mut all_created_nodes, instance);
        }
        cache_updates.extend_new(all_created_nodes);


        //TODO: fix this
        //debug_assert!(assertions::children_nodes_fit(rb!(parent_node).deref()), "{:#?}", blueprint);
    }

    fn implement_node_blueprint(&mut self, parent: Index, blueprint: &NodeBlueprint, all_created_nodes: &mut Vec<Index>, instance: &'a Instance) {
        let parttype = blueprint.parttype_id().map(|id| instance.get_parttype(id));

        let node = Node::new(blueprint.width(), blueprint.height(), blueprint.orientation(), parttype);
        let node_index = self.register_node(node, parent);

        all_created_nodes.push(node_index);

        for child_blueprint in blueprint.children() {
            self.implement_node_blueprint(node_index, child_blueprint, all_created_nodes, instance);
        }
    }

    pub fn remove_node(&mut self, node_index: Index) -> Vec<PartType>{
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

        let parent_node_index = self.nodes[node_index].parent().expect("Cannot remove a node without a parent");
        let parent_node = &mut self.nodes[parent_node_index];

        //Check if there is an empty_node present
        let empty_node = parent_node.children().iter().find(|child: &Index| { self.nodes[*child].is_empty()});

        let mut removed_parts = vec![];

        match empty_node {
            Some(&empty_node_index) => {
                //Scenario 1 and 3
                if parent_node.children().len() > 1 || parent_node.parent().is_none() {
                    //Scenario 1 (also do this when the parent node is the root)
                    //Two children are merged into one

                    let node = &self.nodes[node_index];
                    let empty_node = &self.nodes[empty_node_index];
                    let replacement_node = match parent_node.next_cut_orient() {
                        Orientation::Horizontal => {
                            let new_height = empty_node.height() + node.height();
                            Node::new(node.width(), new_height, node.next_cut_orient(), None)
                        }
                        Orientation::Vertical => {
                            let new_width = empty_node.width() + node.width();
                            Node::new(new_width, node.height(), node.next_cut_orient(), None)
                        }
                    };

                    //Replace the empty node and the node to be removed with a enlarged empty node
                    self.unregister_node(empty_node_index, Some(&mut removed_parts));
                    self.unregister_node(node_index, Some(&mut removed_parts));
                    self.register_node(replacement_node, parent_node_index);
                } else {
                    //Scenario 3: replace the parent with an empty node
                    let grandparent_index = parent_node.parent().expect("grandparent node needs to be present").clone();

                    //create empty parent
                    let empty_parent_node = Node::new(parent_node.width(), parent_node.height(), parent_node.next_cut_orient(), None);

                    //replace
                    self.unregister_node(parent_node_index, Some(&mut removed_parts));
                    self.register_node(empty_parent_node, grandparent_index);
                }
            }
            None => {
                //Scenario 2: convert the node itself into an empty node

                //create empty replacement node
                let node = &self.nodes[node_index];
                let replacement_node = Node::new(node.width(), node.height(), node.next_cut_orient(), None);

                //replace
                self.unregister_node(node_index, Some(&mut removed_parts));
                self.register_node(replacement_node, parent_node_index);
            }
        }
        removed_parts
    }

    fn invalidate_caches(&mut self) {
        self.cached_cost = None;
        self.cached_usage = None;
    }

    fn calculate_cost(&self) -> Cost {
        let material_cost = Cost::empty().add_material_cost(self.sheettype.value());
        self.nodes.iter()
            .map(|(_, node)| node.calculate_cost())
            .fold(material_cost, |acc, cost| acc.add(&cost))
    }

    fn calculate_usage(&self) -> f64 {
        let used_area = self.nodes.iter().map(|(index, node)| {
            node.parttype().map(|| node.area()).unwrap_or(0)
        }).sum();

        used_area as f64 / self.sheettype.area() as f64
    }

    fn register_node(&mut self, mut node: Node, parent: Index) -> Index {
        self.invalidate_caches();

        if let Some(parttype) = node.parttype() {
            self.register_part(parttype);
        }

        let node_index = self.nodes.insert(node);

        //All empty nodes need to be added to the sorted empty nodes list
        if self.nodes[node_index].is_empty() {
            let node_area = self.nodes[node_index].area();
            let result = self.sorted_empty_nodes.binary_search_by(
                &(|n: &Index| {
                    let n_area = self.nodes[*n].area();
                    n_area.cmp(&node_area).reverse()
                })
            );

            match result {
                Ok(i) => self.sorted_empty_nodes.insert(i, node_index),
                Err(i) => self.sorted_empty_nodes.insert(i, node_index),
            }
            //TODO: sort these assertions out
            //debug_assert!(assertions::nodes_sorted_descending_area(&self.sorted_empty_nodes));
            //debug_assert!(assertions::all_nodes_have_parents(&self.sorted_empty_nodes), "{:#?}", self.sorted_empty_nodes.iter().map(|n| n.upgrade().unwrap()).collect::<Vec<_>>());
        }

        //Configure relationship between node and parent
        node.set_parent(parent);
        self.nodes[parent].add_child(node_index);

        debug_assert!(assertions::no_ghost_nodes_in_arena(&self.nodes, &self.top_node));
        node_index
    }

    fn unregister_node(&mut self, node_index: Index, removed_parts: Option<&mut Vec<PartType>>) {
        self.invalidate_caches();

        if let Some(parttype) = node.parttype() {
            if let Some(removed_parts) = removed_parts {
                removed_parts.push(parttype);
            }
            self.unregister_part(parttype);
        }

        //All empty nodes need to be removed from the sorted empty nodes list
        let node = self.nodes.remove(node_index).expect("Node to be removed does not exist");
        if node.is_empty() {
            let lower_index = self.sorted_empty_nodes.partition_point(|n|
                { self.nodes[n].area() > node.area() });

            if self.sorted_empty_nodes[lower_index] == node_index {
                //We have found the correct node, remove it
                self.sorted_empty_nodes.remove(lower_index);
            } else {
                let upper_index = self.sorted_empty_nodes.partition_point(|n|
                    { self.nodes[n].area() >= node.area() });

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

        for child in node.children() {
            self.unregister_node(child.clone(), removed_parts);
        }

        //break the relationship with parent
        if let Some(parent) = self.nodes[node_index].parent() {
            self.nodes[parent].remove_child(node_index);
        }

        debug_assert!(assertions::no_ghost_nodes_in_arena(&self.nodes, &self.top_node));
    }

    fn register_part(&mut self, _parttype: &PartType) {
        self.invalidate_caches();
    }

    fn unregister_part(&mut self, _parttype: &PartType) {
        self.invalidate_caches();
    }

    pub fn get_included_parts(&self) -> Vec<&'a PartType> {
        self.nodes.iter()
            .map(|(_, n)| n.parttype().clone())
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

    pub fn usage(&mut self, force_recalc: bool) -> f64 {
        let usage = match (self.cached_usage.as_ref(), force_recalc) {
            (Some(usage), false) => *usage,
            _ => {
                let usage = self.calculate_usage();
                self.cached_usage = Some(usage);
                usage
            }
        };
        debug_assert!(force_recalc || usage == self.usage(true));
        usage
    }

    pub fn sorted_empty_nodes(&self) -> &Vec<Index> {
        //TODO: fix
        //debug_assert!(assertions::all_nodes_have_parents(&self.sorted_empty_nodes), "{:#?}", self.sorted_empty_nodes.iter().map(|n| n.upgrade().unwrap()).collect::<Vec<_>>());
        //debug_assert!(assertions::cached_empty_nodes_correct(self, &self.sorted_empty_nodes));
        //debug_assert!(assertions::nodes_sorted_descending_area(&self.sorted_empty_nodes), "{:#?}", self.sorted_empty_nodes.iter().map(|n| rb!(n.upgrade().unwrap()).area()).collect::<Vec<_>>());

        &self.sorted_empty_nodes
    }

    pub fn get_removable_nodes(&self) -> Vec<Index> {
        //All nodes with children (except the top node) or that contain a part are removable
        self.nodes.iter()
            .filter(|(index, _)| index != self.top_node)
            .filter(|(_, node)| node.parttype().is_some() || !node.children().is_empty())
            .map(|(index, _)| index)
            .collect_vec()
    }

    pub fn sheettype(&self) -> &'a SheetType {
        self.sheettype
    }

    pub fn top_node(&self) -> &Index {
        &self.top_node
    }

    pub fn nodes(&self) -> &Arena<Node> {
        &self.nodes
    }
}