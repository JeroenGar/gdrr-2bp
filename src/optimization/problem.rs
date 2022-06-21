use std::cell::RefCell;
use std::collections::{LinkedList};
use std::ops::Deref;
use std::rc::Rc;
use indexmap::IndexMap;
use crate::{Instance, PartType, SheetType};
use crate::core::entities::layout::Layout;
use crate::core::insertion::insertion_blueprint::InsertionBlueprint;

pub struct Problem<'a> {
    instance : &'a Instance,
    parttype_qtys: Vec<usize>,
    sheettype_qtys : Vec<usize>,
    layouts : Vec<Rc<RefCell<Layout>>>,
    empty_layouts : Vec<Rc<Layout>>,
    random : rand::rngs::ThreadRng,
}

impl<'a> Problem<'a> {
    pub fn new(instance: &'a Instance) -> Self {
        let parttype_qtys = instance.parts().iter().map(|(_, qty)| *qty).collect::<Vec<_>>();
        let sheettype_qtys = instance.sheets().iter().map(|(_, qty)| *qty).collect::<Vec<_>>();
        let layouts = Vec::new();
        let empty_layouts = Vec::new();
        let random = rand::thread_rng();

        Self { instance, parttype_qtys, sheettype_qtys, layouts, empty_layouts, random }
    }

    pub fn implement_insertion_blueprint(&mut self, blueprint: &InsertionBlueprint) {
        let blueprint_layout = blueprint.layout().as_ref().unwrap().upgrade().unwrap();
        blueprint_layout.borrow_mut().implement_insertion_blueprint(blueprint);
        todo!()
    }


    pub fn instance(&self) -> &'a Instance {
        self.instance
    }
    pub fn parttype_qtys(&self) -> &Vec<usize> {
        &self.parttype_qtys
    }
    pub fn sheettype_qtys(&self) -> &Vec<usize> {
        &self.sheettype_qtys
   }
    pub fn empty_layouts(&self) -> &Vec<Rc<Layout>> {
        &self.empty_layouts
    }
    pub fn random(&self) -> &rand::rngs::ThreadRng {
        &self.random
    }

    pub fn layouts(&self) -> &Vec<Rc<RefCell<Layout>>> {
        &self.layouts
    }
}