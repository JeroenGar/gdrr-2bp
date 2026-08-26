use slotmap::new_key_type;

use crate::core::cost::Cost;
use crate::core::entities::parttype::PartType;
use crate::core::insertion::node_blueprint::NodeBlueprint;
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
    children: Vec<NodeKey>,
    parent: Option<NodeKey>,
    parttype: Option<&'a PartType>,
    next_cut_orient: Orientation,
}


impl<'a> Node<'a> {
    pub fn new(level: u8, width: u64, height: u64, next_cut_orient: Orientation, parttype: Option<&'a PartType>) -> Node<'a> {
        Node {
            level,
            width,
            height,
            children: vec![],
            parent: None,
            parttype,
            next_cut_orient,
        }
    }

    pub fn set_parent(&mut self, parent: NodeKey){
        self.parent = Some(parent);
    }

    pub fn add_child(&mut self, child: NodeKey) {
        self.children.push(child);
    }

    pub fn remove_child(&mut self, old_child: NodeKey) {
        let old_child_index = self.children.iter().position(|c| *c == old_child).expect("Child not found");
        self.children.remove(old_child_index);
    }

    pub fn for_each_insertion_replacement(
        &self,
        parttype: &'a PartType,
        rotation: Rotation,
        max_level: u8,
        emit: &mut impl FnMut(Vec<NodeBlueprint>),
    ) {
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

            emit(vec![part_node, remainder_node]);
            return;
        }
        if self.next_cut_orient == Orientation::Vertical && self.width == part_size.width() {
            let remainder_height = self.height - part_size.height();
            let part_node = NodeBlueprint::new(self.width, part_size.height(), Some(parttype), self.next_cut_orient);
            let remainder_node = NodeBlueprint::new(self.width, remainder_height, None, self.next_cut_orient);

            emit(vec![part_node, remainder_node]);
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

        if self.next_cut_orient == Orientation::Horizontal && self.width == part_size.width() && self.level < max_level {
            let mut copy = NodeBlueprint::new(self.width, self.height, None, self.next_cut_orient);

            let remainder_height = self.height - part_size.height();

            let part_node = NodeBlueprint::new(self.width, part_size.height(), Some(parttype), self.next_cut_orient.rotate());
            let remainder_node = NodeBlueprint::new(self.width, remainder_height, None, self.next_cut_orient.rotate());

            copy.add_child(part_node);
            copy.add_child(remainder_node);

            emit(vec![copy]);
            return;
        }

        if self.next_cut_orient == Orientation::Vertical && self.height == part_size.height() && self.level < max_level {
            let mut copy = NodeBlueprint::new(self.width, self.height, None, self.next_cut_orient);

            let remainder_width = self.width - part_size.width();

            let part_node = NodeBlueprint::new(part_size.width(), self.height, Some(parttype), self.next_cut_orient.rotate());
            let remainder_node = NodeBlueprint::new(remainder_width, self.height, None, self.next_cut_orient.rotate());

            copy.add_child(part_node);
            copy.add_child(remainder_node);

            emit(vec![copy]);
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

        if self.next_cut_orient == Orientation::Horizontal && self.level < max_level {
            let remainder_width_top = self.width - part_size.width();
            let mut part_node_parent = NodeBlueprint::new(part_size.width(), self.height, None, self.next_cut_orient);
            let remainder_node_top = NodeBlueprint::new(remainder_width_top, self.height, None, self.next_cut_orient);

            let remainder_height_bottom = self.height - part_size.height();
            let part_node = NodeBlueprint::new(part_size.width(), part_size.height(), Some(parttype), self.next_cut_orient.rotate());
            let remainder_node_bottom = NodeBlueprint::new(part_size.width(), remainder_height_bottom, None, self.next_cut_orient.rotate());

            part_node_parent.add_child(part_node);
            part_node_parent.add_child(remainder_node_bottom);

            emit(vec![part_node_parent, remainder_node_top]);
        }

        if self.next_cut_orient == Orientation::Vertical && self.level < max_level {
            let remainder_height_top = self.height - part_size.height();
            let mut part_node_parent = NodeBlueprint::new(self.width, part_size.height(), None, self.next_cut_orient);
            let remainder_node_top = NodeBlueprint::new(self.width, remainder_height_top, None, self.next_cut_orient);

            let remainder_width_bottom = self.width - part_size.width();
            let part_node = NodeBlueprint::new(part_size.width(), part_size.height(), Some(parttype), self.next_cut_orient.rotate());
            let remainder_node_bottom = NodeBlueprint::new(remainder_width_bottom, part_size.height(), None, self.next_cut_orient.rotate());

            part_node_parent.add_child(part_node);
            part_node_parent.add_child(remainder_node_bottom);

            emit(vec![part_node_parent, remainder_node_top]);
        }

        /*
             Scenario 4.2: First cut in opposite of current orientation
             ---*****          ---*****             *       ->      *   *
                *   *             *$* *                            / \
                *   *     ->      *****                           *   *
                *   *             *   *                          / \
             ---*****          ---*****                         $   *

         */

        if self.next_cut_orient == Orientation::Horizontal && self.level + 1 < max_level {
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

            emit(vec![copy]);
        }

        if self.next_cut_orient == Orientation::Vertical && self.level + 1 < max_level {
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

            emit(vec![copy]);
        }
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

    pub fn calculate_cost(&self, leftover_valuation_power: f32) -> Cost {
        match (self.parttype, self.children.is_empty()) {
            (Some(_), true) => Cost::empty(), // part-node
            (None, false) => Cost::empty(), // structure-node
            (None, true) => Cost::empty().add_leftover_value(leftover_valuator::valuate(
                self.area(),
                leftover_valuation_power,
            )), //leftover node
            (Some(_), false) => panic!("Parttype set on node with children"),
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
    pub fn children(&self) -> &Vec<NodeKey> {
        &self.children
    }
    pub fn parent(&self) -> &Option<NodeKey> {
        &self.parent
    }
    pub fn level(&self) -> u8 {
        self.level
    }
}
