use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use std::rc::Weak;
use generational_arena::{Arena, Index};

use crate::{Instance, Orientation, PartType};
use crate::core::cost::Cost;
use crate::core::insertion::insertion_blueprint::InsertionBlueprint;
use crate::core::insertion::node_blueprint::NodeBlueprint;
use crate::core::leftover_valuator;
use crate::core::rotation::Rotation;
use crate::util::assertions;
use crate::util::macros::{rb, rbm};

#[derive(Debug, Clone)]
pub struct Node<'a> {
    width: u64,
    height: u64,
    children: Vec<Index>,
    parent: Option<Index>,
    parttype: Option<&'a PartType>,
    next_cut_orient: Orientation,
}


impl<'a> Node<'a> {
    pub fn new(width: u64, height: u64, next_cut_orient: Orientation, parttype: Option<&'a PartType>) -> Node<'a> {
        Node {
            width,
            height,
            children: vec![],
            parent: None,
            parttype,
            next_cut_orient,
        }
    }

    pub fn set_parent(&mut self, parent: Index){
        self.parent = Some(parent);
    }

    pub fn add_child(&mut self, child: Index) {
        self.children.push(child);
        //TODO: debug_assert!(assertions::children_nodes_fit(node_ref.deref()))
    }

    pub fn remove_child(&mut self, old_child: Index) {
        let old_child_index = self.children.iter().position(|c| *c == old_child).expect("Child not found");
        self.children.remove(old_child_index);
    }


    pub fn generate_insertion_node_blueprints(&self, parttype: &'a PartType, rotation: Rotation, mut insertion_replacements: Vec<Vec<NodeBlueprint>>) -> Vec<Vec<NodeBlueprint>> {
        debug_assert!(self.insertion_possible(parttype, rotation));

        let part_size = match rotation {
            Rotation::Default => parttype.size(),
            Rotation::Rotated => parttype.rotated_size()
        };

        /*
             Scenario 1: Part fits exactly into Node
             ---*****          ---*****             *       ->      *
                *   *             *$$$*
                *   *     ->      *$$$*
                *   *             *$$$*
             ---*****          ---*****

             -> node gets replaced by one node on same level
             -> = Scenario 2
         */

        /*
            Scenario 2: Part has same dimensions in the direction of the current cut
             ---*****          ---*****             *       ->      $   *
                *   *             *$* *
                *   *     ->      *$* *
                *   *             *$* *
             ---*****          ---*****

             -> node splits into 2 new nodes on same level
         */


        if self.next_cut_orient == Orientation::Horizontal && self.height == part_size.height() {
            let remainder_width = self.width - part_size.width();
            let part_node = NodeBlueprint::new(part_size.width(), self.height, Some(parttype), self.next_cut_orient);
            let remainder_node = NodeBlueprint::new(remainder_width, self.height, None, self.next_cut_orient);

            insertion_replacements.push(vec![part_node, remainder_node]);
            return insertion_replacements;
        }
        if self.next_cut_orient == Orientation::Vertical && self.width == part_size.width() {
            let remainder_height = self.height - part_size.height();
            let part_node = NodeBlueprint::new(self.width, part_size.height(), Some(parttype), self.next_cut_orient);
            let remainder_node = NodeBlueprint::new(self.width, remainder_height, None, self.next_cut_orient);

            insertion_replacements.push(vec![part_node, remainder_node]);
            return insertion_replacements;
        }

        /*
             Scenario 3: Part fits exactly in opposite dimension of cut
             ---*****          ---*****             *       ->      *    *
                *   *             *$$$*                            / \
                *   *     ->      *****                           $   *
                *   *             *   *
             ---*****          ---*****
         */

        if self.next_cut_orient == Orientation::Horizontal && self.width == part_size.width() {
            let mut copy = NodeBlueprint::new(self.width, self.height, None, self.next_cut_orient);

            let remainder_height = self.height - part_size.height();

            let part_node = NodeBlueprint::new(self.width, part_size.height(), Some(parttype), self.next_cut_orient.rotate());
            let remainder_node = NodeBlueprint::new(self.width, remainder_height, None, self.next_cut_orient.rotate());

            copy.add_child(part_node);
            copy.add_child(remainder_node);

            insertion_replacements.push(vec![copy]);
            return insertion_replacements;
        }

        if self.next_cut_orient == Orientation::Vertical && self.height == part_size.height() {
            let mut copy = NodeBlueprint::new(self.width, self.height, None, self.next_cut_orient);

            let remainder_width = self.width - part_size.width();

            let part_node = NodeBlueprint::new(part_size.width(), self.height, Some(parttype), self.next_cut_orient.rotate());
            let remainder_node = NodeBlueprint::new(remainder_width, self.height, None, self.next_cut_orient.rotate());

            copy.add_child(part_node);
            copy.add_child(remainder_node);

            insertion_replacements.push(vec![copy]);

            return insertion_replacements;
        }

        /*
             Scenario 4: Part doesn't fit exactly in any dimension

             Scenario 4.1: First cut in same direction as current orientation
             ---*****          ---*****             *       ->      *   *
                *   *             *$* *                            / \
                *   *     ->      *** *                           $   *
                *   *             * * *
             ---*****          ---*****

             This requires an extra available level
         */

        if self.next_cut_orient == Orientation::Horizontal {
            let remainder_width_top = self.width - part_size.width();
            let mut part_node_parent = NodeBlueprint::new(part_size.width(), self.height, None, self.next_cut_orient);
            let remainder_node_top = NodeBlueprint::new(remainder_width_top, self.height, None, self.next_cut_orient);

            let remainder_height_bottom = self.height - part_size.height();
            let part_node = NodeBlueprint::new(part_size.width(), part_size.height(), Some(parttype), self.next_cut_orient.rotate());
            let remainder_node_bottom = NodeBlueprint::new(part_size.width(), remainder_height_bottom, None, self.next_cut_orient.rotate());

            part_node_parent.add_child(part_node);
            part_node_parent.add_child(remainder_node_bottom);

            insertion_replacements.push(vec![part_node_parent, remainder_node_top]);
        }

        if self.next_cut_orient == Orientation::Vertical {
            let remainder_height_top = self.height - part_size.height();
            let mut part_node_parent = NodeBlueprint::new(self.width, part_size.height(), None, self.next_cut_orient);
            let remainder_node_top = NodeBlueprint::new(self.width, remainder_height_top, None, self.next_cut_orient);

            let remainder_width_bottom = self.width - part_size.width();
            let part_node = NodeBlueprint::new(part_size.width(), part_size.height(), Some(parttype), self.next_cut_orient.rotate());
            let remainder_node_bottom = NodeBlueprint::new(remainder_width_bottom, part_size.height(), None, self.next_cut_orient.rotate());

            part_node_parent.add_child(part_node);
            part_node_parent.add_child(remainder_node_bottom);

            insertion_replacements.push(vec![part_node_parent, remainder_node_top]);
        }

        /*
             Scenario 4.2: First cut in opposite of current orientation
             ---*****          ---*****             *       ->      *   *
                *   *             *$* *                            / \
                *   *     ->      *****                           *   *
                *   *             *   *                          / \
             ---*****          ---*****                         $   *

         */

        if self.next_cut_orient == Orientation::Horizontal {
            let mut copy = NodeBlueprint::new(self.width, self.height, None, self.next_cut_orient);

            let remainder_height_top = self.height - part_size.height();
            let mut part_node_parent = NodeBlueprint::new(self.width, part_size.height(), None, self.next_cut_orient.rotate());
            let remainder_node_top = NodeBlueprint::new(self.width, remainder_height_top, None, self.next_cut_orient.rotate());

            let remainder_width_bottom = self.width - part_size.width();
            let part_node = NodeBlueprint::new(part_size.width(), part_size.height(), Some(parttype), self.next_cut_orient.rotate().rotate());
            let remainder_node_bottom = NodeBlueprint::new(remainder_width_bottom, part_size.height(), None, self.next_cut_orient.rotate().rotate());

            part_node_parent.add_child(part_node);
            part_node_parent.add_child(remainder_node_bottom);

            copy.add_child(part_node_parent);
            copy.add_child(remainder_node_top);

            insertion_replacements.push(vec![copy]);
        }

        if self.next_cut_orient == Orientation::Vertical {
            let mut copy = NodeBlueprint::new(self.width, self.height, None, self.next_cut_orient);

            let remainder_width_top = self.width - part_size.width();
            let mut part_node_parent = NodeBlueprint::new(part_size.width(), self.height, None, self.next_cut_orient.rotate());
            let remainder_node_top = NodeBlueprint::new(remainder_width_top, self.height, None, self.next_cut_orient.rotate());

            let remainder_height_bottom = self.height - part_size.height();

            let part_node = NodeBlueprint::new(part_size.width(), part_size.height(), Some(parttype), self.next_cut_orient.rotate().rotate());
            let remainder_node_bottom = NodeBlueprint::new(part_size.width(), remainder_height_bottom, None, self.next_cut_orient.rotate().rotate());

            part_node_parent.add_child(part_node);
            part_node_parent.add_child(remainder_node_bottom);

            copy.add_child(part_node_parent);
            copy.add_child(remainder_node_top);

            insertion_replacements.push(vec![copy]);
        }
        return insertion_replacements;
    }

    pub fn insertion_possible(&self, parttype: &PartType, rotation: Rotation) -> bool {
        debug_assert!(*parttype.fixed_rotation() == None || *parttype.fixed_rotation() == Some(rotation));
        debug_assert!(self.children.is_empty() && self.parttype.is_none());

        let part_size = match rotation {
            Rotation::Default => parttype.size(),
            Rotation::Rotated => parttype.rotated_size()
        };

        self.width >= part_size.width() && self.height >= part_size.height()
    }

    pub fn calculate_cost(&self) -> Cost {
        match (self.parttype, self.children.is_empty()) {
            (Some(_), true) => Cost::empty(), // part-node
            (None, false) => Cost::empty(), // structure-node
            (None, true) => Cost::empty().add_leftover_value(leftover_valuator::valuate(self.area())), //leftover node
            (Some(_), false) => panic!("Parttype set on node with children"),
        }
    }

    pub fn calculate_usage(&self) -> f64 {
        if self.parttype.is_some() {
            1.0
        } else if self.children.is_empty() {
            0.0
        } else {
            let mut usage = 0.0;
            for child in &self.children {
                let child_ref = rb!(child);
                usage += child_ref.area() as f64 * child_ref.calculate_usage();
            }
            usage /= self.area() as f64;
            debug_assert!(usage <= 1.0);
            usage
        }
    }

    pub fn is_empty(&self) -> bool {
        self.parttype.is_none() && self.children.is_empty()
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
    pub fn children(&self) -> &Vec<Index> {
        &self.children
    }
    pub fn parent(&self) -> &Option<Index> {
        &self.parent
    }
}



