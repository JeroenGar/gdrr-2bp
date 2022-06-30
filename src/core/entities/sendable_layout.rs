use crate::core::cost::Cost;
use crate::core::entities::layout::Layout;
use crate::core::insertion::node_blueprint::NodeBlueprint;
use crate::core::entities::sheettype::SheetType;
use crate::Instance;

#[derive(Debug, Clone)]
pub struct SendableLayout{
    sheettype_id : usize,
    top_node : NodeBlueprint,
    cost : Cost,
    usage : f64
}

impl SendableLayout{
    pub fn new(layout : &Layout) -> Self{
        Self {
            sheettype_id : layout.sheettype().id(),
            top_node : NodeBlueprint::from_node(&layout.top_node()),
            cost : layout.cost().clone(),
            usage : layout.usage()
        }
    }

    pub fn convert_to_layout(&self, instance : &Instance) -> Layout{
        todo!();
    }
}