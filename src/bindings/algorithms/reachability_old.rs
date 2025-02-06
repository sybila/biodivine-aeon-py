use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use log::{debug, info, trace};

use crate::bindings::algorithms::{
    reachability_config::ReachabilityConfig, reachability_error::ReachabilityError,
};

pub const TARGET_FORWARD_SUPERSET: &str = "Reachability::forward_closed_superset";
pub const TARGET_FORWARD_SUBSET: &str = "Reachability::forward_closed_subset";
pub const TARGET_BACKWARD_SUPERSET: &str = "Reachability::backward_closed_superset";
pub const TARGET_BACKWARD_SUBSET: &str = "Reachability::backward_closed_subset";

/// Implements symbolic reachability operations over a `SymbolicAsyncGraph`. This means the
/// computation of both largets and smallest forward- or backward-closed sets of states.
///
/// Aside from the initial set of states, each algorithm can take a `subgraph` argument
/// which restricts the set of relevant vertices, as well as a BDD size limit and steps limit.
/// See [ReachabilityConfig] and [ReachabilityError] for more info.
pub struct Reachability(ReachabilityConfig);

impl Reachability {
    /// Create a new [Reachability] instance with the given [SymbolicAsyncGraph]
    /// and otherwise default configuration.
    pub fn with_graph(graph: SymbolicAsyncGraph) -> Self {
        Reachability(ReachabilityConfig::with_graph(graph))
    }

    /// Create a new [Reachability] instance with the given [ReachabilityConfig].
    pub fn with_config(config: ReachabilityConfig) -> Self {
        Reachability(config)
    }

    /// Retrieve the internal [ReachabilityConfig] of this instance.
    pub fn config(&self) -> &ReachabilityConfig {
        &self.0
    }

    /// Compute the *greatest superset* of the given `initial` set that is forward closed.
    ///
    /// Intuitively, these are all the vertices that are reachable from the `initial` set.
    pub fn forward_closed_superset(
        &self,
        initial: &GraphColoredVertices,
    ) -> Result<GraphColoredVertices, ReachabilityError> {
        info!(target: TARGET_FORWARD_SUPERSET, "Started with {} initial states.", initial.exact_cardinality());

        let mut result = initial.clone();

        let variables = self.config().sorted_variables();
        let graph = &self.config().graph;
        let subgraph = &self.config().subgraph;

        if let Some(subgraph) = subgraph {
            if !initial.is_subset(subgraph) {
                info!(target: TARGET_FORWARD_SUPERSET, "Initial set is not a subset of the subgraph.");
                return Err(ReachabilityError::InvalidSubgraph);
            }
        }

        let mut steps = 0usize;

        'reach: loop {
            for var in variables.iter().rev() {
                result = is_cancelled!(self.config(), result)?;

                let mut successors = graph.var_post_out(*var, &result);
                if let Some(subgraph) = self.config().subgraph.as_ref() {
                    successors = successors.intersect(subgraph)
                }

                trace!(target: TARGET_FORWARD_SUPERSET, "Found {} successors for {:?}", successors.approx_cardinality(), var);

                if !successors.is_empty() {
                    result = result.union(&successors);
                    steps += 1;

                    debug!(target: TARGET_FORWARD_SUPERSET, "Expanded result to {}[bdd_nodes:{}].", result.approx_cardinality(), result.symbolic_size());

                    if result.as_bdd().size() > self.config().bdd_size_limit {
                        info!(target: TARGET_FORWARD_SUPERSET, "Exceeded BDD size limit.");
                        return Err(ReachabilityError::BddSizeLimitExceeded(result));
                    }

                    if steps > self.config().steps_limit {
                        info!(target: TARGET_FORWARD_SUPERSET, "Exceeded step limit.");
                        return Err(ReachabilityError::StepsLimitExceeded(result));
                    }

                    // Restart the loop.
                    continue 'reach;
                }
            }

            info!(target: TARGET_FORWARD_SUPERSET, "Done. Result: {} states.", result.exact_cardinality());
            return Ok(result);
        }
    }
}
