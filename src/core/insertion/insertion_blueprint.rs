use crate::core::cost::Cost;
use crate::core::entities::node::Node;
use crate::core::insertion::node_blueprint::NodeBlueprint;
use crate::PartType;

#[derive(Debug)]
pub struct InsertionBlueprint<'a,'b> {
    original_node: &'a Node,
    replacements: Vec<NodeBlueprint<'b>>,
    parttype: &'b PartType,
    cost: Cost,
}


impl<'a,'b> InsertionBlueprint<'a,'b> {
    pub fn new(original_node: &'a Node, replacements: Vec<NodeBlueprint<'b>>, parttype: &'b PartType) -> Self {
        let cost = InsertionBlueprint::calculate_cost(original_node, &replacements);
        Self { original_node, replacements, parttype, cost }
    }

    fn calculate_cost(original_node: &'a Node, replacements: &Vec<NodeBlueprint>) -> Cost {
        todo!()
    }
}