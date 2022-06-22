use std::{rc::Rc};
use std::cell::RefCell;
use std::rc::Weak;
use by_address::ByAddress;
use indexmap::IndexMap;
use crate::core::cost::Cost;

use crate::{Orientation, PartType};
use crate::core::insertion::insertion_blueprint::InsertionBlueprint;
use crate::core::insertion::node_blueprint::NodeBlueprint;
use crate::core::rotation::Rotation;

#[derive(Debug)]
pub struct Node<'a> {
    width: u64,
    height: u64,
    children: Vec<Rc<RefCell<Node<'a>>>>,
    parent: Option<Weak<RefCell<Node<'a>>>>,
    parttype: Option<&'a PartType>,
    next_cut_orient: Orientation,
}


impl<'a> Node<'a> {
    pub fn new(width: u64, height: u64, next_cut_orient: Orientation) -> Node<'a> {
        Node {
            width,
            height,
            children: Vec::new(),
            parent: None,
            parttype: None,
            next_cut_orient,
        }
    }

    pub fn new_from_blueprint(blueprint: &NodeBlueprint) -> Node<'a> {
        todo!();
    }

    pub fn create_deep_copy(
        &self,
        parent: Option<Weak<RefCell<Node<'a>>>>,
        original_copy_node_map: &mut IndexMap<ByAddress<Rc<RefCell<Node<'a>>>>,
            Rc<RefCell<Node<'a>>>>) -> Rc<RefCell<Node<'a>>> {
        let mut copy = Node::new(self.width, self.height, self.next_cut_orient);
        copy.set_parent(parent);
        let copy = Rc::new(RefCell::new(copy));

        for child in &self.children {
            let child_copy = child.as_ref().borrow().create_deep_copy(
                Some(Rc::downgrade(&copy)),
                original_copy_node_map);
            original_copy_node_map.insert(ByAddress(child.clone()), child_copy.clone());
            copy.as_ref().borrow_mut().add_child(child_copy);
        }

        copy
    }

    pub fn add_child(&mut self, child: Rc<RefCell<Node<'a>>>) {
        todo!()
    }

    pub fn remove_child(&mut self, child: &Rc<RefCell<Node<'a>>>) -> Rc<RefCell<Node>> {
        /*
           Scenario 1: Waste piece present + other child(ren)
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
        let child_ref = child.as_ref().borrow();

        //Check if there is an empty_node present
        let empty_node = self.children.iter().find(|node: &&Rc<RefCell<Node>>| {
            let node_ref = node.as_ref().borrow();
            node_ref.parttype.is_none() && node_ref.children.is_empty()
        }).cloned();

        if empty_node.is_some() {
            //Scenario 1 and 3
            let mut empty_node_ref = empty_node.as_ref().unwrap().as_ref().borrow_mut();

            if self.children.len() > 1 || self.parent.is_none() {
                //Scenario 1 (also do this when node is the root)
                //Two children are merged into one
                match self.next_cut_orient {
                    Orientation::Horizontal => {
                        let new_height = empty_node_ref.height + child_ref.height;
                        empty_node_ref.set_height(new_height);
                    }
                    Orientation::Vertical => {
                        let new_width = empty_node_ref.width + child_ref.width;
                        empty_node_ref.set_width(new_width);
                    }
                }
            } else {
                //Scenario 3: convert the parent into an empty node
                self.children.clear();
            }

            //Remove the child
            let child_to_remove_index = self.children.iter().position(|c| Rc::ptr_eq(c, child)).unwrap();
            let removed_child = self.children.remove(child_to_remove_index);
        } else {
            //Scenario 2: convert the node itself into an empty node
            let child_to_remove_index = self.children.iter().position(|c| Rc::ptr_eq(c, child)).unwrap();
            let mut child_to_remove = self.children.get(child_to_remove_index).unwrap().as_ref().borrow_mut();
            child_to_remove.children.clear();
        }

        todo!()
    }

    pub fn replace_child(&mut self, old_child: &Node<'a>, replacements: Vec<Node<'a>>) -> Rc<RefCell<Node<'a>>> {
        let old_child_index = self.children.iter().position(|c| c.as_ptr() as *const Node == old_child as *const Node).unwrap();
        let old_child = self.children.remove(old_child_index);

        for replacement in replacements {
            let wrapped = Rc::new(RefCell::new(replacement));
            self.children.push(wrapped);
        }

        old_child
    }

    pub fn generate_insertion_blueprints(wrapped_node: &Rc<RefCell<Node<'a>>>, insertion_blueprints: &mut Vec<InsertionBlueprint<'a>>, parttype: &'a PartType, rotation: Rotation) {
        let node = wrapped_node.as_ref().borrow();
        debug_assert!(node.insertion_possible(parttype, rotation));

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


        if node.next_cut_orient == Orientation::Horizontal && node.height == part_size.height() {
            let remainder_width = node.width - part_size.width();
            let part_node = NodeBlueprint::new(part_size.width(), node.height, Some(parttype), node.next_cut_orient);
            let remainder_node = NodeBlueprint::new(remainder_width, node.height, None, node.next_cut_orient);

            let insertion_blueprint = InsertionBlueprint::new(Rc::downgrade(wrapped_node), vec![part_node, remainder_node], parttype);
            insertion_blueprints.push(insertion_blueprint);
            return;
        }
        if node.next_cut_orient == Orientation::Vertical && node.width == part_size.width() {
            let remainder_height = node.height - part_size.height();
            let part_node = NodeBlueprint::new(node.width, part_size.height(), Some(parttype), node.next_cut_orient);
            let remainder_node = NodeBlueprint::new(node.width, remainder_height, None, node.next_cut_orient);

            let insertion_blueprint = InsertionBlueprint::new(Rc::downgrade(wrapped_node), vec![part_node, remainder_node], parttype);
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

        if node.next_cut_orient == Orientation::Horizontal && node.width == part_size.width() {
            let mut copy = NodeBlueprint::new(node.width, node.height, None, node.next_cut_orient);

            let remainder_height = node.height - part_size.height();

            let part_node = NodeBlueprint::new(node.width, part_size.height(), Some(parttype), node.next_cut_orient.rotate());
            let remainder_node = NodeBlueprint::new(node.width, remainder_height, None, node.next_cut_orient.rotate());

            copy.add_child(part_node);
            copy.add_child(remainder_node);

            let insertion_blueprint = InsertionBlueprint::new(Rc::downgrade(wrapped_node), vec![copy], parttype);
            insertion_blueprints.push(insertion_blueprint);

            return;
        }

        if node.next_cut_orient == Orientation::Vertical && node.height == part_size.height() {
            let mut copy = NodeBlueprint::new(node.width, node.height, None, node.next_cut_orient);

            let remainder_width = node.width - part_size.width();

            let part_node = NodeBlueprint::new(part_size.width(), node.height, Some(parttype), node.next_cut_orient.rotate());
            let remainder_node = NodeBlueprint::new(remainder_width, node.height, None, node.next_cut_orient.rotate());

            copy.add_child(part_node);
            copy.add_child(remainder_node);

            let insertion_blueprint = InsertionBlueprint::new(Rc::downgrade(wrapped_node), vec![copy], parttype);
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

        if node.next_cut_orient == Orientation::Horizontal {
            let remainder_width_top = node.width - part_size.width();
            let mut part_node_parent = NodeBlueprint::new(part_size.width(), node.height, None, node.next_cut_orient);
            let remainder_node_top = NodeBlueprint::new(remainder_width_top, node.height, None, node.next_cut_orient);

            let remainder_height_bottom = node.height - part_size.height();
            let part_node = NodeBlueprint::new(part_size.width(), part_size.height(), Some(parttype), node.next_cut_orient.rotate());
            let remainder_node_bottom = NodeBlueprint::new(part_size.width(), remainder_height_bottom, None, node.next_cut_orient.rotate());

            part_node_parent.add_child(part_node);
            part_node_parent.add_child(remainder_node_bottom);

            let insertion_blueprint = InsertionBlueprint::new(Rc::downgrade(wrapped_node), vec![part_node_parent, remainder_node_top], parttype);
            insertion_blueprints.push(insertion_blueprint);
        }

        if node.next_cut_orient == Orientation::Vertical {
            let remainder_height_top = node.height - part_size.height();
            let mut part_node_parent = NodeBlueprint::new(node.width, part_size.height(), None, node.next_cut_orient);
            let remainder_node_top = NodeBlueprint::new(node.width, remainder_height_top, None, node.next_cut_orient);

            let remainder_width_bottom = node.width - part_size.width();
            let part_node = NodeBlueprint::new(part_size.width(), part_size.height(), Some(parttype), node.next_cut_orient.rotate());
            let remainder_node_bottom = NodeBlueprint::new(remainder_width_bottom, part_size.height(), None, node.next_cut_orient.rotate());

            part_node_parent.add_child(part_node);
            part_node_parent.add_child(remainder_node_bottom);

            let insertion_blueprint = InsertionBlueprint::new(Rc::downgrade(wrapped_node), vec![part_node_parent, remainder_node_top], parttype);
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

        if node.next_cut_orient == Orientation::Horizontal {
            let mut copy = NodeBlueprint::new(node.width, node.height, None, node.next_cut_orient);

            let remainder_height_top = node.height - part_size.height();
            let mut part_node_parent = NodeBlueprint::new(node.width, part_size.height(), None, node.next_cut_orient.rotate());
            let remainder_node_top = NodeBlueprint::new(node.width, remainder_height_top, None, node.next_cut_orient.rotate());

            let remainder_width_bottom = node.width - part_size.width();
            let part_node = NodeBlueprint::new(part_size.width(), part_size.height(), Some(parttype), node.next_cut_orient.rotate().rotate());
            let remainder_node_bottom = NodeBlueprint::new(remainder_width_bottom, part_size.height(), None, node.next_cut_orient.rotate().rotate());

            part_node_parent.add_child(part_node);
            part_node_parent.add_child(remainder_node_bottom);

            copy.add_child(part_node_parent);
            copy.add_child(remainder_node_top);

            let insertion_blueprint = InsertionBlueprint::new(Rc::downgrade(wrapped_node), vec![copy], parttype);
            insertion_blueprints.push(insertion_blueprint);
        }

        if node.next_cut_orient == Orientation::Vertical {
            let mut copy = NodeBlueprint::new(node.width, node.height, None, node.next_cut_orient);

            let remainder_width_top = node.width - part_size.width();
            let mut part_node_parent = NodeBlueprint::new(part_size.width(), node.height, None, node.next_cut_orient.rotate());
            let remainder_node_top = NodeBlueprint::new(remainder_width_top, node.height, None, node.next_cut_orient.rotate().rotate());

            let remainder_height_bottom = node.height - part_size.height();

            let part_node = NodeBlueprint::new(part_size.width(), part_size.height(), Some(parttype), node.next_cut_orient.rotate().rotate());
            let remainder_node_bottom = NodeBlueprint::new(part_size.width(), remainder_height_bottom, None, node.next_cut_orient.rotate().rotate());

            part_node_parent.add_child(part_node);
            part_node_parent.add_child(remainder_node_bottom);

            copy.add_child(part_node_parent);
            copy.add_child(remainder_node_top);

            let insertion_blueprint = InsertionBlueprint::new(Rc::downgrade(wrapped_node), vec![copy], parttype);
            insertion_blueprints.push(insertion_blueprint);
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

    pub fn get_cost() -> Cost {
        todo!()
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
    pub fn children(&self) -> &Vec<Rc<RefCell<Node<'a>>>> {
        &self.children
    }
    pub fn parent(&self) -> &Option<Weak<RefCell<Node<'a>>>> {
        &self.parent
    }


    pub fn set_width(&mut self, width: u64) {
        self.width = width;
    }
    pub fn set_height(&mut self, height: u64) {
        self.height = height;
    }
    pub fn set_children(&mut self, children: Vec<Rc<RefCell<Node<'a>>>>) {
        self.children = children;
    }
    pub fn set_parent(&mut self, parent: Option<Weak<RefCell<Node<'a>>>>) {
        self.parent = parent;
    }

    pub fn set_next_cut_orient(&mut self, next_cut_orient: Orientation) {
        self.next_cut_orient = next_cut_orient;
    }
    pub fn set_parttype(&mut self, parttype: Option<&'a PartType>) {
        self.parttype = parttype;
    }
}



