use std::borrow::Borrow;
use std::cell::RefCell;
use std::rc::Rc;
use crate::core::entities::layout::Layout;
use crate::core::entities::node::Node;
use crate::optimization::problem::Problem;

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