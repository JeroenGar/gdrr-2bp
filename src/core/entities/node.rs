use std::{rc::Rc};
use crate::core::cost::Cost;

use crate::{Orientation, PartType};
use crate::core::insertion::insertion_blueprint::InsertionBlueprint;
use crate::core::insertion::node_blueprint::NodeBlueprint;
use crate::core::rotation::Rotation;
use crate::Orientation::Vertical;

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

    pub fn generate_insertion_blueprints(&self, parttype : &PartType, rotation: Rotation) -> Vec<InsertionBlueprint> {
        let mut insertion_blueprints = Vec::new();
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
            let part_node = NodeBlueprint::new(part_size.width(), self.height, Some(parttype.id()), self.next_cut_orient);
            let remainder_node = NodeBlueprint::new(remainder_width, self.height, None, self.next_cut_orient);

            let insertion_blueprint = InsertionBlueprint::new(self, vec![part_node, remainder_node], parttype.id());
            insertion_blueprints.push(insertion_blueprint);
            return insertion_blueprints;
        }
        if self.next_cut_orient == Orientation::Vertical && self.width == part_size.width() {
            let remainder_height = self.height - part_size.height();
            let part_node = NodeBlueprint::new(self.width, part_size.height(), Some(parttype.id()), self.next_cut_orient);
            let remainder_node = NodeBlueprint::new(self.width, remainder_height, None, self.next_cut_orient);

            let insertion_blueprint = InsertionBlueprint::new(self, vec![part_node, remainder_node], parttype.id());
            insertion_blueprints.push(insertion_blueprint);
            return insertion_blueprints;
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

            let part_node = NodeBlueprint::new(self.width, part_size.height(), Some(parttype.id()), self.next_cut_orient.rotate());
            let remainder_node = NodeBlueprint::new(self.width, remainder_height, None, self.next_cut_orient.rotate());

            copy.add_child(part_node);
            copy.add_child(remainder_node);

            let insertion_blueprint = InsertionBlueprint::new(self, vec![copy], parttype.id());
            insertion_blueprints.push(insertion_blueprint);
            return insertion_blueprints;
        }

        if self.next_cut_orient == Vertical && self.height == part_size.height() {
            let mut copy = NodeBlueprint::new(self.width, self.height, None, self.next_cut_orient);

            let remainder_width = self.width - part_size.width();

            let part_node = NodeBlueprint::new(part_size.width(), self.height, Some(parttype.id()), self.next_cut_orient.rotate());
            let remainder_node = NodeBlueprint::new(remainder_width, self.height, None, self.next_cut_orient.rotate());

            copy.add_child(part_node);
            copy.add_child(remainder_node);

            let insertion_blueprint = InsertionBlueprint::new(self, vec![copy], parttype.id());
            insertion_blueprints.push(insertion_blueprint);
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

        todo!();

        /*
             Scenario 4.2: First cut in opposite of current orientation
             ---*****          ---*****             *       ->      *   *
                *   *             *$* *                            / \
                *   *     ->      *****                           *   *
                *   *             *   *                          / \
             ---*****          ---*****                         $   *

             This requires two extra available levels and is only allowed if NODE_MUST_HAVE_FULL_PART is disabled
         */

        todo!();

        insertion_blueprints
    }

    pub fn insertion_possible() -> bool {
        todo!()
    }

    pub fn get_cost() -> Cost {
        todo!()
    }



}



