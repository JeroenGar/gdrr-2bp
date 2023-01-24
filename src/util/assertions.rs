use std::borrow::Borrow;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::rc::Weak;

use generational_arena::{Arena, Index};
use itertools::Itertools;

use crate::{Orientation, PartType, SheetType};
use crate::core::entities::layout::Layout;
use crate::core::entities::node::Node;
use crate::core::insertion::node_blueprint::NodeBlueprint;
use crate::core::layout_index::LayoutIndex;
use crate::optimization::problem::Problem;
use crate::optimization::rr::insertion_option_cache::InsertionOptionCache;
use crate::optimization::solutions::problem_solution::ProblemSolution;

pub fn children_nodes_fit(node_i: &Index, arena: &Arena<Node>) -> bool {
    let node = &arena[*node_i];
    match node.children().is_empty() {
        true => true,
        false => {
            match node.next_cut_orient() {
                Orientation::Horizontal => {
                    let all_children_same_width = node.children().iter().all(|&c| arena[c].width() == node.width());
                    let sum_of_children_height = node.children().iter().map(|&c| arena[c].height()).sum::<u64>();
                    let all_children_vert_cut_orient = node.children().iter().all(|&c| arena[c].next_cut_orient() == Orientation::Vertical);

                    if !all_children_same_width || sum_of_children_height != node.height() || !all_children_vert_cut_orient {
                        return false;
                    }
                    node.children().iter().all(|c| children_nodes_fit(c, arena))
                }
                Orientation::Vertical => {
                    let all_children_same_height = node.children().iter().all(|&c| arena[c].height() == node.height());
                    let sum_of_children_width = node.children().iter().map(|&c| arena[c].width()).sum::<u64>();
                    let all_children_horz_cut_orient = node.children().iter().all(|&c| arena[c].next_cut_orient() == Orientation::Horizontal);


                    if !all_children_same_height || sum_of_children_width != node.width() || !all_children_horz_cut_orient {
                        return false;
                    }
                    node.children().iter().all(|c| children_nodes_fit(c, arena))
                }
            }
        }
    }
}

pub fn children_node_blueprints_fit(node_bp: &NodeBlueprint) -> bool {
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


                    if !all_children_same_height || sum_of_children_width != node_bp.width() || !all_children_horz_cut_orient {
                        return false;
                    }
                    node_bp.children().iter().all(|nb| children_node_blueprints_fit(nb))
                }
            }
        }
    }
}

pub fn all_weak_references_alive<T>(values: &Vec<Weak<T>>) -> bool {
    for value in values {
        if value.upgrade().is_none() {
            return false;
        }
    }
    return true;
}

pub fn problem_matches_solution(problem: &Problem, solution: &ProblemSolution) -> bool {
    if !problem.layouts().len() == solution.layouts().len() {
        return false;
    }

    for (_, layout) in problem.layouts().iter() {
        let sol_layout = solution.layouts().get(&layout.id()).unwrap();
        match layouts_match(layout, sol_layout) {
            true => (),
            false => {
                return false;
            }
        }
    }
    return true;
    todo!()
}

pub fn layouts_match(l1: &Layout, l2: &Layout) -> bool {
    if l1.sheettype() != l2.sheettype() {
        return false;
    }
    return nodes_match(l1.top_node_index(), l2.top_node_index(), l1.nodes(), l2.nodes());
}

pub fn nodes_match(n_i_1: &Index, n_i_2: &Index, nodes_1 : &Arena<Node>, nodes_2: &Arena<Node>) -> bool {
    let node1 = &nodes_1[*n_i_1];
    let node2 = &nodes_2[*n_i_2];
    if node1.width() != node2.width() ||
        node1.height() != node2.height() ||
        node1.children().len() != node2.children().len() ||
        node1.parttype() != node2.parttype() ||
        node1.next_cut_orient() != node2.next_cut_orient() ||
        node1.parent().is_some() != node2.parent().is_some() {
        return false;
    }
    for (child1, child2) in node1.children().iter().zip(node2.children().iter()) {
        if !nodes_match(child1, child2, nodes_1, nodes_2) {
            return false;
        }
    }
    return true;
}

