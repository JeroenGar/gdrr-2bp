use std::borrow::Borrow;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::path::Iter;
use std::rc::Rc;
use indexmap::{IndexMap, IndexSet};
use rand::Rng;
use crate::core::cost::Cost;
use crate::{Instance, PartType};
use crate::core::entities::layout::Layout;
use crate::core::insertion::insertion_blueprint::InsertionBlueprint;
use crate::core::insertion::insertion_option::InsertionOption;
use crate::optimization::config::Config;
use crate::optimization::problem::Problem;
use crate::optimization::rr::insertion_option_cache::InsertionOptionCache;
use crate::util::multi_map::MultiMap;

pub struct GDRR<'a> {
    config: Config,
    instance: &'a Instance,
    problem: Problem<'a>,
    cost_comparator: fn(&Cost, &Cost) -> Ordering
}


impl<'a> GDRR<'a> {
    pub fn lahc() {
        todo!();
    }

    pub fn ruin(&'a mut self, mut mat_limit_budget: u64) {
        let n_nodes_to_remove = self.problem.random().gen_range(2..(self.config.avg_nodes_removed() - 2) * 2 + 1);

        if mat_limit_budget >= 0 {} else {
            while mat_limit_budget < 0 {
                if self.problem.layouts().is_empty() {
                    break;
                }
                //Search the worst layout
                let layout_min_usage = self.problem.layouts().iter().min_by(|a, b| {
                    let usage_a = a.as_ref().borrow().get_usage();
                    let usage_b = b.as_ref().borrow().get_usage();
                    usage_a.partial_cmp(&usage_b).unwrap()
                }).unwrap().clone();

                //release it and update mat_limit_exceedance
                mat_limit_budget += layout_min_usage.as_ref().borrow().sheettype().value();
                self.problem.release_layout(&layout_min_usage);
            }
        }
    }

    pub fn recreate(&'a mut self, mut mat_limit_budget: u64, max_part_area_not_included: u64) {
        let mut parttypes_to_consider: IndexSet<&PartType> = self.problem.parttype_qtys().iter().enumerate()
            .filter(|(i, q)| { **q > 0 })
            .map(|(i, q)| -> &PartType { self.problem.instance().get_parttype(i).unwrap() }).collect();


        let mut insertion_option_cache = InsertionOptionCache::new();
        let mut part_area_not_included: u64 = 0;

        //Collect all the layouts which should be considered during this recreate iteration
        let mut layouts_to_consider = Vec::new();
        layouts_to_consider.extend(self.problem.layouts().iter().cloned());
        layouts_to_consider.extend(self.problem.empty_layouts().iter()
                .filter(|l| { *self.problem.sheettype_qtys().get(l.as_ref().borrow().sheettype().id().unwrap()).unwrap() > 0 })
                .cloned());


        //Generate insertion options for all relevant parttypes and layouts
        insertion_option_cache.add_for_parttypes(parttypes_to_consider.iter(), &layouts_to_consider);

        while !parttypes_to_consider.is_empty() && part_area_not_included <= max_part_area_not_included {

            let elected_parttype_id = self.select_next_parttype(&parttypes_to_consider, &insertion_option_cache);
            let elected_blueprint = GDRR::select_insertion_blueprint(elected_parttype_id, &insertion_option_cache, mat_limit_budget);

            if elected_blueprint.is_some(){

                let elected_blueprint_sheettype_id =  elected_blueprint.as_ref().unwrap().layout().as_ref().unwrap().upgrade().unwrap().as_ref().borrow().sheettype().id().unwrap();

                let (cache_updates, blueprint_created_new_layout) =
                    self.problem.implement_insertion_blueprint(elected_blueprint.as_ref().unwrap());
                insertion_option_cache.update_cache(&cache_updates, &parttypes_to_consider);

                if blueprint_created_new_layout {
                    //update mat_limit_budget
                    mat_limit_budget -= self.instance.get_sheettype(elected_blueprint_sheettype_id).unwrap().value();
                    //remove the relevant empty_layout from consideration if the stock is empty
                    if *self.problem.sheettype_qtys().get(elected_blueprint_sheettype_id).unwrap() == 0 {
                        self.problem.empty_layouts().iter()
                            .filter(|l| { l.as_ref().borrow().sheettype().id().unwrap() == elected_blueprint_sheettype_id
                        }).for_each(|l| { insertion_option_cache.remove_for_layout(l);
                        });
                    }
                }
            }
            else{
                //if there is no insertion blueprint, the part cannot be added to the problem
                part_area_not_included += *self.problem.parttype_qtys().get(elected_parttype_id).unwrap() as u64
                    * self.instance.get_parttype(elected_parttype_id).unwrap().area();
            }

            if elected_blueprint.is_none() || *self.problem.parttype_qtys().get(elected_parttype_id).unwrap() == 0 {
                //if the parttype could not be added, or if the parttype is not needed anymore, remove it from the cache
                insertion_option_cache.remove_for_parttype(self.instance.get_parttype(elected_parttype_id).unwrap());
                parttypes_to_consider.remove(self.instance.get_parttype(elected_parttype_id).unwrap());
            }
        }


        todo!();
    }

    fn select_next_parttype(&self, parttypes: &IndexSet<&PartType>, insertion_option_cache : &InsertionOptionCache) -> usize {
        todo!();
    }

    fn select_insertion_blueprint(parttype : usize, insertion_option_cache : &InsertionOptionCache<'a>, mut mat_limit_budget : u64) -> Option<Rc<InsertionBlueprint<'a>>> {
        todo!();
    }
}