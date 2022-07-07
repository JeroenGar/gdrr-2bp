use std::cell::RefCell;
use std::rc::Rc;

use crate::{Orientation, PartType};
use crate::core::cost::Cost;
use crate::core::entities::node::Node;
use crate::core::leftover_valuator;

#[derive(Debug, Clone)]
pub struct NodeBlueprint {
    width: u64,
    height: u64,
    children: Vec<NodeBlueprint>,
    parttype_id: Option<usize>,
    next_cut_orient: Orientation,
}

impl NodeBlueprint {
    pub fn new(width: u64, height: u64, parttype: Option<&PartType>, next_cut_orient: Orientation) -> Self {
        let children = Vec::new();
        let parttype_id = match parttype {
            Some(parttype) => Some(parttype.id()),
            None => None,
        };
        Self { width, height, children, parttype_id, next_cut_orient }
    }

    pub fn from_node(node: &Rc<RefCell<Node>>) -> Self {
        let node = node.as_ref().borrow();
        let parttype_id = match node.parttype() {
            Some(pt) => Some(pt.id()),
            None => None
        };
        let mut b_node = Self {
            width: node.width(),
            height: node.height(),
            parttype_id: parttype_id,
            children: Vec::new(),
            next_cut_orient: node.next_cut_orient(),
        };
        node.children().iter().for_each(|child| {
            b_node.children.push(NodeBlueprint::from_node(child));
        });
        b_node
    }

    pub fn add_child(&mut self, child: NodeBlueprint) {
        self.children.push(child);
    }

    pub fn calculate_cost(&self) -> Cost {
        if self.parttype_id.is_some() {
            return Cost::new(0, 0.0, 0, 0);
        } else if self.children.is_empty() {
            return Cost::new(0, leftover_valuator::valuate(self.area()), 0, 0);
        } else {
            let mut cost = Cost::new(0, 0.0, 0, 0);
            for child in &self.children {
                let child_cost = child.calculate_cost();
                cost.add(&child_cost);
            }
            return cost;
        }
    }

    pub fn calculate_usage(&self) -> f64 {
        if self.parttype_id.is_some() {
            1.0
        } else if self.children.is_empty() {
            0.0
        } else {
            let mut usage = 0.0;
            for child in &self.children {
                usage += child.area() as f64 * child.calculate_usage();
            }
            usage /= self.area() as f64;
            debug_assert!(usage <= 1.0);
            usage
        }
    }


    pub fn area(&self) -> u64 {
        self.width * self.height
    }

    pub fn width(&self) -> u64 {
        self.width
    }
    pub fn height(&self) -> u64 {
        self.height
    }
    pub fn children(&self) -> &Vec<NodeBlueprint> {
        &self.children
    }
    pub fn parttype_id(&self) -> Option<usize> {
        self.parttype_id
    }
    pub fn next_cut_orient(&self) -> Orientation {
        self.next_cut_orient
    }
}
