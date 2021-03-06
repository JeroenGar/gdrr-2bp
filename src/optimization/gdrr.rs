use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::VecDeque;
use std::rc::Rc;

use colored::*;
use rand::prelude::SliceRandom;
use rand::Rng;
use rand::rngs::{StdRng};

use crate::{Instance, PartType};
use crate::core::cost::Cost;
use crate::core::entities::layout::Layout;
use crate::core::insertion::insertion_blueprint::InsertionBlueprint;
use crate::optimization::config::Config;
use crate::optimization::problem::Problem;
use crate::optimization::rr::insertion_option_cache::InsertionOptionCache;
use crate::optimization::sol_collectors::local_sol_collector::LocalSolCollector;
use crate::optimization::solutions::problem_solution::ProblemSolution;
use crate::optimization::solutions::solution::Solution;
use crate::util::{assertions, blink};
use crate::util::biased_sampler::BiasedSampler;
use crate::util::macros::{rb, timed_thread_println};
use crate::util::util;

pub struct GDRR<'a> {
    config: &'a Config,
    instance: &'a Instance,
    problem: Problem<'a>,
    cost_comparator: fn(&Cost, &Cost) -> Ordering,
    local_sol_collector: LocalSolCollector<'a>,
}


impl<'a> GDRR<'a> {
    pub fn new(instance: &'a Instance, config: &'a Config, local_sol_collector: LocalSolCollector<'a>) -> Self {
        let problem = Problem::new(instance);
        let cost_comparator = crate::COST_COMPARATOR;
        Self {
            config,
            instance,
            problem,
            cost_comparator,
            local_sol_collector,
        }
    }

    pub fn lahc(&'a mut self) {
        let start_time = std::time::Instant::now();

        let max_rr_iterations = self.config.max_rr_iterations.unwrap_or(usize::MAX);

        let empty_problem_cost = Cost::new(0, 0.0, self.instance.total_part_area(), 0);

        let mut lahc_history: VecDeque<Cost> = VecDeque::with_capacity(self.config.history_length);
        lahc_history.push_back(empty_problem_cost.clone());
        let mut n_iterations = 0;
        let mut n_accepted = 0;
        let mut n_improved = 0;
        let mut mat_limit = self.local_sol_collector.material_limit();
        let mut local_optimum: Option<ProblemSolution> = None;

        while n_iterations < max_rr_iterations && !self.local_sol_collector.terminate() {
            let mat_limit_budget: i128 = match local_optimum.as_ref() {
                Some(solution) => mat_limit as i128 - 1 - solution.cost().material_cost as i128,
                None => mat_limit as i128 - 1 - self.problem.cost().material_cost as i128,
            };

            let mat_limit_budget = self.ruin(mat_limit_budget);
            let max_part_area_not_included = match local_optimum.as_ref() {
                Some(local_optimum) => u64::max(lahc_history.front().unwrap().part_area_excluded, local_optimum.cost().part_area_excluded),
                None => lahc_history.front().unwrap().part_area_excluded
            };

            self.recreate(mat_limit_budget, max_part_area_not_included);

            let cost = self.problem.cost();

            if (self.cost_comparator)(&cost, lahc_history.front().unwrap()) <= Ordering::Equal ||
                (local_optimum.is_some() && (self.cost_comparator)(&cost, local_optimum.as_ref().unwrap().cost()) <= Ordering::Equal) {
                //Solution is better or equivalent to the last entry in the history queue or the local optimum.

                local_optimum = Some(self.problem.create_solution(&local_optimum, Some(cost.clone())));

                lahc_history.pop_front();

                if (self.cost_comparator)(&cost, lahc_history.back().unwrap_or(&empty_problem_cost)) == Ordering::Less {
                    //Current local optimum is better than the last value of the history queue
                    for _ in 0..(self.config.history_length - lahc_history.len()) {
                        lahc_history.push_back(cost.clone());
                    }
                    self.local_sol_collector.report_problem_solution(local_optimum.as_ref().unwrap());
                    n_improved += 1;
                } else {
                    //Current local optimum is not better, add the best cost to the history queue
                    for _ in 0..(self.config.history_length - lahc_history.len()) {
                        lahc_history.push_back(lahc_history.back().unwrap().clone());
                    }
                }
                n_accepted += 1;
            } else {
                self.problem.restore_from_problem_solution(local_optimum.as_ref().unwrap());
            }

            if self.local_sol_collector.material_limit() < mat_limit {
                mat_limit = self.local_sol_collector.material_limit();
                local_optimum = None;
                lahc_history.clear();
                lahc_history.push_back(empty_problem_cost.clone());
            }
            n_iterations += 1;
            if n_iterations % 100 == 0 {
                self.local_sol_collector.rx_sync();
            }

            debug_assert!(lahc_history.len() <= self.config.history_length, "{}", lahc_history.len());
        }
        timed_thread_println!("{}:\t ({:.2} iter/s, {:.2} acc/s, {} impr)",
                "GDRR finished".bright_magenta(),
                 (n_iterations as f64 / (std::time::Instant::now() - start_time).as_millis() as f64 * 1000.0),
                 (n_accepted as f64 / (std::time::Instant::now() - start_time).as_millis() as f64 * 1000.0),
                n_improved
        );
        timed_thread_println!("{}:\t {}", "Final incomp".bright_yellow(),
            match self.local_sol_collector.best_incomplete_solution().as_ref() {
                Some(sol) => {
                    util::solution_stats_string(sol)
                }
                None => "()".to_string()
            });
    }

