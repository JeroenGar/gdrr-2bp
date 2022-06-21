use std::cell::RefCell;
use std::rc::{Rc, Weak};
use crate::core::cost::Cost;
use crate::core::entities::layout::Layout;
use crate::core::entities::node::Node;
use crate::core::insertion::node_blueprint::NodeBlueprint;
use crate::PartType;

#[derive(Debug)]
pub struct InsertionBlueprint<'b> {
    original_node: Weak<RefCell<Node>>,
    replacements: Vec<NodeBlueprint<'b>>,
    parttype: &'b PartType,
    cost: Cost,
    layout: Option<Weak<RefCell<Layout>>>
}


impl<'b> InsertionBlueprint<'b> {
    pub fn new(original_node: Weak<RefCell<Node>>, replacements: Vec<NodeBlueprint<'b>>, parttype: &'b PartType) -> Self {
        let cost = InsertionBlueprint::calculate_cost(&original_node, &replacements);
        let layout = None;
        Self { original_node, replacements, parttype, cost, layout }
    }

    fn calculate_cost(original_node: &Weak<RefCell<Node>>, replacements: &Vec<NodeBlueprint>) -> Cost {
        todo!()
    }

    pub fn set_layout(&mut self, layout: Weak<RefCell<Layout>>) {
        self.layout = Some(layout);
    }



    pub fn replacements(&self) -> &Vec<NodeBlueprint<'b>> {
        &self.replacements
    }
    pub fn parttype(&self) -> &'b PartType {
        self.parttype
    }
    pub fn cost(&self) -> &Cost {
        &self.cost
    }
    pub fn layout(&self) -> &Option<Weak<RefCell<Layout>>> {
        &self.layout
    }
    pub fn original_node(&self) -> &Weak<RefCell<Node>> {
        &self.original_node
    }
}