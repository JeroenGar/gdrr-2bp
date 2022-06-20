use crate::Orientation;

pub struct NodeBlueprint{
    width : u64,
    height: u64,
    children : Vec<NodeBlueprint>,
    parttype: Option<usize>,
    next_cut_orient : Orientation
}

impl NodeBlueprint{
    pub fn new(width: u64, height: u64, parttype: Option<usize>, next_cut_orient: Orientation) -> Self {
        let children = Vec::new();
        Self { width, height, children, parttype, next_cut_orient }
    }


    pub fn add_child(&mut self, child: NodeBlueprint) {
        self.children.push(child);
    }
}