    fn ruin(&mut self, mut mat_limit_budget: i128) -> i128 {
        let n_nodes_to_remove = self.problem.random().gen_range(2..(self.config.avg_nodes_removed - 2) * 2 + 1) + 2;

        if mat_limit_budget >= 0 {
            for _i in 0..n_nodes_to_remove {
                let biased_sampler = BiasedSampler::new_default(
                    self.problem.layouts().iter().map(|l| { Rc::downgrade(l) }).collect(),
                    |a: &RefCell<Layout>, b: &RefCell<Layout>| { a.borrow().usage().partial_cmp(&b.borrow().usage()).unwrap().reverse() },
                );

                let layout = biased_sampler.sample(&mut self.problem.random());

                match layout {
                    Some(layout) => {
                        let removable_nodes = rb!(layout).get_removable_nodes();
                        let selected_node = removable_nodes.choose(&mut self.problem.random()).unwrap().upgrade().unwrap();

                        mat_limit_budget += self.problem.remove_node(&selected_node, &layout) as i128;
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
                    let usage_a = rb!(a).usage();
                    let usage_b = rb!(b).usage();
                    usage_a.partial_cmp(&usage_b).unwrap()
                }).unwrap().clone();

                //release it and update mat_limit_exceedance
                mat_limit_budget += self.problem.remove_node(rb!(layout_min_usage).top_node(), &layout_min_usage) as i128;
            }
        }
        mat_limit_budget
    }

