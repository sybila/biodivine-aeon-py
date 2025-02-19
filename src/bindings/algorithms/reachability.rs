use log::{debug, info, trace};
use pyo3::{pyclass, pymethods, Py, PyResult, Python};

use crate::{
    bindings::{
        algorithms::{reachability_error::ReachabilityError, ReachabilityConfig},
        lib_param_bn::symbolic::set_colored_vertex::ColoredVertexSet,
    },
    throw_runtime_error,
};

pub const TARGET_FORWARD_SUPERSET: &str = "Reachability::forward_closed_superset";
// pub const TARGET_FORWARD_SUBSET: &str = "Reachability::forward_closed_subset";
pub const TARGET_BACKWARD_SUPERSET: &str = "Reachability::backward_closed_superset";
// pub const TARGET_BACKWARD_SUBSET: &str = "Reachability::backward_closed_subset";

/// Implements symbolic reachability operations over a `SymbolicAsyncGraph`. This means the
/// computation of both largets and smallest forward- or backward-closed sets of states.
///
/// Aside from the initial set of states, each algorithm can take a `subgraph` argument
/// which restricts the set of relevant vertices, as well as a BDD size limit and steps limit.
/// See [ReachabilityConfig] and [ReachabilityError] for more info.
#[pyclass(module = "biodivine_aeon", frozen)]
pub struct Reachability {
    config: Py<ReachabilityConfig>,
}

#[pymethods]
impl Reachability {
    #[new]
    /// Create a new [Reachability] instance with the given [ReachabilityConfig].
    pub fn new(config: Py<ReachabilityConfig>) -> Self {
        Reachability { config }
    }

    // /// Create a new [Reachability] instance with the given [AsynchronousGraph]
    // /// and otherwise default configuration.
    // pub fn with_graph(graph: Py<AsynchronousGraph>) -> Self {
    //     Reachability(ReachabilityConfig::new(graph))
    // }

    /// Create a new [Reachability] instance with the given [ReachabilityConfig].
    // #[staticmethod]
    // pub fn with_config(config: Py<ReachabilityConfig>) -> Self {
    //     Reachability(config.get().clone())
    // }
    //
    /// Retrieve the internal [ReachabilityConfig] of this instance.
    // pub fn config(&self) -> &ReachabilityConfig {
    //     &self.0
    // }
    //
    /// Compute the *greatest superset* of the given `initial` set that is forward closed.
    ///
    /// Intuitively, these are all the vertices that are reachable from the `initial` set.
    pub fn forward_closed_superset(
        &self,
        py: Python,
        initial: &ColoredVertexSet,
    ) -> PyResult<ColoredVertexSet> {
        info!(target: TARGET_FORWARD_SUPERSET, "Started with {} initial states.", initial.cardinality());

        let mut result = initial.clone();

        let variables = self.config.get().sorted_variables();
        let graph = &self.config.get().graph.get();
        let subgraph = &self.config.get().subgraph;

        if let Some(subgraph) = subgraph {
            if !initial.is_subset(subgraph) {
                info!(target: TARGET_FORWARD_SUPERSET, "Initial set is not a subset of the subgraph.");
                return throw_runtime_error(format!("{:?}", ReachabilityError::InvalidSubgraph));
            }
        }

        let mut steps = 0usize;

        'reach: loop {
            for var in variables.iter().rev() {
                // TODO: ohtenkay
                // result = is_cancelled!(self.config(), result)?;

                let mut successors = graph.var_post_out_resolved(*var, &result);
                if let Some(subgraph) = self.config.get().subgraph.as_ref() {
                    successors = successors.intersect(subgraph)
                }

                // TODO: ohtenkay - here was approx_cardinality()
                trace!(target: TARGET_FORWARD_SUPERSET, "Found {} successors for {:?}", successors.cardinality(), var);

                if !successors.is_empty() {
                    result = result.union(&successors);
                    steps += 1;

                    // TODO: here was approx_cardinality()
                    debug!(target: TARGET_FORWARD_SUPERSET, "Expanded result to {}[bdd_nodes:{}].", result.cardinality(), result.symbolic_size());

                    if result.to_bdd(py).node_count() > self.config.get().bdd_size_limit {
                        info!(target: TARGET_FORWARD_SUPERSET, "Exceeded BDD size limit.");
                        return throw_runtime_error(format!(
                            "{:?}",
                            ReachabilityError::BddSizeLimitExceeded(result)
                        ));
                    }

                    if steps > self.config.get().steps_limit {
                        info!(target: TARGET_FORWARD_SUPERSET, "Exceeded step limit.");
                        return throw_runtime_error(format!(
                            "{:?}",
                            ReachabilityError::StepsLimitExceeded(result)
                        ));
                    }

                    // Restart the loop.
                    continue 'reach;
                }
            }

            info!(target: TARGET_FORWARD_SUPERSET, "Done. Result: {} states.", result.cardinality());
            return Ok(result);
        }
    }

    /// Compute the *greatest superset* of the given `initial` set that is backward closed.
    ///
    /// Intuitively, these are all the vertices that can reach a vertex in the `initial` set.
    pub fn backward_closed_superset(
        &self,
        py: Python,
        initial: &ColoredVertexSet,
    ) -> PyResult<ColoredVertexSet> {
        info!(target: TARGET_BACKWARD_SUPERSET, "Started with {} initial states.", initial.cardinality());

        let mut result = initial.clone();

        let variables = self.config.get().sorted_variables();
        let graph = &self.config.get().graph.get();
        let subgraph = &self.config.get().subgraph;

        if let Some(subgraph) = subgraph {
            if !initial.is_subset(subgraph) {
                info!(target: TARGET_BACKWARD_SUPERSET, "Initial set is not a subset of the subgraph.");
                return throw_runtime_error(format!("{:?}", ReachabilityError::InvalidSubgraph));
            }
        }

        let mut steps = 0usize;

        'reach: loop {
            for var in variables.iter().rev() {
                // TODO: ohtenkay
                // result = is_cancelled!(self.config(), result)?;

                let mut predecessors = graph.var_pre_out_resolved(*var, &result);
                if let Some(subgraph) = self.config.get().subgraph.as_ref() {
                    predecessors = predecessors.intersect(subgraph)
                }

                // TODO: ohtenkay - here was approx_cardinality()
                trace!(target: TARGET_BACKWARD_SUPERSET, "Found {} predecessors for {:?}", predecessors.cardinality(), var);

                if !predecessors.is_empty() {
                    result = result.union(&predecessors);
                    steps += 1;

                    // TODO: ohtenkay - here was approx_cardinality()
                    debug!(target: TARGET_BACKWARD_SUPERSET, "Expanded result to {}[bdd_nodes:{}].", result.cardinality(), result.symbolic_size());

                    if result.to_bdd(py).node_count() > self.config.get().bdd_size_limit {
                        info!(target: TARGET_BACKWARD_SUPERSET, "Exceeded BDD size limit.");
                        return throw_runtime_error(format!(
                            "{:?}",
                            ReachabilityError::BddSizeLimitExceeded(result)
                        ));
                    }

                    if steps > self.config.get().steps_limit {
                        info!(target: TARGET_BACKWARD_SUPERSET, "Exceeded step limit.");
                        return throw_runtime_error(format!(
                            "{:?}",
                            ReachabilityError::StepsLimitExceeded(result)
                        ));
                    }

                    // Restart the loop.
                    continue 'reach;
                }
            }

            info!(target: TARGET_BACKWARD_SUPERSET, "Done. Result: {} states.", result.cardinality());
            return Ok(result);
        }
    }
}
