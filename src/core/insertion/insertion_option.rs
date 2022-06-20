use std::cell::{Ref, RefCell};
use crate::core::entities::node::Node;
use crate::{Orientation, PartType, Rotation};
use crate::core::insertion::insertion_blueprint::InsertionBlueprint;

pub struct InsertionOption<'a,'b>{
    original_node: &'a Node,
    parttype: &'b PartType,
    rotation: Option<Rotation>,
    blueprints: RefCell<Option<Vec<InsertionBlueprint<'a,'b>>>>
}

impl<'a,'b> InsertionOption<'a,'b>{
    pub fn new(original_node: &'a Node, parttype: &'b PartType, rotation: Option<Rotation>) -> Self {
        Self {
            original_node,
            parttype,
            rotation,
            blueprints: RefCell::new(None)
        }
    }

    pub fn get_blueprints(&self) -> Ref<'_, Option<Vec<InsertionBlueprint<'a,'b>>>> {
        if self.blueprints.borrow().is_none() {
            self.blueprints.replace(Some(self.generate_blueprints()));
        }

        self.blueprints.borrow()
    }

    fn generate_blueprints(&self) -> Vec<InsertionBlueprint<'a,'b>> {
        let mut blueprints = Vec::new();
        match self.rotation{
            Some(rotation) => {
                self.original_node.generate_insertion_blueprints(&mut blueprints, self.parttype, rotation)
            }
            None => {
                self.original_node.generate_insertion_blueprints(&mut blueprints, self.parttype, Rotation::Default);
                self.original_node.generate_insertion_blueprints(&mut blueprints, self.parttype, Rotation::Rotated);

            }
        }
        blueprints
    }
}