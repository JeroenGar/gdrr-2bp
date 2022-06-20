use std::cell::RefCell;

use crate::core::{cost::Cost, insertion::insertion_blueprint::InsertionBlueprint};
use crate::core::entities::node::Node;

use super::{sheettype::SheetType, parttype::PartType};

pub struct Layout{
    id : usize,
    sheettype : usize,
    top_node : Node,
    cached_cost: RefCell<Option<Cost>>,
    usage : f64
}


impl Layout{

    pub fn new(sheettype: &SheetType){
        
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



}