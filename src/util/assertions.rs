use std::borrow::Borrow;
use std::cell::{Ref, RefCell};
use std::ops::Deref;
use std::rc::{Rc, Weak};

use crate::core::entities::layout::Layout;
use crate::core::entities::node::Node;
use crate::core::insertion::node_blueprint::NodeBlueprint;
use crate::optimization::problem::Problem;
use crate::optimization::rr::insertion_option_cache::InsertionOptionCache;
use crate::optimization::solutions::solution::Solution;
use crate::{Orientation, PartType};
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

pub fn insertion_option_cache_is_valid<'a>(problem : &Problem<'a>, ioc : &InsertionOptionCache<'a>, parttypes : &Vec<&'a PartType>) -> bool{
    let mut layouts_to_consider = Vec::new();
    layouts_to_consider.extend(problem.layouts().iter().map(|l| {l.clone()}));
    layouts_to_consider.extend(problem.empty_layouts().iter()
        .filter(|l| { *problem.sheettype_qtys().get(rb!(l).sheettype().id()).unwrap() > 0 })
        .map(|l| {l.clone()}));

    let mut fresh_ioc = InsertionOptionCache::new();

    fresh_ioc.add_for_parttypes(
        parttypes.iter(),
                &layouts_to_consider
    );

    if ioc.is_empty() && fresh_ioc.is_empty() {
        return true;
    }

    for (i,q) in problem.parttype_qtys().iter().enumerate(){
        let parttype = problem.instance().get_parttype(i);
        match (q, parttypes.contains(&parttype)) {
            (0,false) => {
                let ioc_options = ioc.get_for_parttype(&parttype);
                let fresh_ioc_options = fresh_ioc.get_for_parttype(&parttype);

                if ioc_options.is_some() || fresh_ioc_options.is_some() {
                    return false;
                }
            }
            (0,true) => {
                return false;
            }
            (_,true) => {
                let ioc_options = ioc.get_for_parttype(&parttype);
                let fresh_ioc_options = fresh_ioc.get_for_parttype(&parttype);
                let n_ioc_options = match ioc_options {
                    Some(ioc_options) => ioc_options.len(),
                    None => 0
                };
                let n_fresh_ioc_options = match fresh_ioc_options {
                    Some(fresh_ioc_options) => fresh_ioc_options.len(),
                    None => 0
                };

                if n_ioc_options != n_fresh_ioc_options {
                    dbg!(ioc_options);
                    dbg!(fresh_ioc_options);
                    return false;
                }
            }
            (_,_) => ()
        }
    }

    let borrowed_layouts = layouts_to_consider.iter().map(|l| rb!(l)).collect::<Vec<Ref<Layout>>>();
    for node in borrowed_layouts.iter().map(|l| { l.sorted_empty_nodes() }).flatten() {
        let ioc_options = ioc.get_for_node(&node.upgrade().unwrap());
        let fresh_ioc_options = fresh_ioc.get_for_node(&node.upgrade().unwrap());

        match (ioc_options, fresh_ioc_options) {
            (None, None) => (),
            (Some(ioc_options), Some(fresh_ioc_options)) => {
                let ioc_len = ioc_options.len();
                let fresh_ioc_len = fresh_ioc_options.len();

                if ioc_len != fresh_ioc_len {
                    return false;
                }
            }
            (Some(ioc_options),None) => {
                if !ioc_options.is_empty() {
                    dbg!(node.upgrade().unwrap());
                    dbg!(ioc_options);
                    dbg!(fresh_ioc_options);
                    return false;
                }
            }
            (_,_) => {
                let ioc_options_len = match ioc_options {
                    Some(ioc_options) => ioc_options.len(),
                    None => 0
                };
                let fresh_ioc_options_len = match fresh_ioc_options {
                    Some(fresh_ioc_options) => fresh_ioc_options.len(),
                    None => 0
                };
                if ioc_options_len != 0 || fresh_ioc_options_len != 0 {
                    dbg!(node.upgrade().unwrap());
                    dbg!(ioc_options);
                    dbg!(fresh_ioc_options);
                    return false;
                }
            }
        }
    }
    return true;
}

pub fn cached_empty_nodes_correct<'a>(layout : &Layout<'a>, cached_empty_nodes : &Vec<Weak<RefCell<Node<'a>>>>) -> bool {
    let mut all_children = Vec::new();
    rb!(layout.top_node()).get_all_children(&mut all_children);
    let all_empty_children = all_children.iter().filter(|n| {
        rb!(n.upgrade().unwrap()).is_empty()
    }).collect::<Vec<_>>();

    if all_empty_children.len() != cached_empty_nodes.len() {
        return false;
    }

    for empty_node in all_empty_children {
        if !cached_empty_nodes.iter().any(|n| {
            Weak::ptr_eq(empty_node, n)
        }) {
            return false;
        }
    }

    return true;
}