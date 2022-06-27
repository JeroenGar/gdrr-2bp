use std::rc::Rc;
use downcast_rs::DowncastSync;
use indexmap::IndexMap;
use crate::core::cost::Cost;
use crate::core::entities::layout::Layout;
use crate::Instance;

pub trait Solution<'a>{
    fn cost(&self) -> &Cost;

    fn instance(&self) -> &Instance;

    fn layouts(&self) -> &IndexMap<usize, Rc<Layout<'a>>>;

    fn parttype_qtys(&self) -> &Vec<usize>;

    fn sheettype_qtys(&self) -> &Vec<usize>;

}