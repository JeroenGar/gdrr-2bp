use std::borrow::Borrow;
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use crate::core::cost::Cost;
use crate::core::entities::layout::Layout;
use crate::core::entities::node::Node;
use crate::core::insertion::node_blueprint::NodeBlueprint;
use crate::PartType;
use crate::util::assertions;
use crate::util::macros::{rb,rbm};

#[derive(Debug, Clone)]
pub struct InsertionBlueprint<'a> {
    original_node: Weak<RefCell<Node<'a>>>,
    replacements: Vec<NodeBlueprint<'a>>,
    parttype: &'a PartType,
    cost: Cost,
    layout: Option<Weak<RefCell<Layout<'a>>>>
}


impl<'a> InsertionBlueprint<'a> {
    pub fn new(original_node: Weak<RefCell<Node<'a>>>, replacements: Vec<NodeBlueprint<'a>>, parttype: &'a PartType) -> Self {
        debug_assert!(rb!(original_node.upgrade().unwrap()).parent().is_some(),"{:#?}", original_node.upgrade().unwrap());
        debug_assert!(assertions::replacements_fit(&original_node, &replacements), "{:#?}", (&original_node, &replacements));

        let cost = InsertionBlueprint::calculate_cost(&original_node, &replacements);
        Self { original_node, replacements, parttype, cost, layout : None}
    }

    fn calculate_cost(original_node: &Weak<RefCell<Node>>, replacements: &Vec<NodeBlueprint>) -> Cost {
        let original_cost = rb!(original_node.upgrade().unwrap()).calculate_cost();
        let new_cost : Cost = replacements.iter().map(|replacement| replacement.calculate_cost()).sum();

        new_cost - original_cost
    }

    pub fn set_layout(&mut self, layout: Weak<RefCell<Layout<'a>>>) {
        self.layout = Some(layout);
    }

    pub fn replacements(&self) -> &Vec<NodeBlueprint<'a>> {
        &self.replacements
    }
    pub fn parttype(&self) -> &'a PartType {
        self.parttype
    }
    pub fn cost(&self) -> &Cost {
        &self.cost
    }
    pub fn layout(&self) -> &Option<Weak<RefCell<Layout<'a>>>> {
        &self.layout
    }
    pub fn original_node(&self) -> &Weak<RefCell<Node<'a>>> {
        &self.original_node
    }


    pub fn set_original_node(&mut self, original_node: Weak<RefCell<Node<'a>>>) {
        self.original_node = original_node;
    }
    pub fn set_replacements(&mut self, replacements: Vec<NodeBlueprint<'a>>) {
        self.replacements = replacements;
    }
    pub fn set_parttype(&mut self, parttype: &'a PartType) {
        self.parttype = parttype;
    }
}