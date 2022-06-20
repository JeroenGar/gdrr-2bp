use std::collections::{LinkedList};
use indexmap::IndexMap;
use crate::{Instance, PartType, SheetType};
use crate::core::entities::layout::Layout;

pub struct Problem<'a> {
    instance : &'a Instance,
    parttype_qtys: Vec<usize>,
    sheettype_qtys : Vec<usize>,
    layouts : LinkedList<Layout>,
    empty_layouts : Vec<Layout>,
    random : rand::rngs::ThreadRng,
}

impl<'a> Problem<'a> {
    pub fn new(instance: &'a Instance) -> Self {
        let parttype_qtys = instance.parts().iter().map(|(_, qty)| *qty).collect::<Vec<_>>();
        let sheettype_qtys = instance.sheets().iter().map(|(_, qty)| *qty).collect::<Vec<_>>();
        let layouts = LinkedList::new();
        let empty_layouts = Vec::new();
        let random = rand::thread_rng();

        Self { instance, parttype_qtys, sheettype_qtys, layouts, empty_layouts, random }
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
    pub fn layouts(&self) -> &LinkedList<Layout> {
        &self.layouts
    }
    pub fn empty_layouts(&self) -> &Vec<Layout> {
        &self.empty_layouts
    }
    pub fn random(&self) -> &rand::rngs::ThreadRng {
        &self.random
    }
}