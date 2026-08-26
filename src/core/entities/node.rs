use slotmap::{new_key_type, SlotMap};

use crate::core::cost::Cost;
use crate::core::entities::parttype::PartType;
use crate::core::insertion::insertion_blueprint::InsertionShape;
use crate::core::leftover_valuator;
use crate::core::orientation::Orientation;
use crate::core::rotation::Rotation;

new_key_type! {
    pub struct NodeKey;
}

#[derive(Debug, Clone)]
pub struct Node<'a> {
    level: u8,
    width: u64,
    height: u64,
    pub(super) parent: Option<NodeKey>,
    pub(super) first_child: Option<NodeKey>,
    pub(super) last_child: Option<NodeKey>,
    pub(super) previous_sibling: Option<NodeKey>,
    pub(super) next_sibling: Option<NodeKey>,
    parttype: Option<&'a PartType>,
    next_cut_orient: Orientation,
}


impl<'a> Node<'a> {
    pub fn new(level: u8, width: u64, height: u64, next_cut_orient: Orientation, parttype: Option<&'a PartType>) -> Node<'a> {
        Node {
            level,
            width,
            height,
            parent: None,
            first_child: None,
            last_child: None,
            previous_sibling: None,
            next_sibling: None,
            parttype,
            next_cut_orient,
        }
    }

    pub(crate) fn for_each_insertion_shape(
        &self,
        parttype: &PartType,
        rotation: Rotation,
        max_level: u8,
        emit: &mut impl FnMut(InsertionShape),
    ) {
        debug_assert!(self.insertion_possible(parttype, rotation));

        let part_size = match rotation {
            Rotation::Default => parttype.size(),
            Rotation::Rotated => parttype.rotated_size(),
        };
        let fits_along_current_cut = match self.next_cut_orient {
            Orientation::Horizontal => self.height == part_size.height(),
            Orientation::Vertical => self.width == part_size.width(),
        };
        if fits_along_current_cut {
            emit(InsertionShape::AlongCurrentCut);
            return;
        }

        let fits_across_current_cut = match self.next_cut_orient {
            Orientation::Horizontal => self.width == part_size.width(),
            Orientation::Vertical => self.height == part_size.height(),
        };
        if fits_across_current_cut && self.level < max_level {
            emit(InsertionShape::AcrossCurrentCut);
            return;
        }

        if self.level < max_level {
            emit(InsertionShape::AlongThenAcross);
        }
        if self.level + 1 < max_level {
            emit(InsertionShape::AcrossThenAlong);
        }
    }

    pub fn insertion_possible(&self, parttype: &PartType, rotation: Rotation) -> bool {
        debug_assert!(*parttype.fixed_rotation() == None || *parttype.fixed_rotation() == Some(rotation));
        debug_assert!(!self.has_children() && self.parttype.is_none());

        let part_size = match rotation {
            Rotation::Default => parttype.size(),
            Rotation::Rotated => parttype.rotated_size()
        };

        self.width >= part_size.width() && self.height >= part_size.height()
    }

    pub fn calculate_cost(&self, leftover_valuation_power: f32) -> Cost {
        match (self.parttype, self.has_children()) {
            (Some(_), false) => Cost::empty(), // part-node
            (None, true) => Cost::empty(), // structure-node
            (None, false) => Cost::empty().add_leftover_value(leftover_valuator::valuate(
                self.area(),
                leftover_valuation_power,
            )), //leftover node
            (Some(_), true) => panic!("Parttype set on node with children"),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.parttype.is_none() && !self.has_children()
    }

    pub fn width(&self) -> u64 {
        self.width
    }
    pub fn height(&self) -> u64 {
        self.height
    }
    pub fn parttype(&self) -> &Option<&PartType> {
        &self.parttype
    }
    pub fn next_cut_orient(&self) -> Orientation {
        self.next_cut_orient
    }
    pub fn area(&self) -> u64 {
        self.width * self.height
    }
    pub fn has_children(&self) -> bool {
        self.first_child.is_some()
    }
    pub fn children<'n>(&self, nodes: &'n SlotMap<NodeKey, Node<'a>>) -> impl Iterator<Item = NodeKey> + 'n {
        let mut next_child = self.first_child;
        std::iter::from_fn(move || {
            let child = next_child?;
            next_child = nodes[child].next_sibling;
            Some(child)
        })
    }
    pub fn parent(&self) -> Option<NodeKey> {
        self.parent
    }
    pub(crate) fn first_child(&self) -> Option<NodeKey> {
        self.first_child
    }
    pub(crate) fn last_child(&self) -> Option<NodeKey> {
        self.last_child
    }
    pub(crate) fn previous_sibling(&self) -> Option<NodeKey> {
        self.previous_sibling
    }
    pub(crate) fn next_sibling(&self) -> Option<NodeKey> {
        self.next_sibling
    }
    pub fn level(&self) -> u8 {
        self.level
    }
}
