use std::cell::RefCell;
use std::collections::LinkedList;
use std::ops::Deref;
use std::rc::{Rc, Weak};

use indexmap::{IndexMap, IndexSet};

use crate::{Instance, PartType, SheetType};
use crate::core::entities::layout::Layout;
use crate::core::entities::node::Node;
use crate::core::insertion::insertion_blueprint::InsertionBlueprint;
use crate::optimization::rr::cache_updates::CacheUpdates;
use crate::util::assertions;

pub struct Problem<'a> {
    instance: &'a Instance,
    parttype_qtys: Vec<usize>,
    sheettype_qtys: Vec<usize>,
    layouts: Vec<Rc<RefCell<Layout<'a>>>>,
    empty_layouts: Vec<Rc<RefCell<Layout<'a>>>>,
    unchanged_layouts : IndexSet<usize>,
    random: rand::rngs::ThreadRng,
    counter_layout_id: usize,
    counter_solution_id: usize
}

impl<'a> Problem<'a> {
    pub fn new(instance: &'a Instance) -> Self {
        let parttype_qtys = instance.parts().iter().map(|(_, qty)| *qty).collect::<Vec<_>>();
        let sheettype_qtys = instance.sheets().iter().map(|(_, qty)| *qty).collect::<Vec<_>>();
        let layouts = Vec::new();
        let empty_layouts = Vec::new();
        let unchanged_layouts = IndexSet::new();
        let random = rand::thread_rng();
        let counter_layout_id = 0;
        let counter_solution_id = 0;

        Self {
            instance,
            parttype_qtys,
            sheettype_qtys,
            layouts,
            empty_layouts,
            unchanged_layouts,
            random,
            counter_layout_id,
            counter_solution_id
        }
    }

    pub fn implement_insertion_blueprint(&mut self, blueprint: &InsertionBlueprint<'a>) -> (CacheUpdates<'a, Weak<RefCell<Node<'a>>>>, bool) {
        let blueprint_layout = blueprint.layout().as_ref().unwrap().upgrade().unwrap();

        let blueprint_creates_new_layout = self.empty_layouts.iter().any(|e| Rc::ptr_eq(e, &blueprint_layout));

        let cache_updates = match blueprint_creates_new_layout {
            false => {
                self.register_part(blueprint.parttype(), 1);
                let mut cache_updates = CacheUpdates::new(Rc::downgrade(&blueprint_layout));
                blueprint_layout.borrow_mut().implement_insertion_blueprint(blueprint, &mut cache_updates);

                cache_updates
            }
            true => {
                let copy = blueprint_layout.borrow().create_deep_copy(self.next_layout_id());
                //Create a copy of the insertion blueprint and map it to the copy of the layout
                let mut insertion_bp_copy = blueprint.clone();
                //Modify so the blueprint so the original node maps to the respective node of the copied layout
                let modified_original_node = copy.top_node().as_ref().borrow().children().first().unwrap().clone();
                insertion_bp_copy.set_original_node(Rc::downgrade(&modified_original_node));
                //wrap the copied layout
                let copy = Rc::new(RefCell::new(copy));
                insertion_bp_copy.set_layout(Rc::downgrade(&copy));
                self.register_layout(copy.clone());

                //Search the layout again in the problem, to please the borrow checker
                let copy = self.layouts.iter().find(|l| Rc::ptr_eq(l, &copy)).unwrap();

                let mut cache_updates = CacheUpdates::new(Rc::downgrade(&copy));
                copy.as_ref().borrow_mut().implement_insertion_blueprint(&insertion_bp_copy, &mut cache_updates);

                cache_updates
            }
        };

        (cache_updates, blueprint_creates_new_layout)
    }

    pub fn remove_node(&mut self, node: &Rc<RefCell<Node<'a>>>, layout: &Rc<RefCell<Layout<'a>>>) {
        debug_assert!(assertions::node_belongs_to_layout(node, layout));
        debug_assert!(assertions::layout_belongs_to_problem(layout, self));

        let mut layout_ref = layout.as_ref().borrow_mut();

        if Rc::ptr_eq(node, layout_ref.top_node()) {
            //The node to remove is the root node of the layout, so the entire layout is removed
            self.unregister_layout(layout);
        } else {
            let removed_node = layout_ref.remove_node(node);
            let mut parts_to_release = Vec::new();
            removed_node.as_ref().borrow().get_included_parts(&mut parts_to_release);
            parts_to_release.iter().for_each(|p| { self.unregister_part(p, 1) });

            if layout_ref.is_empty() {
                self.unregister_layout(layout);
            }
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

    pub fn register_layout(&mut self, layout: Rc<RefCell<Layout<'a>>>) {
        self.register_sheet(layout.borrow().sheettype(), 1);
        layout.borrow().get_included_parts().iter().for_each(
            |p| { self.register_part(p, 1) });

        self.layouts.push(layout);
    }

    pub fn unregister_layout(&mut self, layout: &Rc<RefCell<Layout<'a>>>) {
        debug_assert!(assertions::layout_belongs_to_problem(layout, self));

        self.unregister_sheet(layout.borrow().sheettype(), 1);
        layout.borrow().get_included_parts().iter().for_each(
            |p| { self.unregister_part(p, 1) });

        self.layouts.retain(|l| !Rc::ptr_eq(l, layout));
    }

    fn register_part(&mut self, parttype: &'a PartType, qty: usize) {
        let id = parttype.id();
        debug_assert!(self.parttype_qtys[id] - qty >= 0);

        self.parttype_qtys[parttype.id()] -= qty;
    }

    fn unregister_part(&mut self, parttype: &'a PartType, qty: usize) {
        let id = parttype.id();
        debug_assert!(self.parttype_qtys[id] + qty <= self.instance.get_parttype_qty(id).unwrap());

        self.parttype_qtys[id] += qty;
    }

    fn register_sheet(&mut self, sheettype: &'a SheetType, qty: usize) {
        let id = sheettype.id();
        debug_assert!(self.sheettype_qtys[id] - qty >= 0);

        self.sheettype_qtys[id] -= qty;
    }

    fn unregister_sheet(&mut self, sheettype: &'a SheetType, qty: usize) {
        let id = sheettype.id();
        debug_assert!(self.sheettype_qtys[id] + qty <= self.instance.get_sheettype_qty(id).unwrap());

        self.sheettype_qtys[id] += qty;
    }

    fn next_layout_id(&mut self) -> usize {
        self.counter_layout_id += 1;
        self.counter_layout_id
    }

    fn next_solution_id(&mut self) -> usize {
        self.counter_solution_id += 1;
        self.counter_solution_id
    }


    pub fn empty_layouts(&self) -> &Vec<Rc<RefCell<Layout<'a>>>> {
        &self.empty_layouts
    }

    pub fn counter_layout_id(&self) -> usize {
        self.counter_layout_id
    }


    pub fn unchanged_layouts(&self) -> &IndexSet<usize> {
        &self.unchanged_layouts
    }
    pub fn counter_solution_id(&self) -> usize {
        self.counter_solution_id
    }
}

impl<'a> PartialEq for Problem<'a> {
    fn eq(&self, other: &Problem<'a>) -> bool {
        std::ptr::eq(self, other)
    }
}