    fn recreate(&mut self, mut mat_limit_budget: i128, max_part_area_excluded: u64) {
        let mut parttypes_to_consider: Vec<&PartType> = self.problem.parttype_qtys().iter().enumerate()
            .filter(|(_i, q)| { **q > 0 })
            .map(|(i, _q)| -> &PartType { self.problem.instance().get_parttype(i) }).collect();


        let mut insertion_option_cache = InsertionOptionCache::new();
        let mut part_area_not_included: u64 = 0;

        //Collect all the layouts which should be considered during this recreate iteration
        let mut layouts_to_consider = Vec::new();
        layouts_to_consider.extend(self.problem.layouts().iter().cloned());
        layouts_to_consider.extend(self.problem.empty_layouts().iter()
            .filter(|l| { *self.problem.sheettype_qtys().get(rb!(l).sheettype().id()).unwrap() > 0 })
            .cloned());


        //Generate insertion options for all relevant parttypes and layouts
        insertion_option_cache.add_for_parttypes(parttypes_to_consider.iter(), &layouts_to_consider);
        debug_assert!(assertions::insertion_option_cache_is_valid(&self.problem, &insertion_option_cache, &parttypes_to_consider));

        while !parttypes_to_consider.is_empty() && part_area_not_included <= max_part_area_excluded {
            let elected_parttype = GDRR::select_next_parttype(&self.instance, &mut parttypes_to_consider, &insertion_option_cache, self.problem.random(), &self.config);
            let elected_blueprint = GDRR::select_insertion_blueprint(elected_parttype, &insertion_option_cache, mat_limit_budget, self.problem.random(), &self.config, &self.cost_comparator);

            if elected_blueprint.is_some() {
                let elected_blueprint_sheettype_id = rb!(elected_blueprint.as_ref().unwrap().layout().as_ref().unwrap().upgrade().unwrap()).sheettype().id();

                let (cache_updates, blueprint_created_new_layout) =
                    self.problem.implement_insertion_blueprint(elected_blueprint.as_ref().unwrap());
                insertion_option_cache.update_cache(&cache_updates, &parttypes_to_consider);

                if blueprint_created_new_layout {
                    //update mat_limit_budget
                    mat_limit_budget -= self.instance.get_sheettype(elected_blueprint_sheettype_id).value() as i128;
                    //remove the relevant empty_layout from consideration if the stock is empty
                    if *self.problem.sheettype_qtys().get(elected_blueprint_sheettype_id).unwrap() == 0 {
                        self.problem.empty_layouts().iter()
                            .filter(|l| {
                                rb!(l).sheettype().id() == elected_blueprint_sheettype_id
                            }).for_each(|l| {
                            insertion_option_cache.remove_for_layout(l);
                        });
                    }
                }
                if *self.problem.parttype_qtys().get(elected_parttype.id()).unwrap() == 0 {
                    //if the parttype is not needed anymore, remove it from the cache
                    insertion_option_cache.remove_for_parttype(elected_parttype);
                    parttypes_to_consider.retain(|pt| { pt.id() != elected_parttype.id() });
                }

                if insertion_option_cache.is_empty() {
                    break;
                }

                debug_assert!(assertions::insertion_option_cache_is_valid(&self.problem, &insertion_option_cache, &parttypes_to_consider), "{:#?}\n{:#?}", elected_blueprint, cache_updates);
            } else {
                //if there is no insertion blueprint, the part cannot be added to the problem
                part_area_not_included += *self.problem.parttype_qtys().get(elected_parttype.id()).unwrap() as u64
                    * elected_parttype.area();
                insertion_option_cache.remove_for_parttype(elected_parttype);

                parttypes_to_consider.retain(|pt| { pt.id() != elected_parttype.id() });

                debug_assert!(assertions::insertion_option_cache_is_valid(&self.problem, &insertion_option_cache, &parttypes_to_consider), "{:#?}", elected_blueprint);
            }
        }
    }

    fn select_next_parttype<'b : 'a>(instance: &'b Instance, parttypes: &mut Vec<&'a PartType>, insertion_option_cache: &InsertionOptionCache<'a>, rand: &mut StdRng, config: &Config) -> &'b PartType {
        parttypes.shuffle(rand);
        let n_options: Vec<usize> = parttypes.iter().map(|p| {
            match insertion_option_cache.get_for_parttype(*p) {
                Some(options) => {
                    options.len()
                }
                None => {
                    0
                }
            }
        }).collect();

        let selected_index = blink::select_lowest_entry(&n_options, config.blink_rate, rand);
        let selected_parttype_id = parttypes[selected_index].id();

        instance.get_parttype(selected_parttype_id)
    }

    fn select_insertion_blueprint(parttype: &'a PartType, insertion_option_cache: &InsertionOptionCache<'a>, mat_limit_budget: i128, rand: &mut StdRng, config: &Config, cost_comparator: &fn(&Cost, &Cost) -> Ordering) -> Option<InsertionBlueprint<'a>> {
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
                    if rb!(option.layout().upgrade().unwrap()).is_empty() {
                        if mat_limit_budget >= rb!(option.layout().upgrade().unwrap()).sheettype().value() as i128 {
                            new_layout_blueprints.extend(option.get_blueprints());
                        }
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
                        let selected_blinked_index = blink::select_lowest_in_range(0..existing_layout_blueprints.len(), config.blink_rate, rand);
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