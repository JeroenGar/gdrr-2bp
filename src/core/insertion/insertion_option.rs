use std::cell::{Ref, RefCell};
use std::hash::{Hash, Hasher};
use std::rc::Weak;
use by_address::ByAddress;
use crate::core::entities::node::Node;
use crate::{Orientation, PartType, Rotation};
use crate::core::entities::layout::Layout;
use crate::core::insertion::insertion_blueprint::InsertionBlueprint;

#[derive(Debug)]
pub struct InsertionOption<'a> {
    original_node: Weak<RefCell<Node<'a>>>,
    parttype: &'a PartType,
    rotation: Option<Rotation>,
    layout : Weak<RefCell<Layout<'a>>>
}

//TODO: cache the blueprints

impl<'a> InsertionOption<'a> {
    pub fn new(original_node: Weak<RefCell<Node<'a>>>, parttype: &'a PartType, rotation: Option<Rotation>, layout : Weak<RefCell<Layout<'a>>>) -> Self {
        Self {
            original_node,
            parttype,
            rotation,
            layout
        }
    }

    pub fn get_blueprints(&self) -> Vec<InsertionBlueprint<'a>> {
        /*if self.blueprints.borrow().is_none() {
            self.blueprints.replace(Some(self.generate_blueprints()));
        }

        self.blueprints.borrow()*/
        InsertionOption::generate_blueprints(self)
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
        blueprints.iter_mut().for_each(|blueprint| {
            blueprint.set_layout(self.layout.clone());
        });

        blueprints
    }


    pub fn parttype(&self) -> &'a PartType {
        self.parttype
    }
    pub fn rotation(&self) -> Option<Rotation> {
        self.rotation
    }

    pub fn original_node(&self) -> &Weak<RefCell<Node<'a>>> {
        &self.original_node
    }

    pub fn layout(&self) -> &Weak<RefCell<Layout<'a>>> {
        &self.layout
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