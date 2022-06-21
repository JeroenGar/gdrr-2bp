use std::cell::{Ref, RefCell};
use std::hash::{Hash, Hasher};
use std::rc::Weak;
use by_address::ByAddress;
use crate::core::entities::node::Node;
use crate::{Orientation, PartType, Rotation};
use crate::core::insertion::insertion_blueprint::InsertionBlueprint;

#[derive(Debug)]
pub struct InsertionOption<'a> {
    original_node: Weak<RefCell<Node<'a>>>,
    parttype: &'a PartType,
    rotation: Option<Rotation>,
    blueprints: RefCell<Option<Vec<InsertionBlueprint<'a>>>>,
}

impl<'a> InsertionOption<'a> {
    pub fn new(original_node: Weak<RefCell<Node<'a>>>, parttype: &'a PartType, rotation: Option<Rotation>) -> Self {
        Self {
            original_node,
            parttype,
            rotation,
            blueprints: RefCell::new(None),
        }
    }

    pub fn get_blueprints(&self) -> Ref<'_, Option<Vec<InsertionBlueprint<'a>>>> {
        if self.blueprints.borrow().is_none() {
            self.blueprints.replace(Some(self.generate_blueprints()));
        }

        self.blueprints.borrow()
    }

    fn generate_blueprints(&self) -> Vec<InsertionBlueprint<'a>> {
        let mut blueprints = Vec::new();
        let original_node = self.original_node.upgrade().unwrap();
        match self.rotation {
            Some(rotation) => {
                Node::generate_insertion_blueprints(&original_node, &mut blueprints, self.parttype, rotation)
            }
            None => {
                Node::generate_insertion_blueprints(&original_node, &mut blueprints, self.parttype, Rotation::Default);
                Node::generate_insertion_blueprints(&original_node, &mut blueprints, self.parttype, Rotation::Rotated);
            }
        }
        blueprints
    }


    pub fn parttype(&self) -> &'a PartType {
        self.parttype
    }
    pub fn rotation(&self) -> Option<Rotation> {
        self.rotation
    }
    pub fn blueprints(&self) -> &RefCell<Option<Vec<InsertionBlueprint<'a>>>> {
        &self.blueprints
    }
    pub fn original_node(&self) -> &Weak<RefCell<Node<'a>>> {
        &self.original_node
    }
}

impl Hash for InsertionOption<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        ByAddress(self.original_node()).hash(state);
        self.parttype().hash(state);
        self.rotation().hash(state);
    }
}

impl PartialEq for InsertionOption<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.original_node() as *const _ == other.original_node() as *const _ &&
            self.parttype() == other.parttype() &&
            self.rotation() == other.rotation()
    }
}

impl Eq for InsertionOption<'_> {}