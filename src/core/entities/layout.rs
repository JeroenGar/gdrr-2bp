use std::cell::RefCell;
use std::collections::LinkedList;
use std::rc::{Rc, Weak};

use crate::core::{cost::Cost, insertion::insertion_blueprint::InsertionBlueprint};
use crate::core::entities::node::Node;
use crate::Orientation;

use super::{sheettype::SheetType, parttype::PartType};

#[derive(Debug)]
pub struct Layout{
    id : usize,
    sheettype : usize,
    top_node : Rc<RefCell<Node>>,
    cached_cost: RefCell<Option<Cost>>,
    usage : f64,
    sorted_empty_nodes: Vec<Weak<RefCell<Node>>>
}


impl Layout{

    pub fn new(sheettype: &SheetType, first_cut_orientation : Orientation) -> Self{
        //let top_node = Node::new(sheettype.width(), sheettype.height(), first_cut_orientation);
        todo!()
    }

    pub fn implement_insertion_blueprint(&mut self, blueprint: &InsertionBlueprint) {
        todo!()
    }

    pub fn implement_insertion(&mut self, blueprint: &InsertionBlueprint){
        todo!()
    }

    pub fn remove_node(&mut self, node: &Node){
        todo!()
    }

    pub fn create_deep_copy(&self) -> Layout{
        todo!()
    }

    pub fn get_cost(&self) -> Cost{
        todo!()
    }

    fn recalculate_cost(&self){
        todo!()
    }

    fn register_node(&mut self, node: &Node, recursive : bool){
        todo!()
    }

    fn deregister_node(&mut self, node: &Node, recursive : bool){
        todo!()
    }

    fn register_part(&mut self, parttype: &PartType){
        todo!()
    }

    fn deregister_part(&mut self, parttype: &PartType){
        todo!()
    }

    pub fn get_sorted_empty_nodes(&self) -> &Vec<Weak<RefCell<Node>>>{
        todo!()
    }

}