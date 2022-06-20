use crate::Instance;
use crate::optimization::config::Config;
use crate::optimization::problem::Problem;

pub struct RR<'a> {
    config : Config,
    instance : &'a Instance,
    problem : Problem<'a>
}


impl<'a> RR<'a> {

    pub fn ruin(){
        todo!();
    }

    pub fn recreate(){
        todo!();
    }

    fn best_fit_insert_parts(){
        todo!();
    }

    fn select_insertion_blueprint(){
        todo!();
    }

}