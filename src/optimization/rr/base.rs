use std::borrow::Borrow;
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use by_address::ByAddress;
use crate::core::entities::node::Node;
use crate::core::insertion::insertion_option::InsertionOption;
use crate::optimization::problem::Problem;
use crate::{PartType, Rotation};
use crate::core::entities::layout::Layout;
use crate::core::insertion::insertion_blueprint::InsertionBlueprint;
use crate::optimization::rr::cache_updates::CacheUpdates;
use crate::util::multi_map::MultiMap;

pub fn add_insertion_options_for_parttype<'a>(
    problem: &'a Problem<'a>, mut parttypes: Vec<&'a PartType>,
    option_node_map: &mut MultiMap<ByAddress<Rc<RefCell<Node<'a>>>>, Rc<InsertionOption<'a>>>,
    option_parttype_map: &mut MultiMap<&'a PartType, Rc<InsertionOption<'a>>>) {
    if parttypes.is_empty() {
        return;
    }
    //sort by decreasing area
    parttypes.sort_by(|a, b| {
        a.area().cmp(&b.area()).reverse()
    });

    for layout in problem.layouts().iter() {
        let layout = layout.as_ref().borrow();
        let sorted_empty_nodes = layout.get_sorted_empty_nodes();
        let mut starting_index = 0;
        for empty_node in sorted_empty_nodes.iter() {
            let empty_node = empty_node.upgrade().unwrap();
            let empty_node_ref = empty_node.as_ref().borrow();
            if parttypes.get(parttypes.len() - 1).unwrap().area() > empty_node_ref.area() {
                //The smallest parttype is larger than this node, there are no possible insertion options left.
                break;
            }
            for i in starting_index..parttypes.len() {
                let parttype = *parttypes.get(i).unwrap();

                if empty_node_ref.area() < parttype.area() {
                    //The smallest parttype is larger than this node, there are no possible insertion options left.
                    starting_index += i + 1;
                } else {
                    let insertion_option = generate_insertion_option_for_node_and_parttype(&empty_node, parttype);
                    match insertion_option {
                        Some(insertion_option) => {
                            let insertion_option = Rc::new(insertion_option);
                            option_node_map.insert(ByAddress(empty_node.clone()), insertion_option.clone());
                            option_parttype_map.insert(parttype, insertion_option.clone());
                        }
                        None => {}
                    }
                }
            }
        }
    }
}

pub fn add_insertion_options_for_node<'a>(
    node: &Rc<RefCell<Node<'a>>>, parttypes: Vec<&'a PartType>,
    option_node_map: &mut MultiMap<ByAddress<Rc<RefCell<Node<'a>>>>, Rc<InsertionOption<'a>>>,
    option_parttype_map: &mut MultiMap<&'a PartType, Rc<InsertionOption<'a>>>) {
    let node_ref = node.as_ref().borrow();
    if node_ref.parttype().is_none() && node_ref.children().is_empty() {
        for parttype in parttypes {
            let insertion_option = generate_insertion_option_for_node_and_parttype(node, parttype);
            match insertion_option {
                Some(insertion_option) => {
                    let insertion_option = Rc::new(insertion_option);
                    option_node_map.insert(ByAddress(node.clone()), insertion_option.clone());
                    option_parttype_map.insert(parttype, insertion_option.clone());
                }
                None => {}
            }
        }
    }
}

fn generate_insertion_option_for_node_and_parttype<'a>(node: &Rc<RefCell<Node<'a>>>, parttype: &'a PartType) -> Option<InsertionOption<'a>> {
    let node_ref = node.as_ref().borrow();
    match parttype.fixed_rotation() {
        Some(fixed_rotation) => {
            match node_ref.insertion_possible(parttype, *fixed_rotation) {
                true => Some(InsertionOption::new(Rc::downgrade(&node), parttype, Some(*fixed_rotation))),
                false => None
            }
        }
        None => {
            let default_possible = node_ref.insertion_possible(parttype, Rotation::Default);
            let rotated_possible = node_ref.insertion_possible(parttype, Rotation::Rotated);
            match (default_possible, rotated_possible) {
                (true, true) => {
                    Some(InsertionOption::new(Rc::downgrade(&node), parttype, None))
                }
                (true, false) => {
                    Some(InsertionOption::new(Rc::downgrade(&node), parttype, Some(Rotation::Default)))
                }
                (false, true) => {
                    Some(InsertionOption::new(Rc::downgrade(&node), parttype, Some(Rotation::Rotated)))
                }
                (false, false) => {
                    None
                }
            }
        }
    }
}

pub fn remove_insertion_options_for_parttype<'a>(
    parttype: &'a PartType,
    option_node_map: &mut MultiMap<ByAddress<Rc<RefCell<Node<'a>>>>, Rc<InsertionOption<'a>>>,
    option_parttype_map: &'a mut MultiMap<&'a PartType, Rc<InsertionOption<'a>>>) {
    for insert_opt in option_parttype_map.get(&parttype).unwrap() {
        option_node_map.remove(&ByAddress(insert_opt.original_node().upgrade().unwrap()), insert_opt);
    }
    option_parttype_map.remove_all(&parttype);
}

pub fn remove_insertion_options_for_node<'a>(
    node: Rc<RefCell<Node<'a>>>,
    option_node_map: &mut MultiMap<ByAddress<Rc<RefCell<Node<'a>>>>, Rc<InsertionOption<'a>>>,
    option_parttype_map: &'a mut MultiMap<&'a PartType, Rc<InsertionOption<'a>>>) {
    //todo!();

    for insert_opt in option_node_map.get(&ByAddress(node.clone())).unwrap() {
        option_parttype_map.remove(&insert_opt.parttype(), insert_opt);
    };
    option_node_map.remove_all(&ByAddress(node.clone()));
}

pub fn implement_insertion_blueprint<'a>(
    problem: &mut Problem,
    insertion_blueprint: &'a InsertionBlueprint,
    mat_limit_margin: u64)
    -> (u64, CacheUpdates<Rc<RefCell<Node<'a>>>>) {
    todo!();
}
