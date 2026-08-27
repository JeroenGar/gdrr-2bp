use std::cmp::Ordering;
use std::collections::HashSet;
use std::rc::Weak;

use itertools::Itertools;
use slotmap::SlotMap;

use crate::core::entities::layout::Layout;
use crate::core::entities::node::{Node, NodeKey};
use crate::core::entities::parttype::PartType;
use crate::core::entities::sheettype::SheetType;
use crate::core::insertion::node_blueprint::NodeBlueprint;
use crate::core::layout_index::LayoutIndex;
use crate::core::orientation::Orientation;
use crate::optimization::problem::Problem;
use crate::optimization::rr::insertion_option_cache::InsertionOptionCache;
use crate::optimization::solutions::problem_solution::ProblemSolution;

/// A set of functions which ensure correct behaviour of the code. Used for debugging purposes
/// They are called with debug_assert!() macro throughout the code.
/// Are not compiled in release mode

pub fn children_nodes_fit(node_i: &NodeKey, arena: &SlotMap<NodeKey, Node>) -> bool {
    let node = &arena[*node_i];
    match node.has_children() {
        false => true,
        true => {
            match node.next_cut_orient() {
                Orientation::Horizontal => {
                    let all_children_same_width = node.children(arena).all(|c| arena[c].width() == node.width());
                    let sum_of_children_height = node.children(arena).map(|c| arena[c].height()).sum::<u64>();
                    let all_children_vert_cut_orient = node.children(arena).all(|c| arena[c].next_cut_orient() == Orientation::Vertical);

                    if !all_children_same_width || sum_of_children_height != node.height() || !all_children_vert_cut_orient {
                        return false;
                    }
                    node.children(arena).all(|c| children_nodes_fit(&c, arena))
                }
                Orientation::Vertical => {
                    let all_children_same_height = node.children(arena).all(|c| arena[c].height() == node.height());
                    let sum_of_children_width = node.children(arena).map(|c| arena[c].width()).sum::<u64>();
                    let all_children_horz_cut_orient = node.children(arena).all(|c| arena[c].next_cut_orient() == Orientation::Horizontal);


                    if !all_children_same_height || sum_of_children_width != node.width() || !all_children_horz_cut_orient {
                        return false;
                    }
                    node.children(arena).all(|c| children_nodes_fit(&c, arena))
                }
            }
        }
    }
}

pub fn children_node_blueprints_fit(node_bp: &NodeBlueprint) -> bool {
    match node_bp.children.is_empty() {
        true => true,
        false => {
            match node_bp.next_cut_orient {
                Orientation::Horizontal => {
                    let all_children_same_width = node_bp.children.iter().all(|nb| nb.width == node_bp.width);
                    let sum_of_children_height = node_bp.children.iter().map(|nb| nb.height).sum::<u64>();
                    let all_children_vert_cut_orient = node_bp.children.iter().all(|nb| nb.next_cut_orient == Orientation::Vertical);

                    if !all_children_same_width || sum_of_children_height != node_bp.height || !all_children_vert_cut_orient {
                        return false;
                    }
                    node_bp.children.iter().all(|nb| children_node_blueprints_fit(nb))
                }
                Orientation::Vertical => {
                    let all_children_same_height = node_bp.children.iter().all(|nb| nb.height == node_bp.height);
                    let sum_of_children_width = node_bp.children.iter().map(|nb| nb.width).sum::<u64>();
                    let all_children_horz_cut_orient = node_bp.children.iter().all(|nb| nb.next_cut_orient == Orientation::Horizontal);


                    if !all_children_same_height || sum_of_children_width != node_bp.width || !all_children_horz_cut_orient {
                        return false;
                    }
                    node_bp.children.iter().all(|nb| children_node_blueprints_fit(nb))
                }
            }
        }
    }
}

pub fn all_weak_references_alive<T>(values: &[Weak<T>]) -> bool {
    for value in values {
        if value.upgrade().is_none() {
            return false;
        }
    }
    return true;
}

