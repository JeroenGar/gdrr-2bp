use crate::core::cost::Cost;
use crate::core::entities::layout::Layout;
use crate::core::insertion::node_blueprint::NodeBlueprint;

///Representation of a layout that can be sent across threads

#[derive(Debug, Clone)]
pub struct SendableLayout {
    pub sheettype_id: usize,
    pub top_node: NodeBlueprint,
    pub cost: Cost,
    pub usage: f64,
}

impl SendableLayout {
    pub fn new(layout: &Layout) -> Self {
        Self {
            sheettype_id: layout.sheettype().id,
            top_node: NodeBlueprint::from_node(*layout.top_node_index(), layout.nodes()),
            cost: layout.cost_immut(false),
            usage: layout.usage(),
        }
    }

}
