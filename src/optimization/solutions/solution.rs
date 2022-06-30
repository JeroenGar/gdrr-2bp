use crate::core::cost::Cost;

pub trait Solution{
    fn cost(&self) -> &Cost;

    fn n_layouts(&self) -> usize;

    fn parttype_qtys(&self) -> &Vec<usize>;

    fn sheettype_qtys(&self) -> &Vec<usize>;

    fn is_complete(&self) -> bool{
        self.cost().part_area_excluded == 0
    }

    fn usage(&self) -> f64;
}