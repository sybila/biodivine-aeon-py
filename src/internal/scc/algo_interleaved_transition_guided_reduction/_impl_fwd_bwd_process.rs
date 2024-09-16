use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use pyo3::PyResult;

use crate::internal::scc::algo_interleaved_transition_guided_reduction::{
    BwdProcess, FwdProcess, Process, Scheduler,
};
use crate::internal::scc::algo_saturated_reachability::reachability_step;
use crate::log_essential;

impl BwdProcess {
    pub fn new(initial: GraphColoredVertices, universe: GraphColoredVertices) -> BwdProcess {
        BwdProcess {
            universe,
            bwd: initial,
        }
    }

    pub fn get_reachable_set(&self) -> &GraphColoredVertices {
        &self.bwd
    }
}

impl FwdProcess {
    pub fn new(initial: GraphColoredVertices, universe: GraphColoredVertices) -> FwdProcess {
        FwdProcess {
            universe,
            fwd: initial,
        }
    }

    pub fn get_reachable_set(&self) -> &GraphColoredVertices {
        &self.fwd
    }
}

impl Process for BwdProcess {
    fn step(&mut self, scheduler: &mut Scheduler, graph: &SymbolicAsyncGraph) -> PyResult<bool> {
        let result = reachability_step(
            &mut self.bwd,
            &self.universe,
            scheduler.get_active_variables(),
            |var, set| graph.var_pre(var, set),
        );

        let problem_size = self.bwd.symbolic_size();
        if log_essential(scheduler.log_level, problem_size) {
            let current = self.bwd.approx_cardinality();
            let max = scheduler.universe.approx_cardinality();
            println!(
                " >> [BWD process] Reachability progress: {}[nodes:{}] candidates ({:.2} log-%).",
                current,
                problem_size,
                (current.log2() / max.log2()) * 100.0
            );
        }

        result
    }

    fn weight(&self) -> usize {
        self.bwd.symbolic_size()
    }

    fn discard_states(&mut self, set: &GraphColoredVertices) {
        self.universe = self.universe.minus(set);
        self.bwd = self.bwd.minus(set);
    }
}

impl Process for FwdProcess {
    fn step(&mut self, scheduler: &mut Scheduler, graph: &SymbolicAsyncGraph) -> PyResult<bool> {
        let result = reachability_step(
            &mut self.fwd,
            &self.universe,
            scheduler.get_active_variables(),
            |var, set| graph.var_post(var, set),
        );

        let problem_size = self.fwd.symbolic_size();
        if log_essential(scheduler.log_level, problem_size) {
            let current = self.fwd.approx_cardinality();
            let max = scheduler.universe.approx_cardinality();
            println!(
                " >> [BWD process] Reachability progress: {}[nodes:{}] candidates ({:.2} log-%).",
                current,
                problem_size,
                (current.log2() / max.log2()) * 100.0
            );
        }

        result
    }

    fn weight(&self) -> usize {
        self.fwd.symbolic_size()
    }

    fn discard_states(&mut self, set: &GraphColoredVertices) {
        self.universe = self.universe.minus(set);
        self.fwd = self.fwd.minus(set);
    }
}
