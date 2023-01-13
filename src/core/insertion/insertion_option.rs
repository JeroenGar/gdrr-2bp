use std::cell::{RefCell};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::rc::Weak;

use by_address::ByAddress;
use generational_arena::{Arena, Index};
use itertools::Itertools;

use crate::{PartType, Rotation};
use crate::core::entities::layout::Layout;
use crate::core::entities::node::Node;
use crate::core::insertion::insertion_blueprint::InsertionBlueprint;
use crate::core::layout_index::LayoutIndex;
use crate::util::macros::{rb};

pub struct InsertionOption<'a> {
    layout: LayoutIndex,
    original_node: Index,
    parttype: &'a PartType,
    rotation: Option<Rotation>,
}

//TODO: cache the blueprints

impl<'a> InsertionOption<'a> {
    pub fn new(layout: LayoutIndex, original_node: Index, parttype: &'a PartType, rotation: Option<Rotation>) -> Self {
        //TODO: fix this
        //debug_assert!(rb!(original_node.upgrade().unwrap()).parent().is_some(), "{:#?}", original_node.upgrade().unwrap());
        Self {
            layout,
            original_node,
            parttype,
            rotation
        }
    }

    pub fn generate_blueprints(&self, layout: &Layout) -> Vec<InsertionBlueprint<'a>> {
        let original_node = &layout.nodes()[self.original_node];
        let node_blueprints = match self.rotation {
            Some(rotation) => {
                original_node.generate_insertion_node_blueprints(self.parttype, rotation,  vec![])
            }
            None => {
                let mut node_blueprints = original_node.generate_insertion_node_blueprints(self.parttype, Rotation::Default, vec![]);
                original_node.generate_insertion_node_blueprints(self.parttype, Rotation::Rotated, node_blueprints);
            }
        };
        let original_cost = original_node.calculate_cost();

        //Convert the node blueprints into insertion blueprints
        node_blueprints.into_iter().map(|nbs| {
            let new_cost = nbs.iter().map(|replacement| replacement.calculate_cost()).sum();
            let insertion_cost = new_cost - original_cost;
            InsertionBlueprint::new(self.layout, self.original_node, nbs, self.parttype, insertion_cost)
        }).collect_vec()
    }

    pub fn parttype(&self) -> &'a PartType {
        self.parttype
    }

    pub fn rotation(&self) -> Option<Rotation> {
        self.rotation
    }

    pub fn original_node(&self) -> &Index {
        &self.original_node
    }

    pub fn layout(&self) -> &LayoutIndex {
        &self.layout
    }
}

impl Hash for InsertionOption<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.original_node().hash(state);
        self.parttype().hash(state);
        self.rotation().hash(state);
    }
}

impl PartialEq for InsertionOption<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.original_node() == other.original_node() &&
            self.parttype() == other.parttype() &&
            self.rotation() == other.rotation()
    }
}

impl Eq for InsertionOption<'_> {}

impl<'a> Debug for InsertionOption<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "InsertionOption {{ original_node: {:?}, parttype: {:?}, rotation: {:?}, layout: {:?}}}", self.original_node().upgrade().unwrap(), self.parttype(), self.rotation(), self.layout().upgrade().unwrap().borrow().id())
    }
}