use std::{rc::Rc};
use crate::core::cost::Cost;

use crate::{Orientation, PartType};
use crate::core::insertion::insertion_blueprint::InsertionBlueprint;
use crate::core::insertion::node_blueprint::NodeBlueprint;
use crate::core::rotation::Rotation;

pub struct Node {
    width : u64,
    height: u64,
    children : Vec<Rc<Node>>,
    parttype: Option<usize>,
    next_cut_orient : Orientation
}


impl Node {
    pub fn new(width: u64, height: u64, next_cut_orient: Orientation) -> Node {
        Node {
            width,
            height,
            children: Vec::new(),
            parttype: None,
            next_cut_orient
        }
    }

    pub fn new_from_blueprint(blueprint : &NodeBlueprint) -> Node {
        todo!();
    }

    pub fn create_deep_copy(&self) -> Node {
        todo!();
    }

    pub fn add_child(&mut self, child: Node) {
        todo!()
    }

    pub fn remove_child(&mut self, child: &Node) {
        todo!()
    }

    pub fn replace_child(&mut self, old_child: &Node, replacements: Vec<Node>) {
        todo!()
    }

    pub fn generate_insertion_blueprints<'a,'b>(&'a self, insertion_blueprints: &mut Vec<InsertionBlueprint<'a,'b>>, parttype : &'b PartType , rotation: Rotation) {
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

            let insertion_blueprint = InsertionBlueprint::new(self, vec![part_node, remainder_node], parttype);
            insertion_blueprints.push(insertion_blueprint);
            return;
        }
        if self.next_cut_orient == Orientation::Vertical && self.width == part_size.width() {
            let remainder_height = self.height - part_size.height();
            let part_node = NodeBlueprint::new(self.width, part_size.height(), Some(parttype), self.next_cut_orient);
            let remainder_node = NodeBlueprint::new(self.width, remainder_height, None, self.next_cut_orient);

            let insertion_blueprint = InsertionBlueprint::new(self, vec![part_node, remainder_node], parttype);
            insertion_blueprints.push(insertion_blueprint);
            return;
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

            let insertion_blueprint = InsertionBlueprint::new(self, vec![copy], parttype);
            insertion_blueprints.push(insertion_blueprint);

            return;
        }

        if self.next_cut_orient == Orientation::Vertical && self.height == part_size.height() {
            let mut copy = NodeBlueprint::new(self.width, self.height, None, self.next_cut_orient);

            let remainder_width = self.width - part_size.width();

            let part_node = NodeBlueprint::new(part_size.width(), self.height, Some(parttype), self.next_cut_orient.rotate());
            let remainder_node = NodeBlueprint::new(remainder_width, self.height, None, self.next_cut_orient.rotate());

            copy.add_child(part_node);
            copy.add_child(remainder_node);

            let insertion_blueprint = InsertionBlueprint::new(self, vec![copy], parttype);
            insertion_blueprints.push(insertion_blueprint);

            return;
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

            let insertion_blueprint = InsertionBlueprint::new(self, vec![part_node_parent, remainder_node_top], parttype);
            insertion_blueprints.push(insertion_blueprint);
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

            let insertion_blueprint = InsertionBlueprint::new(self, vec![part_node_parent, remainder_node_top], parttype);
            insertion_blueprints.push(insertion_blueprint);
        }

        /*
             Scenario 4.2: First cut in opposite of current orientation
             ---*****          ---*****             *       ->      *   *
                *   *             *$* *                            / \
                *   *     ->      *****                           *   *
                *   *             *   *                          / \
             ---*****          ---*****                         $   *

             This requires two extra available levels and is only allowed if NODE_MUST_HAVE_FULL_PART is disabled
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

            let insertion_blueprint = InsertionBlueprint::new(self, vec![copy], parttype);
            insertion_blueprints.push(insertion_blueprint);
        }

        if self.next_cut_orient == Orientation::Vertical{
            let mut copy = NodeBlueprint::new(self.width, self.height, None, self.next_cut_orient);

            let remainder_width_top = self.width - part_size.width();
            let mut part_node_parent = NodeBlueprint::new(part_size.width(), self.height, None, self.next_cut_orient.rotate());
            let remainder_node_top = NodeBlueprint::new(remainder_width_top, self.height, None, self.next_cut_orient.rotate().rotate());

            let remainder_height_bottom = self.height - part_size.height();

            let part_node = NodeBlueprint::new(part_size.width(), part_size.height(), Some(parttype), self.next_cut_orient.rotate().rotate());
            let remainder_node_bottom = NodeBlueprint::new(part_size.width(), remainder_height_bottom, None, self.next_cut_orient.rotate().rotate());

            part_node_parent.add_child(part_node);
            part_node_parent.add_child(remainder_node_bottom);

            copy.add_child(part_node_parent);
            copy.add_child(remainder_node_top);

            let insertion_blueprint = InsertionBlueprint::new(self, vec![copy], parttype);
            insertion_blueprints.push(insertion_blueprint);
        }
    }

    pub fn insertion_possible(&self, parttype : &PartType, rotation : Rotation) -> bool {
        debug_assert!(*parttype.fixed_rotation() == None || *parttype.fixed_rotation() == Some(rotation));
        debug_assert!(self.children.is_empty() && self.parttype.is_none());

        let part_size = match rotation {
            Rotation::Default => parttype.size(),
            Rotation::Rotated => parttype.rotated_size()
        };

        self.width >= part_size.width() && self.height >= part_size.height()
    }

    pub fn get_cost() -> Cost {
        todo!()
    }


    pub fn width(&self) -> u64 {
        self.width
    }
    pub fn height(&self) -> u64 {
        self.height
    }
    pub fn children(&self) -> &Vec<Rc<Node>> {
        &self.children
    }
    pub fn parttype(&self) -> Option<usize> {
        self.parttype
    }
    pub fn next_cut_orient(&self) -> Orientation {
        self.next_cut_orient
    }

    pub fn area(&self) -> u64 {
        self.width * self.height
    }
}



