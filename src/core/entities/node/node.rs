use std::{rc::Rc};
use crate::core::cost::Cost;

use crate::core::entities::{layout::Layout};
use crate::core::entities::node::node_blueprint::NodeBlueprint;
use crate::Orientation;

pub struct Node {
    width : u64,
    height: u64,
    parent : Option<Rc<Node>>,
    children : Vec<Rc<Node>>,
    parttype: Option<usize>,
    next_cut_orient : Orientation
}


impl Node {
    pub fn new(width: u64, height: u64, next_cut_orient: Orientation) -> Node {
        Node {
            width: width,
            height: height,
            parent: None,
            children: Vec::new(),
            parttype: None,
            next_cut_orient: next_cut_orient
        }
    }

    pub fn new_from_blueprint(blueprint : &NodeBlueprint) -> Node {
        todo!();
    }

    pub fn create_deep_copy(&self) -> Node {
        todo!();
    }

    pub fn add_child(&mut self, child: Node) {
        todo!()
    }

    pub fn remove_child(&mut self, child: &Node) {
        todo!()
    }

    pub fn replace_child(&mut self, old_child: &Node, replacements: Vec<Node>) {
        todo!()
    }

    pub fn generate_insertion_blueprints(){
        todo!()
    }

    pub fn insertion_possible() -> bool {
        todo!()
    }

    pub fn get_cost() -> Cost {
        todo!()
    }



}



