use crate::core::cost::Cost;
use crate::core::entities::layout::Layout;
use crate::core::insertion::node_blueprint::NodeBlueprint;
use crate::optimization::instance::Instance;

///Representation of a layout that can be sent across threads

#[derive(Debug, Clone)]
pub struct SendableLayout {
    sheettype_id: usize,
    top_node: NodeBlueprint,
    cost: Cost,
    usage: f64,
}

impl SendableLayout {
    pub fn new(layout: &Layout) -> Self {
        Self {
            sheettype_id: layout.sheettype().id(),
            top_node: NodeBlueprint::from_node(*layout.top_node_index(), layout.nodes()),
            cost: layout.cost_immut(false),
            usage: layout.usage_immut(false),
        }
    }

    pub fn convert_to_layout<'a>(&self, _instance: &'a Instance) -> Layout<'a> {
        todo!();
    }

    pub fn sheettype_id(&self) -> usize {
        self.sheettype_id
    }
    pub fn top_node(&self) -> &NodeBlueprint {
        &self.top_node
    }
    pub fn cost(&self) -> &Cost {
        &self.cost
    }
    pub fn usage(&self) -> f64 {
        self.usage
    }
}