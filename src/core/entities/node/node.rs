use std::{rc::Rc};

use crate::core::entities::{layout::Layout};

pub struct Node {
    width : u64,
    height: u64,
    parent : Option<Rc<Node>>,
    children : Vec<Rc<Node>>,
    parttype: Option<usize>,
}



