use std::iter::Map;
use std::rc::{Rc, Weak};
use by_address::ByAddress;
use crate::core::entities::node::Node;
use crate::core::insertion::insertion_option::InsertionOption;
use crate::optimization::problem::Problem;
use crate::{PartType, Rotation};
use crate::util::multi_map::MultiMap;

pub fn add_insertion_options_for_parttype<'s, 'l : 's>(
    problem: &'l mut Problem, mut parttypes: Vec<usize>,
    option_node_map: &mut MultiMap<ByAddress<Rc<Node>>, Rc<InsertionOption<'s, 'l>>>,
    option_parttype_map: &mut MultiMap<usize, Rc<InsertionOption<'s, 'l>>>) {
    if parttypes.is_empty() {
        return;
    }
    let instance = problem.instance();
    //sort by decreasing area
    parttypes.sort_by(|a, b| {
        let parttype_a = instance.get_parttype(*a).unwrap();
        let parttype_b = instance.get_parttype(*b).unwrap();
        parttype_a.area().cmp(&parttype_b.area()).reverse()
    });

    for layout in problem.layouts().iter() {
        let sorted_empty_nodes = layout.get_sorted_empty_nodes();
        let mut starting_index = 0;
        for empty_node in sorted_empty_nodes.iter() {
            if instance.get_parttype(*parttypes.get(parttypes.len() - 1).unwrap()).unwrap().area() > empty_node.area() {
                //The smallest parttype is larger than this node, there are no possible insertion options left.
                break;
            }
            for i in starting_index..parttypes.len() {
                let parttype_index = *parttypes.get(i).unwrap();
                let parttype = instance.get_parttype(parttype_index).unwrap();

                if empty_node.area() < parttype.area() {
                    //The smallest parttype is larger than this node, there are no possible insertion options left.
                    starting_index += i + 1;
                } else {
                    let insertion_option = match parttype.fixed_rotation() {
                        Some(fixed_rotation) => {
                            match empty_node.insertion_possible(parttype, *fixed_rotation) {
                                true => Some(InsertionOption::new(empty_node, parttype, Some(*fixed_rotation))),
                                false => None
                            }
                        }
                        None => {
                            let default_possible = empty_node.insertion_possible(parttype, Rotation::Default);
                            let rotated_possible = empty_node.insertion_possible(parttype, Rotation::Rotated);
                            match (default_possible, rotated_possible) {
                                (true, true) => {
                                    Some(InsertionOption::new(empty_node, parttype, None))
                                }
                                (true, false) => {
                                    Some(InsertionOption::new(empty_node, parttype, Some(Rotation::Default)))
                                }
                                (false, true) => {
                                    Some(InsertionOption::new(empty_node, parttype, Some(Rotation::Rotated)))
                                }
                                (false, false) => {
                                    None
                                }
                            }
                        }
                    };
                    match insertion_option {
                        Some(insertion_option) => {
                            let insertion_option = Rc::new(insertion_option);
                            option_node_map.insert(ByAddress(empty_node.clone()), insertion_option.clone());
                            option_parttype_map.insert(parttype_index, insertion_option.clone());
                        }
                        None => {}
                    }
                }
            }
        }
    }


    todo!();
}

pub fn add_insertion_options_for_node() {
    todo!();
}

pub fn remove_insertion_options_for_parttype() {
    todo!();
}

pub fn remove_insertion_options_for_node() {
    todo!();
}

pub fn implement_insertion_blueprint() {
    todo!();
}