pub fn problem_matches_solution(problem: &Problem, solution: &ProblemSolution) -> bool {
    if problem.layouts().len() != solution.layouts().len() {
        return false;
    }

    for (layout_key, layout) in problem.layouts().iter() {
        let sol_layout = solution.layouts().get(layout_key).unwrap();
        match layouts_match(layout, sol_layout) {
            true => (),
            false => {
                return false;
            }
        }
    }
    return true;
}

pub fn layouts_match(l1: &Layout, l2: &Layout) -> bool {
    if l1.sheettype() != l2.sheettype() {
        return false;
    }
    return nodes_match(l1.top_node_index(), l2.top_node_index(), l1.nodes(), l2.nodes());
}

pub fn nodes_match(n_i_1: &NodeKey, n_i_2: &NodeKey, nodes_1 : &SlotMap<NodeKey, Node>, nodes_2: &SlotMap<NodeKey, Node>) -> bool {
    let node1 = &nodes_1[*n_i_1];
    let node2 = &nodes_2[*n_i_2];
    if node1.width() != node2.width() ||
        node1.height() != node2.height() ||
        node1.parttype() != node2.parttype() ||
        node1.next_cut_orient() != node2.next_cut_orient() ||
        node1.parent().is_some() != node2.parent().is_some() {
        return false;
    }
    let mut children1 = node1.children(nodes_1);
    let mut children2 = node2.children(nodes_2);
    loop {
        match (children1.next(), children2.next()) {
            (Some(child1), Some(child2)) if nodes_match(&child1, &child2, nodes_1, nodes_2) => (),
            (None, None) => break,
            _ => return false,
        }
    }
    return true;
}

pub fn insertion_option_cache_is_valid<'a>(problem: &Problem<'a>, ioc: &InsertionOptionCache<'a>, parttypes: &[&'a PartType]) -> bool {
    //Iterate all layouts which should be considered during this recreate iteration
    let layouts_to_consider = || problem.layouts().iter().map(|(i, l)| (LayoutIndex::Existing(i), l))
        .chain(problem.empty_layouts().iter().enumerate()
            .filter(|(_, l)| problem.sheettype_qtys()[l.sheettype().id] > 0)
            .map(|(i, l)| (LayoutIndex::empty(i), l))
        );

    let mut fresh_ioc = InsertionOptionCache::new(problem.instance());

    fresh_ioc.add_for_parttypes(
        parttypes,
        layouts_to_consider(),
    );

    if ioc.is_empty() && fresh_ioc.is_empty() {
        return true;
    }

    for (i, q) in problem.parttype_qtys().iter().enumerate() {
        let parttype = problem.instance().get_parttype(i);
        match (q, parttypes.contains(&parttype)) {
            (0, true) => {
                return false;
            }
            (_, true) => {
                let ioc_options = ioc.get_for_parttype(parttype).collect_vec();
                let fresh_ioc_options = fresh_ioc.get_for_parttype(parttype).collect_vec();

                if !same_multiset(&ioc_options, &fresh_ioc_options) {
                    dbg!(ioc_options);
                    dbg!(fresh_ioc_options);
                    return false;
                }
            }
            (_,_) => ()
        }
    }

    for (layout_index, layout) in layouts_to_consider(){
        for node_index in layout.sorted_empty_nodes(){
            let node = &layout.nodes()[*node_index];
            let ioc_options = ioc.get_for_node(node_index, &layout_index)
                .filter(|option| parttypes.contains(&option.parttype()))
                .collect_vec();
            let fresh_ioc_options = fresh_ioc.get_for_node(node_index, &layout_index)
                .collect_vec();

            if !same_multiset(&ioc_options, &fresh_ioc_options) {
                dbg!(node);
                dbg!(ioc_options);
                dbg!(fresh_ioc_options);
                return false;
            }
        }
    }
    return true;
}

