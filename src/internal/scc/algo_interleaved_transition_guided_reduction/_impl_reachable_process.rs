use biodivine_lib_param_bn::VariableId;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use pyo3::PyResult;

use crate::internal::scc::algo_interleaved_transition_guided_reduction::{
    ExtendedComponentProcess, FwdProcess, Process, ReachableProcess, Scheduler,
};
use crate::internal::scc::algo_saturated_reachability::reach_bwd;
use crate::{log_essential, should_log};

impl ReachableProcess {
    pub fn new(
        var: VariableId,
        graph: &SymbolicAsyncGraph,
        universe: GraphColoredVertices,
    ) -> ReachableProcess {
        let var_can_post = graph.var_can_post(var, &universe);
        ReachableProcess {
            variable: var,
            fwd: FwdProcess::new(var_can_post, universe),
        }
    }
}

impl Process for ReachableProcess {
    fn step(&mut self, scheduler: &mut Scheduler, graph: &SymbolicAsyncGraph) -> PyResult<bool> {
        if self.fwd.step(scheduler, graph)? {
            let fwd_set = self.fwd.get_reachable_set();

            // If fwd set is not the whole universe, it probably has a basin.
            if fwd_set != scheduler.get_universe() {
                if log_essential(scheduler.log_level, fwd_set.symbolic_size()) {
                    println!(
                        " > Completed forward-reachability for {} transitions. Start pruning the basin...",
                        self.variable
                    );
                }

                let basin_only = reach_bwd(
                    graph,
                    fwd_set,
                    scheduler.get_universe(),
                    scheduler.get_active_variables(),
                    scheduler.log_level,
                )?
                .minus(fwd_set);

                if should_log(scheduler.log_level) {
                    println!(
                        " > Discarded {} instances using the {} transition basin.",
                        basin_only.approx_cardinality(),
                        self.variable
                    );
                }

                if !basin_only.is_empty() {
                    scheduler.discard_vertices(&basin_only);
                }
            } else if log_essential(scheduler.log_level, fwd_set.symbolic_size()) {
                println!(
                    " > Completed forward-reachability for {} transitions. Basin is empty.",
                    self.variable
                );
            }

            scheduler.spawn(ExtendedComponentProcess::new(
                self.variable,
                fwd_set.clone(),
                scheduler.get_universe().clone(),
                graph,
            ));
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn weight(&self) -> usize {
        self.fwd.weight()
    }

    fn discard_states(&mut self, set: &GraphColoredVertices) {
        self.fwd.discard_states(set)
    }
}
