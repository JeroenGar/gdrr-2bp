use std::borrow::Borrow;
use std::cell::{Ref, RefCell};
use std::ops::Deref;
use std::rc::{Rc, Weak};

use crate::core::entities::layout::Layout;
use crate::core::entities::node::Node;
use crate::core::insertion::node_blueprint::NodeBlueprint;
use crate::optimization::problem::Problem;
use crate::Orientation;

pub fn node_belongs_to_layout<'a>(node: &Rc<RefCell<Node<'a>>>, layout: &Rc<RefCell<Layout<'a>>>) -> bool {
    node_belongs_to_owner(node, layout.as_ref().borrow().top_node())
}


pub fn node_belongs_to_owner<'a>(node: &Rc<RefCell<Node<'a>>>, owner_node: &Rc<RefCell<Node<'a>>>) -> bool {
    let owner_ref = owner_node.as_ref().borrow();
    match owner_ref.children().is_empty() {
        true => false,
        false => {
            match owner_ref.children().iter().any(|c| Rc::ptr_eq(c, node)) {
                true => true,
                false => {
                    owner_ref.children().iter().any(|c| node_belongs_to_owner(node, c))
                }
            }
        }
    }
}

pub fn layout_belongs_to_problem<'a>(layout: &Rc<RefCell<Layout<'a>>>, problem: &Problem<'a>) -> bool {
    problem.layouts().iter().any(|l| Rc::ptr_eq(l, layout))
}

pub fn children_nodes_fit(node: &Node) -> bool {

    match node.children().is_empty() {
        true => true,
        false => {
            match node.next_cut_orient() {
                Orientation::Horizontal => {
                    let all_children_same_width = node.children().iter().all(|c| c.as_ref().borrow().width() == node.width());
                    let sum_of_children_height = node.children().iter().map(|c| c.as_ref().borrow().height()).sum::<u64>();
                    let all_children_vert_cut_orient = node.children().iter().all(|c| c.as_ref().borrow().next_cut_orient() == Orientation::Vertical);

                    if !all_children_same_width || sum_of_children_height != node.height() || !all_children_vert_cut_orient {
                        return false;
                    }
                    node.children().iter().all(|c| children_nodes_fit(c.as_ref().borrow().deref()))
                }
                Orientation::Vertical => {
                    let all_children_same_height = node.children().iter().all(|c| c.as_ref().borrow().height() == node.height());
                    let sum_of_children_width = node.children().iter().map(|c| c.as_ref().borrow().width()).sum::<u64>();
                    let all_children_horz_cut_orient = node.children().iter().all(|c| c.as_ref().borrow().next_cut_orient() == Orientation::Horizontal);


                    if !all_children_same_height || sum_of_children_width != node.width() || !all_children_horz_cut_orient{
                        return false;
                    }
                    node.children().iter().all(|c| children_nodes_fit(c.as_ref().borrow().deref()))
                }
            }
        }
    }
}

pub fn replacements_fit(original_node : &Weak<RefCell<Node>>, replacements : &Vec<NodeBlueprint>) -> bool {
    let node = original_node.upgrade().unwrap();
    let node_ref = node.as_ref().borrow();

    if replacements.iter().any(|r| r.next_cut_orient() != node_ref.next_cut_orient()) {
        return false;
    }

    match node_ref.next_cut_orient(){
        Orientation::Horizontal => {
            let all_replacements_same_height = replacements.iter().all(|nb| nb.height() == node_ref.height());
            let sum_of_replacements_width = replacements.iter().map(|nb| nb.width()).sum::<u64>();


            if !all_replacements_same_height || sum_of_replacements_width != node_ref.width() {
                return false;
            }
        }
        Orientation::Vertical => {
            let all_replacements_same_width = replacements.iter().all(|nb| nb.width() == node_ref.width());
            let sum_of_replacements_height = replacements.iter().map(|nb| nb.height()).sum::<u64>();

            if !all_replacements_same_width || sum_of_replacements_height != node_ref.height() {
                return false;
            }
        }
    }

    replacements.iter().all(|nb| children_node_blueprints_fit(nb))

}

pub fn children_node_blueprints_fit(node_bp : &NodeBlueprint) -> bool {
    match node_bp.children().is_empty() {
        true => true,
        false => {
            match node_bp.next_cut_orient() {
                Orientation::Horizontal => {
                    let all_children_same_width = node_bp.children().iter().all(|nb| nb.width() == node_bp.width());
                    let sum_of_children_height = node_bp.children().iter().map(|nb| nb.height()).sum::<u64>();
                    let all_children_vert_cut_orient = node_bp.children().iter().all(|nb| nb.next_cut_orient() == Orientation::Vertical);

                    if !all_children_same_width || sum_of_children_height != node_bp.height() || !all_children_vert_cut_orient {
                        return false;
                    }
                    node_bp.children().iter().all(|nb| children_node_blueprints_fit(nb))
                }
                Orientation::Vertical => {
                    let all_children_same_height = node_bp.children().iter().all(|nb| nb.height() == node_bp.height());
                    let sum_of_children_width = node_bp.children().iter().map(|nb| nb.width()).sum::<u64>();
                    let all_children_horz_cut_orient = node_bp.children().iter().all(|nb| nb.next_cut_orient() == Orientation::Horizontal);


                    if !all_children_same_height || sum_of_children_width != node_bp.width() || !all_children_horz_cut_orient{
                        return false;
                    }
                    node_bp.children().iter().all(|nb| children_node_blueprints_fit(nb))
                }
            }
        }
    }
}

pub fn nodes_sorted_descending_area(nodes : &Vec<Weak<RefCell<Node>>>) -> bool {
    let mut prev_area = u64::MAX;

    for node in nodes {
        let area = node.upgrade().unwrap().as_ref().borrow().area();
        if area > prev_area {
            return false;
        }
        else{
            prev_area = area;
        }
    }
    return true;
}

pub fn all_weak_references_alive<T>(values: &Vec<Weak<T>>) -> bool{
    for value in values {
        if value.upgrade().is_none() {
            return false;
        }
    }
    return true;
}