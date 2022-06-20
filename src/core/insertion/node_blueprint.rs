use crate::{Orientation, PartType};

pub struct NodeBlueprint<'a>{
    width : u64,
    height: u64,
    children : Vec<NodeBlueprint<'a>>,
    parttype: Option<&'a PartType>,
    next_cut_orient : Orientation
}

impl<'a> NodeBlueprint<'a>{
    pub fn new(width: u64, height: u64, parttype: Option<&'a PartType>, next_cut_orient: Orientation) -> Self {
        let children = Vec::new();
        Self { width, height, children, parttype, next_cut_orient }
    }


    pub fn add_child(&mut self, child: NodeBlueprint<'a>) {
        self.children.push(child);
    }
}
