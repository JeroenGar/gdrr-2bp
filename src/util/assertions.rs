use std::any::Any;
use std::borrow::Borrow;
use std::cell::{Ref, RefCell};
use std::ops::Deref;
use std::rc::{Rc, Weak};

use crate::core::entities::layout::Layout;
use crate::core::entities::node::Node;
use crate::core::insertion::node_blueprint::NodeBlueprint;
use crate::optimization::problem::Problem;
use crate::optimization::solutions::solution::Solution;
use crate::Orientation;
use crate::util::macros::{rb,rbm};


pub fn node_belongs_to_layout<'a>(node: &Rc<RefCell<Node<'a>>>, layout: &Layout<'a>) -> bool {
    Rc::ptr_eq(node, layout.top_node()) || node_belongs_to_owner(node, layout.top_node())
}


pub fn node_belongs_to_owner<'a>(node: &Rc<RefCell<Node<'a>>>, owner_node: &Rc<RefCell<Node<'a>>>) -> bool {
    let owner_ref = rb!(owner_node);
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
                    let all_children_same_width = node.children().iter().all(|c| rb!(c).width() == node.width());
                    let sum_of_children_height = node.children().iter().map(|c| rb!(c).height()).sum::<u64>();
                    let all_children_vert_cut_orient = node.children().iter().all(|c| rb!(c).next_cut_orient() == Orientation::Vertical);

                    if !all_children_same_width || sum_of_children_height != node.height() || !all_children_vert_cut_orient {
                        return false;
                    }
                    node.children().iter().all(|c| children_nodes_fit(rb!(c).deref()))
                }
                Orientation::Vertical => {
                    let all_children_same_height = node.children().iter().all(|c| rb!(c).height() == node.height());
                    let sum_of_children_width = node.children().iter().map(|c| rb!(c).width()).sum::<u64>();
                    let all_children_horz_cut_orient = node.children().iter().all(|c| rb!(c).next_cut_orient() == Orientation::Horizontal);


                    if !all_children_same_height || sum_of_children_width != node.width() || !all_children_horz_cut_orient{
                        return false;
                    }
                    node.children().iter().all(|c| children_nodes_fit(rb!(c).deref()))
                }
            }
        }
    }
}

pub fn replacements_fit(original_node : &Weak<RefCell<Node>>, replacements : &Vec<NodeBlueprint>) -> bool {
    let node = original_node.upgrade().unwrap();
    let node_ref = rb!(node);

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
        let area = rb!(node.upgrade().unwrap()).area();
        if area > prev_area {
            return false;
        }
        else{
            prev_area = area;
        }
    }
    return true;
}

pub fn all_nodes_have_parents(nodes : &Vec<Weak<RefCell<Node>>>) -> bool {
    nodes.iter().all(|n| rb!(n.upgrade().unwrap()).parent().is_some())
}

pub fn all_weak_references_alive<T>(values: &Vec<Weak<T>>) -> bool{
    for value in values {
        if value.upgrade().is_none() {
            return false;
        }
    }
    return true;
}

pub fn problem_matches_solution(problem : &Problem, solution : &dyn Solution) -> bool {
    for layout in problem.layouts().iter() {
        let sol_layout = solution.layouts().get(&rb!(layout).id()).unwrap();
        match layouts_match(rb!(layout).deref(), rb!(sol_layout).deref()) {
            true => (),
            false => {
                return false
            }
        }
    }
    return true;
}

pub fn layouts_match(layout1 : &Layout, layout2 : &Layout) -> bool {
    if layout1.sheettype() != layout2.sheettype() {
        return false;
    }
    return nodes_match(rb!(layout1.top_node()).deref(), rb!(layout2.top_node()).deref());
}

pub fn nodes_match(node1 : &Node, node2 : &Node) -> bool {
    if node1.width() != node2.width() ||
        node1.height() != node2.height() ||
        node1.children().len() != node2.children().len() ||
        node1.parttype() != node2.parttype() ||
        node1.next_cut_orient() != node2.next_cut_orient() ||
        node1.parent().is_some() != node2.parent().is_some() {
        return false;
    }
    for (child1, child2) in node1.children().iter().zip(node2.children().iter()) {
        if !nodes_match(rb!(child1).deref(), rb!(child2).deref()) {
            return false;
        }
    }
    return true;
}