pub fn insertion_option_cache_is_valid<'a>(problem: &Problem<'a>, ioc: &InsertionOptionCache<'a>, parttypes: &Vec<&'a PartType>) -> bool {
    //Collect all the layouts which should be considered during this recreate iteration
    let layouts_to_consider = problem.layouts().iter().map(|(i, l)| (LayoutIndex::Existing(i), l))
        .chain(problem.empty_layouts().iter().enumerate()
            .filter(|(_, l)| problem.sheettype_qtys()[l.sheettype().id()] > 0)
            .map(|(i, l)| (LayoutIndex::Empty(i), l))
        )
        .collect_vec();

    let mut fresh_ioc = InsertionOptionCache::new();

    fresh_ioc.add_for_parttypes(
        parttypes,
        &layouts_to_consider,
    );

    if ioc.is_empty() && fresh_ioc.is_empty() {
        return true;
    }

    for (i, q) in problem.parttype_qtys().iter().enumerate() {
        let parttype = problem.instance().get_parttype(i);
        match (q, parttypes.contains(&parttype)) {
            (0, false) => {
                let ioc_options = ioc.get_for_parttype(&parttype);
                let fresh_ioc_options = fresh_ioc.get_for_parttype(&parttype);

                if ioc_options.is_some() || fresh_ioc_options.is_some() {
                    return false;
                }
            }
            (0, true) => {
                return false;
            }
            (_, true) => {
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

    for (layout_index, layout) in layouts_to_consider.iter(){
        for node_index in layout.sorted_empty_nodes(){
            let node = &layout.nodes()[*node_index];
            let ioc_options = ioc.get_for_node(node_index, layout_index);
            let fresh_ioc_options = fresh_ioc.get_for_node(node_index, layout_index);

            match (ioc_options, fresh_ioc_options) {
                (None, None) => (),
                (Some(ioc_options), Some(fresh_ioc_options)) => {
                    let ioc_len = ioc_options.len();
                    let fresh_ioc_len = fresh_ioc_options.len();

                    if ioc_len != fresh_ioc_len {
                        return false;
                    }
                }
                (Some(ioc_options), None) => {
                    if !ioc_options.is_empty() {
                        dbg!(node);
                        dbg!(ioc_options);
                        dbg!(fresh_ioc_options);
                        return false;
                    }
                }
                (_, _) => {
                    let ioc_options_len = match ioc_options {
                        Some(ioc_options) => ioc_options.len(),
                        None => 0
                    };
                    let fresh_ioc_options_len = match fresh_ioc_options {
                        Some(fresh_ioc_options) => fresh_ioc_options.len(),
                        None => 0
                    };
                    if ioc_options_len != 0 || fresh_ioc_options_len != 0 {
                        dbg!(node);
                        dbg!(ioc_options);
                        dbg!(fresh_ioc_options);
                        return false;
                    }
                }
            }
        }
    }
    return true;
}

pub fn cached_sorted_empty_nodes_correct(nodes: &Arena<Node>, cached_sorted_empty_nodes: &Vec<Index>) -> bool {
    let all_empty_nodes = nodes.iter().filter(|(i,n)| n.is_empty()).map(|(i,n)| i).collect_vec();

    if all_empty_nodes.len() != cached_sorted_empty_nodes.len() {
        return false;
    }

    //ensure that all empty nodes are in the sorted list
    if !all_empty_nodes.iter().all(|n| cached_sorted_empty_nodes.contains(n)){
        return false;
    }

    //ensure that the sorted list is sorted in descending area
    let correctly_sorted = cached_sorted_empty_nodes.iter().tuples().all(|(a,b)|{
        let a = &nodes[*a];
        let b = &nodes[*b];
        a.area().cmp(&b.area()) != Ordering::Less
    });

    if !correctly_sorted {
        return false;
    }

    return true;
}

pub fn instance_parttypes_and_sheettypes_ids_correct(parttypes: &Vec<(PartType, usize)>, sheettypes: &Vec<(SheetType, usize)>) -> bool {
    parttypes.iter().enumerate().all(|(i, (p, _qty))| {
        p.id() == i
    }) && sheettypes.iter().enumerate().all(|(i, (s, _qty))| {
        s.id() == i
    })
}

pub fn no_ghost_nodes_in_arena(nodes: &Arena<Node>, top_node: &Index) -> bool {
    //Every node in the arena (except the top_node should be referenced by another node

    let mut buffer = vec![*top_node];
    let mut referenced_indices = HashSet::new();

    while !buffer.is_empty() {
        let index = buffer.pop().unwrap();
        referenced_indices.insert(index);
        let node = &nodes[index];
        buffer.extend(node.children().iter().cloned());
    }

    nodes.iter().all(|(i, _n)| {
        referenced_indices.contains(&i)
    })
}

pub fn node_child_parent_relations_valid(node: &Arena<Node>, top_node: &Index) -> bool {
    // every child c of node n should have n as its parent
    // and
    // every node n should be a child of its parent p

    node.iter().all(|(i, n)| {
        n.children().iter().all(|c| node[*c].parent() == &Some(i))
    }) &&
        node.iter().all(|(i, n)| {
            n.parent().map(|p| node[p].children().contains(&i)).unwrap_or(i == *top_node)
        })
}

pub fn node_arena_valid(nodes: &Arena<Node>, top_node: &Index) -> bool {
    assert!(no_ghost_nodes_in_arena(nodes, top_node));
    assert!(node_child_parent_relations_valid(nodes, top_node));

    true
}