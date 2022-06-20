use std::cell::{Ref, RefCell};
use std::hash::{Hash, Hasher};
use by_address::ByAddress;
use crate::core::entities::node::Node;
use crate::{Orientation, PartType, Rotation};
use crate::core::insertion::insertion_blueprint::InsertionBlueprint;

#[derive(Debug)]
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


    pub fn original_node(&self) -> &'a Node {
        self.original_node
    }
    pub fn parttype(&self) -> &'b PartType {
        self.parttype
    }
    pub fn rotation(&self) -> Option<Rotation> {
        self.rotation
    }
    pub fn blueprints(&self) -> &RefCell<Option<Vec<InsertionBlueprint<'a, 'b>>>> {
        &self.blueprints
    }
}

impl Hash for InsertionOption<'_,'_>{
    fn hash<H: Hasher>(&self, state: &mut H) {
        ByAddress(self.original_node()).hash(state);
        self.parttype().hash(state);
        self.rotation().hash(state);
    }
}

impl PartialEq for InsertionOption<'_, '_> {
    fn eq(&self, other: &Self) -> bool {
        self.original_node() as *const _ == other.original_node() as *const _ &&
        self.parttype() == other.parttype() &&
        self.rotation() == other.rotation()
    }
}

impl Eq for InsertionOption<'_, '_> {

}