fn same_multiset<T: PartialEq>(left: &[T], right: &[T]) -> bool {
    left.len() == right.len() && left.iter().all(|item| {
        left.iter().filter(|candidate| *candidate == item).count()
            == right.iter().filter(|candidate| *candidate == item).count()
    })
}

pub fn cached_sorted_empty_nodes_correct(nodes: &SlotMap<NodeKey, Node>, cached_sorted_empty_nodes: &[NodeKey]) -> bool {
    let all_empty_nodes = nodes.iter().filter(|(_i,n)| n.is_empty()).map(|(i,_n)| i).collect_vec();

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

pub fn cached_used_part_area_correct(nodes: &SlotMap<NodeKey, Node>, cached_used_part_area: u64) -> bool {
    nodes.iter()
        .filter_map(|(_, node)| *node.parttype())
        .map(PartType::area)
        .sum::<u64>() == cached_used_part_area
}

pub fn cached_removable_nodes_correct(nodes: &SlotMap<NodeKey, Node>, removable_nodes: &[NodeKey]) -> bool {
    nodes.iter().all(|(node_key, node)| {
        let removable = node.parttype().is_some() || node.has_children();
        match node.removable_position() {
            Some(position) => removable && removable_nodes.get(position) == Some(&node_key),
            None => !removable,
        }
    }) && removable_nodes.iter().enumerate().all(|(position, node_key)| {
        nodes.get(*node_key).is_some_and(|node| {
            node.removable_position() == Some(position)
                && (node.parttype().is_some() || node.has_children())
        })
    })
}

pub fn instance_parttypes_and_sheettypes_ids_correct(parttypes: &[(PartType, usize)], sheettypes: &[(SheetType, usize)]) -> bool {
    parttypes.iter().enumerate().all(|(i, (p, _qty))| {
        p.id() == i
    }) && sheettypes.iter().enumerate().all(|(i, (s, _qty))| {
        s.id == i
    })
}

pub fn no_ghost_nodes_in_arena(nodes: &SlotMap<NodeKey, Node>, top_node: &NodeKey) -> bool {
    //Every node in the arena (except the top_node should be referenced by another node

    let mut buffer = vec![*top_node];
    let mut referenced_indices = HashSet::new();

    while !buffer.is_empty() {
        let index = buffer.pop().unwrap();
        if !referenced_indices.insert(index) {
            return false;
        }
        let node = &nodes[index];
        buffer.extend(node.children(nodes));
    }

    nodes.iter().all(|(i, _n)| {
        referenced_indices.contains(&i)
    })
}

pub fn node_child_parent_relations_valid(nodes: &SlotMap<NodeKey, Node>, top_node: &NodeKey) -> bool {
    // every child c of node n should have n as its parent
    // and
    // every node n should be a child of its parent p

    nodes.iter().all(|(parent_key, parent)| {
        let mut previous_child = None;
        let mut next_child = parent.first_child();
        let mut n_children = 0;

        while let Some(child_key) = next_child {
            n_children += 1;
            if n_children > nodes.len() {
                return false;
            }
            let Some(child) = nodes.get(child_key) else {
                return false;
            };
            if child.parent() != Some(parent_key) || child.previous_sibling() != previous_child {
                return false;
            }
            previous_child = Some(child_key);
            next_child = child.next_sibling();
        }

        previous_child == parent.last_child()
    }) && nodes.iter().all(|(node_key, node)| {
        match node.parent() {
            Some(parent_key) => nodes.get(parent_key)
                .is_some_and(|parent| parent.children(nodes).any(|child| child == node_key)),
            None => node_key == *top_node
                && node.previous_sibling().is_none()
                && node.next_sibling().is_none(),
        }
    })
}

pub fn node_arena_valid(nodes: &SlotMap<NodeKey, Node>, top_node: &NodeKey) -> bool {
    assert!(node_child_parent_relations_valid(nodes, top_node));
    assert!(no_ghost_nodes_in_arena(nodes, top_node));

    true
}
