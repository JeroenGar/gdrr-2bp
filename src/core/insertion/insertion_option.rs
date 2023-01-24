use std::cell::{RefCell};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::rc::Weak;

use by_address::ByAddress;
use generational_arena::{Arena, Index};
use itertools::Itertools;

use crate::{PartType, Rotation};
use crate::core::cost::Cost;
use crate::core::entities::layout::Layout;
use crate::core::entities::node::Node;
use crate::core::insertion::insertion_blueprint::InsertionBlueprint;
use crate::core::layout_index::LayoutIndex;
use crate::optimization::problem::Problem;

#[derive(Debug, PartialEq, Eq)]
pub struct InsertionOption<'a> {
    layout_i: LayoutIndex,
    original_node_i: Index,
    parttype: &'a PartType,
    rotation: Option<Rotation>,
}

//TODO: cache the blueprints

impl<'a> InsertionOption<'a> {
    pub fn new(layout_i: LayoutIndex, original_node_i: Index, parttype: &'a PartType, rotation: Option<Rotation>) -> Self {
        //TODO: fix this
        //debug_assert!(rb!(original_node.upgrade().unwrap()).parent().is_some(), "{:#?}", original_node.upgrade().unwrap());
        Self {
            layout_i,
            original_node_i,
            parttype,
            rotation
        }
    }

    pub fn generate_blueprints(&self, problem: &Problem) -> Vec<InsertionBlueprint<'a>> {
        let layout = problem.get_layout(&self.layout_i);
        let original_node = &layout.nodes()[self.original_node_i];
        let node_blueprints = match self.rotation {
            Some(rotation) => {
                original_node.generate_insertion_node_blueprints(self.parttype, rotation,  vec![])
            }
            None => {
                let mut node_blueprints = original_node.generate_insertion_node_blueprints(self.parttype, Rotation::Default, vec![]);
                original_node.generate_insertion_node_blueprints(self.parttype, Rotation::Rotated, node_blueprints)
            }
        };
        let original_cost = original_node.calculate_cost();

        //Convert the node blueprints into insertion blueprints
        node_blueprints.into_iter().map(|nbs| {
            let new_cost = nbs.iter().map(|replacement| replacement.calculate_cost()).sum::<Cost>();
            let insertion_cost = new_cost.subtract(&original_cost);
            InsertionBlueprint::new(self.layout_i, self.original_node_i, nbs, self.parttype, insertion_cost)
        }).collect_vec()
    }

    pub fn parttype(&self) -> &'a PartType {
        self.parttype
    }

    pub fn rotation(&self) -> Option<Rotation> {
        self.rotation
    }

    pub fn original_node_index(&self) -> &Index {
        &self.original_node_i
    }

    pub fn layout_index(&self) -> &LayoutIndex {
        &self.layout_i
    }
}