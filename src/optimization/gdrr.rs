use std::borrow::Borrow;
use rand::Rng;
use crate::Instance;
use crate::optimization::config::Config;
use crate::optimization::problem::Problem;

pub struct GDRR<'a> {
    config : Config,
    instance : &'a Instance,
    problem : Problem<'a>
}


impl<'a> GDRR<'a> {

    pub fn lahc(){
        todo!();
    }

    pub fn ruin(&mut self, mut mat_limit_exceedance: u64){
        let n_nodes_to_remove = self.problem.random().gen_range(2..(self.config.avg_nodes_removed() - 2) * 2 + 1);

        if mat_limit_exceedance <= 0 {

        }
        else{
            while mat_limit_exceedance > 0 {
                if self.problem.layouts().is_empty() {
                    break;
                }
                //Search the worst layout
                let layout_min_usage = self.problem.layouts().iter().min_by(|a,b| {
                    let usage_a = a.as_ref().borrow().get_usage();
                    let usage_b = b.as_ref().borrow().get_usage();
                    usage_a.partial_cmp(&usage_b).unwrap()
                }).unwrap().clone();

                //release it and update mat_limit_exceedance
                mat_limit_exceedance -= layout_min_usage.as_ref().borrow().sheettype().value();
                self.problem.release_layout(&layout_min_usage);
            }
        }

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