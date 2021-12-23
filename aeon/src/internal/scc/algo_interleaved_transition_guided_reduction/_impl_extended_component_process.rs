use crate::internal::scc::algo_interleaved_transition_guided_reduction::{
    BwdProcess, ExtendedComponentProcess, Process, Scheduler,
};
use crate::internal::scc::algo_saturated_reachability::reach_bwd;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use biodivine_lib_param_bn::VariableId;

impl ExtendedComponentProcess {
    pub fn new(
        variable: VariableId,
        fwd_set: GraphColoredVertices,
        universe: GraphColoredVertices,
        graph: &SymbolicAsyncGraph,
    ) -> ExtendedComponentProcess {
        let var_can_post = graph.var_can_post(variable, &universe);
        ExtendedComponentProcess {
            variable,
            fwd_set: fwd_set.clone(),
            bwd: BwdProcess::new(var_can_post, fwd_set),
        }
    }
}

impl Process for ExtendedComponentProcess {
    fn step(&mut self, scheduler: &mut Scheduler, graph: &SymbolicAsyncGraph) -> bool {
        if self.bwd.step(scheduler, graph) {
            let extended_component = self.bwd.get_reachable_set();
            let bottom = self.fwd_set.minus(extended_component);

            if !bottom.is_empty() {
                let basin_only = reach_bwd(
                    graph,
                    &bottom,
                    scheduler.get_universe(),
                    scheduler.get_active_variables(),
                )
                .minus(&bottom);
                if !basin_only.is_empty() {
                    scheduler.discard_vertices(&basin_only);
                }
            }

            let var_can_post = graph.var_can_post(self.variable, scheduler.get_universe());
            if var_can_post.is_empty() {
                scheduler.discard_variable(self.variable);
            }

            true
        } else {
            false
        }
    }

    fn weight(&self) -> usize {
        self.bwd.weight()
    }

    fn discard_states(&mut self, set: &GraphColoredVertices) {
        self.bwd.discard_states(set);
        self.fwd_set = self.fwd_set.minus(set);
    }
}
