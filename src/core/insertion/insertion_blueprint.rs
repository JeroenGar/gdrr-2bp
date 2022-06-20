use crate::core::cost::Cost;
use crate::core::entities::node::Node;
use crate::core::insertion::node_blueprint::NodeBlueprint;

pub struct InsertionBlueprint<'a> {
    original_node: &'a Node,
    replacements: Vec<NodeBlueprint>,
    parttype: usize,
    cost: Cost,
}


impl<'a> InsertionBlueprint<'a> {
    pub fn new(original_node: &'a Node, replacements: Vec<NodeBlueprint>, parttype: usize) -> Self {
        let cost = InsertionBlueprint::calculate_cost(original_node, &replacements);
        Self { original_node, replacements, parttype, cost }
    }

    fn calculate_cost(original_node: &'a Node, replacements: &Vec<NodeBlueprint>) -> Cost {
        todo!()
    }
}