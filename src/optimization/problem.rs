use std::collections::{LinkedList};
use std::ops::Deref;
use indexmap::IndexMap;
use crate::{Instance, PartType, SheetType};
use crate::core::entities::layout::Layout;
use crate::core::insertion::insertion_blueprint::InsertionBlueprint;

pub struct Problem<'a> {
    instance : &'a Instance,
    parttype_qtys: Vec<usize>,
    sheettype_qtys : Vec<usize>,
    layouts : Vec<Layout>,
    empty_layouts : Vec<Layout>,
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
        let blueprint_layout_ptr = blueprint.original_node().layout().as_ptr();
        let layout_index = self.layouts.iter().position(
            |l| l as *const Layout == blueprint_layout_ptr)
            .unwrap();

        let mut layout = self.layouts.get_mut(layout_index).unwrap();
        layout.implement_insertion_blueprint(blueprint);
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
    pub fn empty_layouts(&self) -> &Vec<Layout> {
        &self.empty_layouts
    }
    pub fn random(&self) -> &rand::rngs::ThreadRng {
        &self.random
    }
    pub fn layouts(&self) -> &Vec<Layout> {
        &self.layouts
    }
}