use std::borrow::Borrow;
use std::cell::RefCell;
use std::rc::Rc;
use crate::core::entities::layout::Layout;
use crate::core::entities::node::Node;
use crate::optimization::problem::Problem;
use crate::Orientation;

pub fn node_belongs_to_layout<'a>(node: &Rc<RefCell<Node<'a>>>, layout: &Rc<RefCell<Layout<'a>>>) -> bool {
    node_belongs_to_owner(node, layout.as_ref().borrow().top_node())
}


pub fn node_belongs_to_owner<'a>(node: &Rc<RefCell<Node<'a>>>, owner_node: &Rc<RefCell<Node<'a>>>) -> bool {
    let owner_ref = owner_node.as_ref().borrow();
    match owner_ref.children().is_empty(){
        true => false,
        false => {
            match owner_ref.children().iter().any(|c| Rc::ptr_eq(c, node)){
                true => true,
                false => {
                    owner_ref.children().iter().any(|c| node_belongs_to_owner(node, c))
                }
            }
        }
    }
}

pub fn layout_belongs_to_problem<'a>(layout: &Rc<RefCell<Layout<'a>>>, problem : &Problem<'a>) -> bool {
    problem.layouts().iter().any(|l| Rc::ptr_eq(l, layout))
}

pub fn children_nodes_fit(node: &Rc<RefCell<Node>>) -> bool{
    let node_ref = node.as_ref().borrow();

    match node_ref.children().is_empty() {
        true => true,
        false => {
            match node_ref.next_cut_orient() {
                Orientation::Horizontal => {
                    let all_children_same_width = node_ref.children().iter().all(|c| c.as_ref().borrow().width() == node_ref.width());
                    let sum_of_children_height = node_ref.children().iter().map(|c| c.as_ref().borrow().height()).sum::<u64>();

                    if !all_children_same_width || sum_of_children_height != node_ref.height() {
                        return false;
                    }
                    node_ref.children().iter().all(|c| children_nodes_fit(c))
                }
                Orientation::Vertical => {
                    let all_children_same_height = node_ref.children().iter().all(|c| c.as_ref().borrow().height() == node_ref.height());
                    let sum_of_children_width = node_ref.children().iter().map(|c| c.as_ref().borrow().width()).sum::<u64>();

                    if !all_children_same_height || sum_of_children_width != node_ref.width() {
                        return false;
                    }
                    node_ref.children().iter().all(|c| children_nodes_fit(c))
                }
            }
        }
    }
}