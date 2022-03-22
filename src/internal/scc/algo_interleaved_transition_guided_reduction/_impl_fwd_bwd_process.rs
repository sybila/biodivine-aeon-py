use crate::internal::scc::algo_interleaved_transition_guided_reduction::{
    BwdProcess, FwdProcess, Process, Scheduler,
};
use crate::internal::scc::algo_saturated_reachability::reachability_step;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};

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
    fn step(&mut self, scheduler: &mut Scheduler, graph: &SymbolicAsyncGraph) -> bool {
        reachability_step(
            &mut self.bwd,
            &self.universe,
            scheduler.get_active_variables(),
            |var, set| graph.var_pre(var, set),
        )
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
    fn step(&mut self, scheduler: &mut Scheduler, graph: &SymbolicAsyncGraph) -> bool {
        reachability_step(
            &mut self.fwd,
            &self.universe,
            scheduler.get_active_variables(),
            |var, set| graph.var_post(var, set),
        )
    }

    fn weight(&self) -> usize {
        self.fwd.symbolic_size()
    }

    fn discard_states(&mut self, set: &GraphColoredVertices) {
        self.universe = self.universe.minus(set);
        self.fwd = self.fwd.minus(set);
    }
}
