use std::borrow::Borrow;
use crate::{Orientation, PartType};
use crate::core::cost::Cost;
use crate::core::leftover_valuator;

#[derive(Debug, Clone)]
pub struct NodeBlueprint<'a> {
    width: u64,
    height: u64,
    children: Vec<NodeBlueprint<'a>>,
    parttype: Option<&'a PartType>,
    next_cut_orient: Orientation,
}

impl<'a> NodeBlueprint<'a> {
    pub fn new(width: u64, height: u64, parttype: Option<&'a PartType>, next_cut_orient: Orientation) -> Self {
        let children = Vec::new();
        Self { width, height, children, parttype, next_cut_orient }
    }

    pub fn add_child(&mut self, child: NodeBlueprint<'a>) {
        self.children.push(child);
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
                let child_cost = child.calculate_cost();
                cost.add(&child_cost);
            }
            return cost;
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
    pub fn children(&self) -> &Vec<NodeBlueprint<'a>> {
        &self.children
    }
    pub fn parttype(&self) -> Option<&'a PartType> {
        self.parttype
    }
    pub fn next_cut_orient(&self) -> Orientation {
        self.next_cut_orient
    }
}
