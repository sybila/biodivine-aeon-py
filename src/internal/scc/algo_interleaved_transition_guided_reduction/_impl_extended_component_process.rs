use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use biodivine_lib_param_bn::VariableId;
use pyo3::PyResult;

use crate::internal::scc::algo_interleaved_transition_guided_reduction::{
    BwdProcess, ExtendedComponentProcess, Process, Scheduler,
};
use crate::internal::scc::algo_saturated_reachability::reach_bwd;
use crate::{log_essential, should_log};

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
    fn step(&mut self, scheduler: &mut Scheduler, graph: &SymbolicAsyncGraph) -> PyResult<bool> {
        if self.bwd.step(scheduler, graph)? {
            let extended_component = self.bwd.get_reachable_set();
            let bottom = self.fwd_set.minus(extended_component);

            if log_essential(scheduler.log_level, extended_component.symbolic_size()) {
                println!(
                    " > Completed extended component for {} transitions.",
                    self.variable
                );
            }

            // This is a modification of the original TGR to allow faster checking
            // of regions that are not trap sets but might still contain attractors.

            // Check if fwd_set is globally forward-closed. If not, we extend the bottom
            // set with everything that is "outside" of the current universe. These states
            // are technically not all reachable from fwd_set, but for our purposes, reaching
            // one such state is enough to prove there isn't an attractor.
            let mut is_forward_closed = true;
            for var in graph.variables() {
                let step = graph.var_can_post_out(var, &self.fwd_set);
                if !step.is_empty() {
                    is_forward_closed = false;
                    break;
                }
            }

            let bottom = if is_forward_closed {
                bottom
            } else {
                let complement = graph
                    .unit_colored_vertices()
                    .minus(scheduler.get_universe());
                bottom.union(&complement)
            };

            if !bottom.is_empty() {
                if log_essential(scheduler.log_level, bottom.symbolic_size()) {
                    println!(
                        " > Start pruning the basin of {} extended component.",
                        self.variable
                    );
                }
                let basin_only = reach_bwd(
                    graph,
                    &bottom,
                    scheduler.get_universe(),
                    scheduler.get_active_variables(),
                    scheduler.log_level,
                )?
                .minus(&bottom);

                if should_log(scheduler.log_level) {
                    println!(
                        " > Discarded {} instances based on the {} extended component.",
                        basin_only.approx_cardinality(),
                        self.variable
                    );
                }

                if !basin_only.is_empty() {
                    scheduler.discard_vertices(&basin_only);
                }
            }

            let var_can_post = graph.var_can_post(self.variable, scheduler.get_universe());
            if var_can_post.is_empty() {
                scheduler.discard_variable(self.variable);
            }

            Ok(true)
        } else {
            Ok(false)
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
