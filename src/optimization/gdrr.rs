use std::borrow::Borrow;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::path::Iter;
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};
use rand::prelude::SliceRandom;
use rand::Rng;
use rand::rngs::ThreadRng;

use crate::{Instance, PartType};
use crate::core::cost::Cost;
use crate::core::entities::layout::Layout;
use crate::core::insertion::insertion_blueprint::InsertionBlueprint;
use crate::core::insertion::insertion_option::InsertionOption;
use crate::optimization::config::Config;
use crate::optimization::problem::Problem;
use crate::optimization::rr::insertion_option_cache::InsertionOptionCache;
use crate::util::biased_sampler::BiasedSampler;
use crate::util::blink;
use crate::util::multi_map::MultiMap;

pub struct GDRR<'a> {
    config: Config,
    instance: &'a Instance,
    problem: Problem<'a>,
    cost_comparator: fn(&Cost, &Cost) -> Ordering,
}


impl<'a> GDRR<'a> {
    pub fn lahc() {
        todo!();
    }

    pub fn ruin(&'a mut self, mut mat_limit_budget: u64) {
        let n_nodes_to_remove = self.problem.random().gen_range(2..(self.config.avg_nodes_removed() - 2) * 2 + 1) + 2;

        if mat_limit_budget >= 0 {
            for i in 0..n_nodes_to_remove {
                let reversed_layout_usage_comparator = |a: &RefCell<Layout>, b: &RefCell<Layout>| { a.borrow().get_usage().partial_cmp(&b.borrow().get_usage()).unwrap().reverse() };

                let biased_sampler = BiasedSampler::new_default(
                    self.problem.layouts().iter().map(|l| { Rc::downgrade(l) }).collect(),
                    reversed_layout_usage_comparator,
                );

                let layout = biased_sampler.sample(&mut self.problem.random());

                match layout {
                    Some(layout) => {
                        let layout = layout.upgrade().unwrap();
                        let mut layout_ref = layout.as_ref().borrow_mut();
                        let removable_nodes = layout_ref.get_removable_nodes();
                        let selected_node = removable_nodes.choose(&mut self.problem.random()).unwrap().upgrade().unwrap();

                        self.problem.remove_node(&selected_node, &layout);
                    }
                    None => { break; }
                }
            }
        } else {
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
                self.problem.unregister_layout(&layout_min_usage);
            }
        }
    }

    pub fn recreate(&'a mut self, mut mat_limit_budget: u64, max_part_area_not_included: u64) {
        let mut parttypes_to_consider: IndexSet<&PartType> = self.problem.parttype_qtys().iter().enumerate()
            .filter(|(i, q)| { **q > 0 })
            .map(|(i, q)| -> &PartType { self.problem.instance().get_parttype(i) }).collect();


        let mut insertion_option_cache = InsertionOptionCache::new();
        let mut part_area_not_included: u64 = 0;

        //Collect all the layouts which should be considered during this recreate iteration
        let mut layouts_to_consider = Vec::new();
        layouts_to_consider.extend(self.problem.layouts().iter().cloned());
        layouts_to_consider.extend(self.problem.empty_layouts().iter()
            .filter(|l| { *self.problem.sheettype_qtys().get(l.as_ref().borrow().sheettype().id()).unwrap() > 0 })
            .cloned());


        //Generate insertion options for all relevant parttypes and layouts
        insertion_option_cache.add_for_parttypes(parttypes_to_consider.iter(), &layouts_to_consider);

        while !parttypes_to_consider.is_empty() && part_area_not_included <= max_part_area_not_included {
            let elected_parttype = GDRR::select_next_parttype(&self.instance, &parttypes_to_consider, &insertion_option_cache, self.problem.random(), &self.config);
            let elected_blueprint = GDRR::select_insertion_blueprint(elected_parttype, &insertion_option_cache, mat_limit_budget, self.problem.random(), &self.config, &self.cost_comparator);

            if elected_blueprint.is_some() {
                let elected_blueprint_sheettype_id = elected_blueprint.as_ref().unwrap().layout().as_ref().unwrap().upgrade().unwrap().as_ref().borrow().sheettype().id();

                let (cache_updates, blueprint_created_new_layout) =
                    self.problem.implement_insertion_blueprint(elected_blueprint.as_ref().unwrap());
                insertion_option_cache.update_cache(&cache_updates, &parttypes_to_consider);

                if blueprint_created_new_layout {
                    //update mat_limit_budget
                    mat_limit_budget -= self.instance.get_sheettype(elected_blueprint_sheettype_id).value();
                    //remove the relevant empty_layout from consideration if the stock is empty
                    if *self.problem.sheettype_qtys().get(elected_blueprint_sheettype_id).unwrap() == 0 {
                        self.problem.empty_layouts().iter()
                            .filter(|l| {
                                l.as_ref().borrow().sheettype().id() == elected_blueprint_sheettype_id
                            }).for_each(|l| {
                            insertion_option_cache.remove_for_layout(l);
                        });
                    }
                }
            } else {
                //if there is no insertion blueprint, the part cannot be added to the problem
                part_area_not_included += *self.problem.parttype_qtys().get(elected_parttype.id()).unwrap() as u64
                    * elected_parttype.area();
            }

            if elected_blueprint.is_none() || *self.problem.parttype_qtys().get(elected_parttype.id()).unwrap() == 0 {
                //if the parttype could not be added, or if the parttype is not needed anymore, remove it from the cache
                insertion_option_cache.remove_for_parttype(elected_parttype);
                parttypes_to_consider.remove(elected_parttype);
            }
        }


        todo!();
    }

    fn select_next_parttype<'b : 'a>(instance: &'b Instance, parttypes: &IndexSet<&'a PartType>, insertion_option_cache: &InsertionOptionCache<'a>, rand: &mut ThreadRng, config: &Config) -> &'b PartType {
        let mut parttypes_to_consider = parttypes.iter().map(|p| { p.id() }).collect::<Vec<_>>();
        parttypes_to_consider.shuffle(rand);

        parttypes_to_consider.sort_by(|a, b| {
            let a_insertion_options = match insertion_option_cache.get_for_parttype(instance.get_parttype(*a)) {
                Some(options) => options.len(),
                None => 0
            };
            let b_insertion_options = match insertion_option_cache.get_for_parttype(instance.get_parttype(*b)) {
                Some(options) => options.len(),
                None => 0
            };
            a_insertion_options.cmp(&b_insertion_options)
        });

        let selected_index = blink::select_lowest(0..parttypes_to_consider.len(), config.blink_chance(), rand);
        let selected_parttype_id = parttypes_to_consider[selected_index];

        instance.get_parttype(selected_parttype_id)
    }

    fn select_insertion_blueprint(parttype: &'a PartType, insertion_option_cache: &InsertionOptionCache<'a>, mut mat_limit_budget: u64, rand: &mut ThreadRng, config: &Config, cost_comparator: &fn(&Cost, &Cost) -> Ordering) -> Option<InsertionBlueprint<'a>> {
        let insertion_options = insertion_option_cache.get_for_parttype(parttype);
        match insertion_options {
            Some(options) => {
                //Collect the blueprints
                let mut existing_layout_blueprints: Vec<InsertionBlueprint<'a>> = Vec::new();
                let mut new_layout_blueprints: Vec<InsertionBlueprint<'a>> = Vec::new();

                for option in options {
                    if existing_layout_blueprints.len() > 20 {
                        break; //enough blueprints to consider
                    }
                    if option.layout().upgrade().unwrap().as_ref().borrow().is_empty() &&
                        mat_limit_budget >= option.layout().upgrade().unwrap().as_ref().borrow().sheettype().value() {
                        new_layout_blueprints.extend(option.get_blueprints());
                    } else {
                        existing_layout_blueprints.extend(option.get_blueprints());
                    }
                }
                match existing_layout_blueprints.is_empty() {
                    false => {
                        //Sort the blueprints by cost
                        existing_layout_blueprints.sort_by(|a, b| {
                            cost_comparator(a.cost(), b.cost())
                        });
                        //Select the best (blinked) one
                        let selected_blinked_index = blink::select_lowest(0..existing_layout_blueprints.len(), config.blink_chance(), rand);
                        Some(existing_layout_blueprints.remove(selected_blinked_index))
                    }
                    true => {
                        //No blueprints for existing layouts, try new layouts
                        match new_layout_blueprints.is_empty() {
                            true => {
                                //No insertion blueprint available
                                None
                            }
                            false => {
                                //Select a random blueprint from the new layout blueprints
                                let selected_index = rand.gen_range(0..new_layout_blueprints.len());
                                Some(new_layout_blueprints.remove(selected_index))
                            }
                        }
                    }
                }
            }
            None => {
                None
            }
        }
    }
}