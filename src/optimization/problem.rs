use std::cell::RefCell;
use std::collections::{LinkedList};
use std::ops::Deref;
use std::rc::Rc;
use indexmap::IndexMap;
use crate::{Instance, PartType, SheetType};
use crate::core::entities::layout::Layout;
use crate::core::entities::node::Node;
use crate::core::insertion::insertion_blueprint::InsertionBlueprint;
use crate::optimization::rr::cache_updates::CacheUpdates;
use crate::util::assertions;

pub struct Problem<'a> {
    instance : &'a Instance,
    parttype_qtys: Vec<usize>,
    sheettype_qtys : Vec<usize>,
    layouts : Vec<Rc<RefCell<Layout<'a>>>>,
    empty_layouts : Vec<Rc<RefCell<Layout<'a>>>>,
    random : rand::rngs::ThreadRng,
    counter_layout_id : usize
}

impl<'a> Problem<'a> {
    pub fn new(instance: &'a Instance) -> Self {
        let parttype_qtys = instance.parts().iter().map(|(_, qty)| *qty).collect::<Vec<_>>();
        let sheettype_qtys = instance.sheets().iter().map(|(_, qty)| *qty).collect::<Vec<_>>();
        let layouts = Vec::new();
        let empty_layouts = Vec::new();
        let random = rand::thread_rng();
        let counter_layout_id = 0;

        Self { instance, parttype_qtys, sheettype_qtys, layouts, empty_layouts, random, counter_layout_id }
    }

    pub fn implement_insertion_blueprint(&mut self, blueprint: &'a InsertionBlueprint<'a>) -> CacheUpdates<Rc<RefCell<Node<'a>>>>{
        let blueprint_layout = blueprint.layout().as_ref().unwrap().upgrade().unwrap();

        let blueprint_uses_existing_layout = !self.empty_layouts.iter().any(|e| Rc::ptr_eq(e, &blueprint_layout));

        let cache_updates = match blueprint_uses_existing_layout {
            true => {
                self.register_part(blueprint.parttype(), 1);
                let cache_updates = blueprint_layout.borrow_mut().implement_insertion_blueprint(blueprint);
                cache_updates
            }
            false => {
                let copy = blueprint_layout.borrow().create_deep_copy();
                //Create a copy of the insertion blueprint and map it to the copy of the layout
                let mut insertion_bp_copy = blueprint.clone();
                //Modify so the original node maps a node of the copied layout
                insertion_bp_copy.set_original_node(Rc::downgrade(&copy.top_node().as_ref().borrow().children().first().unwrap()));
                let copy = Rc::new(RefCell::new(copy));
                insertion_bp_copy.set_layout(Rc::downgrade(&copy));
                let cache_updates = copy.as_ref().borrow_mut().implement_insertion_blueprint(&insertion_bp_copy);

                self.register_layout(&copy);

                cache_updates
            }
        };

        cache_updates

    }

    pub fn remove_node(&mut self, node: &Rc<RefCell<Node<'a>>>, layout: &Rc<RefCell<Layout<'a>>>) {
        debug_assert!(assertions::node_belongs_to_layout(node, layout));
        debug_assert!(assertions::layout_belongs_to_problem(layout, self));

        let mut layout_ref = layout.as_ref().borrow_mut();
        let released_parts = layout_ref.remove_node(node);

        released_parts.iter().for_each(|p| {self.release_part(p, 1)});
        if layout_ref.is_empty() {
            self.release_layout(layout);
        }
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

    pub fn random(&mut self) -> &mut rand::rngs::ThreadRng {
        &mut self.random
    }

    pub fn layouts(&self) -> &Vec<Rc<RefCell<Layout<'a>>>> {
        &self.layouts
    }

    pub fn register_layout(&mut self, layout: &Rc<RefCell<Layout<'a>>>) {
        todo!(); //register parts & sheets
        self.layouts.push(layout.clone());
    }

    pub fn release_layout(&mut self, layout: &Rc<RefCell<Layout<'a>>>) {
        debug_assert!(assertions::layout_belongs_to_problem(layout, self));
        todo!(); //register parts & sheets
        self.layouts.retain(|l| !Rc::ptr_eq(l, layout));
    }

    fn register_part(&mut self, parttype : &'a PartType, qty : usize) {
        let id = parttype.id().unwrap();
        debug_assert!(self.parttype_qtys[id] - qty >= 0);

        self.parttype_qtys[parttype.id().unwrap()] -= qty;
    }

    fn release_part(&mut self, parttype : &'a PartType, qty : usize) {
        let id = parttype.id().unwrap();
        debug_assert!(self.parttype_qtys[id] + qty <= self.instance.get_parttype_qty(id).unwrap());

        self.parttype_qtys[id] += qty;
    }

    fn register_sheet(&mut self, sheettype : &'a SheetType, qty : usize) {
        let id = sheettype.id().unwrap();
        debug_assert!(self.sheettype_qtys[id] - qty >= 0);

        self.sheettype_qtys[id] -= qty;
    }

    fn release_sheet(&mut self, sheettype : &'a SheetType, qty : usize) {
        let id = sheettype.id().unwrap();
        debug_assert!(self.sheettype_qtys[id] + qty <= self.instance.get_sheettype_qty(id).unwrap());

        self.sheettype_qtys[id] += qty;
    }

    fn get_layout_id(&mut self) -> usize {
        self.counter_layout_id += 1;
        self.counter_layout_id
    }

}