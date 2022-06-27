use std::borrow::Borrow;
use std::rc::Rc;
use indexmap::IndexMap;
use crate::core::cost::Cost;
use crate::core::entities::layout::Layout;
use crate::Instance;
use crate::optimization::problem::Problem;
use crate::optimization::solutions::solution::Solution;

pub struct ProblemSolution<'a> {
    instance : &'a Instance,
    problem : &'a Problem<'a>,
    layouts : IndexMap<usize, Rc<Layout<'a>>>,
    cost : Cost,
    id : usize,
    parttype_qtys : Vec<usize>,
    sheettype_qtys : Vec<usize>
}

impl<'a> ProblemSolution<'a> {

    pub fn new(problem : &'a Problem<'a>, cost : Cost, id : usize, prev_solution : &ProblemSolution<'a>) -> ProblemSolution<'a>{
        debug_assert!(id == prev_solution.id + 1);
        debug_assert!(problem == prev_solution.problem);

        let mut layouts = IndexMap::new();

        for layout in problem.layouts() {
            let layout_ref = layout.as_ref().borrow();
            let layout_id = layout_ref.id();
            if problem.unchanged_layouts().contains(&layout_id){
                layouts.insert(layout_id, prev_solution.layouts.get(&layout_id).unwrap().clone());
            }
            else {
                layouts.insert(layout_id, Rc::new(layout_ref.create_deep_copy(layout_id)));
            }
        }

        let parttype_qtys = problem.parttype_qtys().clone();
        let sheettype_qtys = problem.sheettype_qtys().clone();

        Self {
            instance : problem.instance(),
            problem,
            layouts,
            cost,
            id,
            parttype_qtys,
            sheettype_qtys
        }
    }

    pub fn new_force_copy_all(problem : &'a Problem<'a>, cost : Cost, id : usize) -> ProblemSolution<'a>{
        let mut layouts = IndexMap::new();

        for layout in problem.layouts() {
            let layout_ref = layout.as_ref().borrow();
            layouts.insert(layout_ref.id(), Rc::new(layout_ref.create_deep_copy(layout_ref.id())));
        }

        let parttype_qtys = problem.parttype_qtys().clone();
        let sheettype_qtys = problem.sheettype_qtys().clone();

        Self {
            instance : problem.instance(),
            problem,
            layouts,
            cost,
            id,
            parttype_qtys,
            sheettype_qtys
        }
    }


    pub fn instance(&self) -> &'a Instance {
        self.instance
    }
    pub fn layouts(&self) -> &IndexMap<usize, Rc<Layout<'a>>> {
        &self.layouts
    }
    pub fn cost(&self) -> &Cost {
        &self.cost
    }
    pub fn id(&self) -> usize {
        self.id
    }
    pub fn parttype_qtys(&self) -> &Vec<usize> {
        &self.parttype_qtys
    }
    pub fn sheettype_qtys(&self) -> &Vec<usize> {
        &self.sheettype_qtys
    }
    pub fn problem(&self) -> &'a Problem<'a> {
        self.problem
    }
}

impl<'a> Solution<'a> for ProblemSolution<'a>{
    fn cost(&self) -> &Cost {
        &self.cost
    }

    fn instance(&self) -> &Instance {
        self.instance
    }

    fn layouts(&self) -> &IndexMap<usize, Rc<Layout<'a>>> {
        &self.layouts
    }

    fn parttype_qtys(&self) -> &Vec<usize> {
        &self.parttype_qtys
    }

    fn sheettype_qtys(&self) -> &Vec<usize> {
        &self.sheettype_qtys
    }
}