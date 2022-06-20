use std::rc::{Rc};
use by_address::ByAddress;
use crate::core::entities::node::Node;
use crate::core::insertion::insertion_option::InsertionOption;
use crate::optimization::problem::Problem;
use crate::{PartType, Rotation};
use crate::util::multi_map::MultiMap;

pub fn add_insertion_options_for_parttype<'s, 'l : 's>(
    problem: &'l Problem, mut parttypes: Vec<&'l PartType>,
    option_node_map: &mut MultiMap<ByAddress<&'s Node>, Rc<InsertionOption<'s, 'l>>>,
    option_parttype_map: &mut MultiMap<&'l PartType, Rc<InsertionOption<'s, 'l>>>) {
    if parttypes.is_empty() {
        return;
    }
    //sort by decreasing area
    parttypes.sort_by(|a, b| {
        a.area().cmp(&b.area()).reverse()
    });

    for layout in problem.layouts().iter() {
        let sorted_empty_nodes = layout.get_sorted_empty_nodes();
        let mut starting_index = 0;
        for empty_node in sorted_empty_nodes.iter() {
            if parttypes.get(parttypes.len() - 1).unwrap().area() > empty_node.area() {
                //The smallest parttype is larger than this node, there are no possible insertion options left.
                break;
            }
            for i in starting_index..parttypes.len() {
                let parttype = *parttypes.get(i).unwrap();

                if empty_node.area() < parttype.area() {
                    //The smallest parttype is larger than this node, there are no possible insertion options left.
                    starting_index += i + 1;
                } else {
                    let insertion_option = generate_insertion_option_for_node_and_parttype(empty_node, parttype);
                    match insertion_option {
                        Some(insertion_option) => {
                            let insertion_option = Rc::new(insertion_option);
                            option_node_map.insert(ByAddress(empty_node.as_ref()), insertion_option.clone());
                            option_parttype_map.insert(parttype, insertion_option.clone());
                        }
                        None => {}
                    }
                }
            }
        }
    }
}

pub fn add_insertion_options_for_node<'s, 'l : 's>(
    node: &'s Node, mut parttypes: Vec<&'l PartType>,
    option_node_map: &mut MultiMap<ByAddress<&'s Node>, Rc<InsertionOption<'s, 'l>>>,
    option_parttype_map: &mut MultiMap<&'l PartType, Rc<InsertionOption<'s, 'l>>>) {
    if node.parttype().is_none() && node.children().is_empty() {
        for parttype in parttypes {
            let insertion_option = generate_insertion_option_for_node_and_parttype(node, parttype);
            match insertion_option {
                Some(insertion_option) => {
                    let insertion_option = Rc::new(insertion_option);
                    option_node_map.insert(ByAddress(node), insertion_option.clone());
                    option_parttype_map.insert(parttype, insertion_option.clone());
                }
                None => {}
            }
        }
    }
}

fn generate_insertion_option_for_node_and_parttype<'s, 'l : 's>(node: &'s Node, parttype: &'l PartType) -> Option<InsertionOption<'s, 'l>> {
    match parttype.fixed_rotation() {
        Some(fixed_rotation) => {
            match node.insertion_possible(parttype, *fixed_rotation) {
                true => Some(InsertionOption::new(node, parttype, Some(*fixed_rotation))),
                false => None
            }
        }
        None => {
            let default_possible = node.insertion_possible(parttype, Rotation::Default);
            let rotated_possible = node.insertion_possible(parttype, Rotation::Rotated);
            match (default_possible, rotated_possible) {
                (true, true) => {
                    Some(InsertionOption::new(node, parttype, None))
                }
                (true, false) => {
                    Some(InsertionOption::new(node, parttype, Some(Rotation::Default)))
                }
                (false, true) => {
                    Some(InsertionOption::new(node, parttype, Some(Rotation::Rotated)))
                }
                (false, false) => {
                    None
                }
            }
        }
    }
}

pub fn remove_insertion_options_for_parttype<'s, 'l : 's>(
    parttype: &'l PartType,
    option_node_map: &mut MultiMap<ByAddress<&'s Node>, Rc<InsertionOption<'s, 'l>>>,
    option_parttype_map: &mut MultiMap<&'l PartType, Rc<InsertionOption<'s, 'l>>>) {

    for insert_opt in option_parttype_map.get(&parttype).unwrap() {
        option_node_map.remove(&ByAddress(insert_opt.original_node()), insert_opt);
    }
    option_parttype_map.remove_all(&parttype);

}

pub fn remove_insertion_options_for_node<'s, 'l : 's>(
    node: &'s Node,
    option_node_map: &mut MultiMap<ByAddress<Rc<Node>>, Rc<InsertionOption<'s, 'l>>>,
    option_parttype_map: &mut MultiMap<&'l PartType, Rc<InsertionOption<'s, 'l>>>) {
    todo!();
}

pub fn implement_insertion_blueprint() {
    todo!();
}
