use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::io::empty;
use std::ops::Deref;
use std::rc::Rc;
use std::rc::Weak;
use by_address::ByAddress;
use indexmap::IndexMap;

use crate::{Orientation, PartType};
use crate::core::cost::Cost;
use crate::core::insertion::insertion_blueprint::InsertionBlueprint;
use crate::core::insertion::node_blueprint::NodeBlueprint;
use crate::core::leftover_valuator;
use crate::core::rotation::Rotation;
use crate::optimization::config::Config;
use crate::optimization::rr::cache_updates::CacheUpdates;
use crate::util::assertions;
use crate::util::macros::{rb,rbm};


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

    pub fn new_top_node(width : u64, height : u64, next_cut_orient: Orientation) -> Rc<RefCell<Node<'a>>> {
        let mut top_node = Rc::new(RefCell::new(
            Node::new(width, height, next_cut_orient)
        ));
        let mut placeholder_node = Node::new(width, height, next_cut_orient.rotate());
        placeholder_node.parent = Some(Rc::downgrade(&top_node));
        rbm!(top_node).children.push(Rc::new(RefCell::new(placeholder_node)));

        top_node
    }

    pub fn new_from_blueprint(blueprint: &NodeBlueprint<'a>, parent: Weak<RefCell<Node<'a>>>, all_created_nodes: &mut Vec<Weak<RefCell<Node<'a>>>>) -> Rc<RefCell<Node<'a>>> {
        let mut node = Node {
            width: blueprint.width(),
            height: blueprint.height(),
            children: Vec::new(),
            parent: Some(parent),
            parttype: blueprint.parttype(),
            next_cut_orient: blueprint.next_cut_orient(),
        };

        let mut node = Rc::new(RefCell::new(node));
        all_created_nodes.push(Rc::downgrade(&node));

        let children = blueprint.children().iter().map(|child_bp| {
            Node::new_from_blueprint(child_bp, Rc::downgrade(&node), all_created_nodes)
        }).collect();

        rbm!(node).children = children;

        node
    }

    pub fn create_deep_copy(&self, parent: Option<Weak<RefCell<Node<'a>>>>) -> Rc<RefCell<Node<'a>>> {
        let mut copy = Node {
            width : self.width,
            height : self.height,
            children : Vec::new(),
            parent,
            parttype : self.parttype,
            next_cut_orient : self.next_cut_orient
        };
        let copy = Rc::new(RefCell::new(copy));

        for child in &self.children {
            let child_copy = rb!(child).create_deep_copy(
                Some(Rc::downgrade(&copy)));
            rbm!(copy).children.push(child_copy);
        }

        copy
    }

    pub fn replace_child(node : &Rc<RefCell<Node<'a>>>, old_child: &Rc<RefCell<Node<'a>>>, replacements: Vec<Rc<RefCell<Node<'a>>>>) {
        let mut node_ref = rbm!(node);

        rbm!(old_child).parent = None;
        let old_child_index = node_ref.children.iter().position(|c| Rc::ptr_eq(c, old_child)).unwrap();
        node_ref.children.remove(old_child_index);

        replacements.iter().for_each(|r| {
            rbm!(r).parent = Some(Rc::downgrade(node));
        });
        node_ref.children.extend(replacements);

        debug_assert!(assertions::children_nodes_fit(node_ref.deref()))
    }

    pub fn replace_children(node : &Rc<RefCell<Node<'a>>>, old_children: Vec<&Rc<RefCell<Node<'a>>>>, replacements: Vec<Rc<RefCell<Node<'a>>>>){
        let mut node_ref = rbm!(node);

        for old_child in old_children.iter() {
            rbm!(old_child).parent = None;
            let old_child_index = node_ref.children.iter().position(|c| Rc::ptr_eq(c, old_child)).unwrap();
            node_ref.children.remove(old_child_index);
        }

        replacements.iter().for_each(|r| {
            rbm!(r).parent = Some(Rc::downgrade(node));
        });
        node_ref.children.extend(replacements);

        debug_assert!(assertions::children_nodes_fit(node_ref.deref()))
    }

    pub fn clear_children(node : &Rc<RefCell<Node<'a>>>) {
        let mut node_ref = rbm!(node);
        node_ref.children.iter().for_each(|child| {
            let mut child_ref = rbm!(child);
            child_ref.parent = None;
        });
        node_ref.children.clear();
    }

    pub fn generate_insertion_blueprints(wrapped_node: &Rc<RefCell<Node<'a>>>, insertion_blueprints: &mut Vec<InsertionBlueprint<'a>>, parttype: &'a PartType, rotation: Rotation) {
        let node = rb!(wrapped_node);
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
            let remainder_node_top = NodeBlueprint::new(remainder_width_top, node.height, None, node.next_cut_orient.rotate());

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

    pub fn get_included_parts(&self, included_parts: &mut Vec<&'a PartType>) {
        debug_assert!(!(self.parttype.is_some() && !self.children.is_empty()));

        match self.parttype {
            Some(parttype) => {
                included_parts.push(parttype);
            }
            None => {
                for child in &self.children {
                    rb!(child).get_included_parts(included_parts);
                }
            }
        }
    }

    pub fn get_all_children(&self, children: &mut Vec<Weak<RefCell<Node<'a>>>>) {
        debug_assert!(!(self.parttype.is_some() && !self.children.is_empty()));

        match self.children.is_empty() {
            true => {
                // do nothing
            }
            false => {
                for child in &self.children {
                    children.push(Rc::downgrade(&child));
                    rb!(child).get_all_children(children);
                }
            }
        }
    }

    pub fn get_all_removable_children(&self, children : &mut Vec<Weak<RefCell<Node<'a>>>>) {
        debug_assert!(!(self.parttype.is_some() && !self.children.is_empty()));

        match self.children.is_empty() {
            true => {
                // do nothing
            }
            false => {
                for child in &self.children {
                    if rb!(child).parttype.is_some() || !rb!(child).children.is_empty() {
                        children.push(Rc::downgrade(&child));
                    }
                    rb!(child).get_all_removable_children(children);
                }
            }
        }
    }

    pub fn calculate_cost(&self) -> Cost {
        if self.parttype.is_some() {
            return Cost::new(0, 0.0, 0);
        }
        else if self.children.is_empty() {
            return Cost::new(0, leftover_valuator::valuate(self.area()), 0);
        }
        else {
            let mut cost = Cost::new(0, 0.0, 0);
            for child in &self.children {
                let child_cost = rb!(child).calculate_cost();
                cost.add(&child_cost);
            }
            return cost;
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

    pub fn is_empty(&self) -> bool{
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
    pub fn children(&self) -> &Vec<Rc<RefCell<Node<'a>>>> {
        &self.children
    }
    pub fn parent(&self) -> &Option<Weak<RefCell<Node<'a>>>> {
        &self.parent
    }

}



