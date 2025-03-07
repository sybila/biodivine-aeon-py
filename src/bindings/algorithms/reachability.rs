use biodivine_lib_param_bn::{
    biodivine_std::traits::Set,
    symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph},
};
use log::{debug, info, trace};
use pyo3::pyclass;

use crate::{
    bindings::algorithms::{reachability_error::ReachabilityError, ReachabilityConfig},
    is_cancelled,
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
#[pyclass(module = "biodivine_aeon", frozen)]
pub struct Reachability {
    config: ReachabilityConfig,
}

impl Reachability {
    /// Create a new [Reachability] instance with the given [SymbolicAsyncGraph]
    /// and otherwise default configuration.
    pub fn with_graph(graph: SymbolicAsyncGraph) -> Self {
        Reachability {
            config: ReachabilityConfig::with_graph(graph),
        }
    }

    /// Create a new [Reachability] instance with the given [ReachabilityConfig].
    pub fn with_config(config: ReachabilityConfig) -> Self {
        Reachability { config }
    }

    /// Retrieve the internal [ReachabilityConfig] of this instance.
    pub fn config(&self) -> &ReachabilityConfig {
        &self.config
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

    /// Compute the *greatest superset* of the given `initial` set that is backward closed.
    ///
    /// Intuitively, these are all the vertices that can reach a vertex in the `initial` set.
    pub fn backward_closed_superset(
        &self,
        initial: &GraphColoredVertices,
    ) -> Result<GraphColoredVertices, ReachabilityError> {
        info!(target: TARGET_BACKWARD_SUPERSET, "Started with {} initial states.", initial.exact_cardinality());

        let mut result = initial.clone();

        let variables = self.config().sorted_variables();
        let graph = &self.config().graph;
        let subgraph = &self.config().subgraph;

        if let Some(subgraph) = subgraph {
            if !initial.is_subset(subgraph) {
                info!(target: TARGET_BACKWARD_SUPERSET, "Initial set is not a subset of the subgraph.");
                return Err(ReachabilityError::InvalidSubgraph);
            }
        }

        let mut steps = 0usize;

        'reach: loop {
            for var in variables.iter().rev() {
                result = is_cancelled!(self.config(), result)?;

                let mut predecessors = graph.var_pre_out(*var, &result);
                if let Some(subgraph) = self.config().subgraph.as_ref() {
                    predecessors = predecessors.intersect(subgraph)
                }

                trace!(target: TARGET_BACKWARD_SUPERSET, "Found {} predecessors for {:?}", predecessors.approx_cardinality(), var);

                if !predecessors.is_empty() {
                    result = result.union(&predecessors);
                    steps += 1;

                    debug!(target: TARGET_BACKWARD_SUPERSET, "Expanded result to {}[bdd_nodes:{}].", result.approx_cardinality(), result.symbolic_size());

                    if result.as_bdd().size() > self.config().bdd_size_limit {
                        info!(target: TARGET_BACKWARD_SUPERSET, "Exceeded BDD size limit.");
                        return Err(ReachabilityError::BddSizeLimitExceeded(result));
                    }

                    if steps > self.config().steps_limit {
                        info!(target: TARGET_BACKWARD_SUPERSET, "Exceeded step limit.");
                        return Err(ReachabilityError::StepsLimitExceeded(result));
                    }

                    // Restart the loop.
                    continue 'reach;
                }
            }

            info!(target: TARGET_BACKWARD_SUPERSET, "Done. Result: {} states.", result.exact_cardinality());
            return Ok(result);
        }
    }

    /// Compute the *greatest subset* of the given `initial` set that is forward closed.
    ///
    /// Intuitively, this removes all vertices that can reach a vertex outside the `initial`
    /// set.
    pub fn forward_closed_subset(
        &self,
        initial: &GraphColoredVertices,
    ) -> Result<GraphColoredVertices, ReachabilityError> {
        info!(target: TARGET_FORWARD_SUBSET, "Started with {} initial states.", initial.exact_cardinality());

        let mut result = initial.clone();

        let variables = self.config().sorted_variables();
        let graph = &self.config().graph;
        let subgraph = &self.config().subgraph;

        if let Some(subgraph) = subgraph {
            if !initial.is_subset(subgraph) {
                info!(target: TARGET_FORWARD_SUBSET, "Initial set is not a subset of the subgraph.");
                return Err(ReachabilityError::InvalidSubgraph);
            }
        }

        let mut steps = 0usize;

        'reach: loop {
            for var in variables.iter().rev() {
                result = is_cancelled!(self.config(), result)?;

                let mut can_go_out = graph.var_can_post_out(*var, &result);
                if let Some(subgraph) = self.config().subgraph.as_ref() {
                    // The vertex can only escape if the successor is also
                    // in the prescribed subgraph.
                    // TODO: Cache this set.
                    let does_not_count = graph.var_can_post_out(*var, subgraph);
                    can_go_out = can_go_out.minus(&does_not_count);
                }

                trace!(target: TARGET_FORWARD_SUBSET, "Found {} leaving vertices for {:?}", can_go_out.approx_cardinality(), var);

                if !can_go_out.is_empty() {
                    result = result.minus(&can_go_out);
                    steps += 1;

                    debug!(target: TARGET_FORWARD_SUBSET, "Reduced result to {}[bdd_nodes:{}].", result.approx_cardinality(), result.symbolic_size());

                    if result.as_bdd().size() > self.config().bdd_size_limit {
                        info!(target: TARGET_FORWARD_SUBSET, "Exceeded BDD size limit.");
                        return Err(ReachabilityError::BddSizeLimitExceeded(result));
                    }

                    if steps > self.config().steps_limit {
                        info!(target: TARGET_FORWARD_SUBSET, "Exceeded step limit.");
                        return Err(ReachabilityError::StepsLimitExceeded(result));
                    }

                    // Restart the loop.
                    continue 'reach;
                }
            }

            info!(target: TARGET_FORWARD_SUBSET, "Done. Result: {} states.", result.exact_cardinality());
            return Ok(result);
        }
    }

    /// Compute the *greatest subset* of the given `initial` set that is backward closed.
    ///
    /// Intuitively, this removes all vertices that can be reached by a vertex outside
    /// the `initial` set.
    pub fn backward_closed_subset(
        &self,
        initial: &GraphColoredVertices,
    ) -> Result<GraphColoredVertices, ReachabilityError> {
        info!(target: TARGET_BACKWARD_SUBSET, "Started with {} initial states.", initial.exact_cardinality());

        let mut result = initial.clone();

        let variables = self.config().sorted_variables();
        let graph = &self.config().graph;
        let subgraph = &self.config().subgraph;

        if let Some(subgraph) = subgraph {
            if !initial.is_subset(subgraph) {
                info!(target: TARGET_BACKWARD_SUBSET, "Initial set is not a subset of the subgraph.");
                return Err(ReachabilityError::InvalidSubgraph);
            }
        }

        let mut steps = 0usize;

        'reach: loop {
            for var in variables.iter().rev() {
                result = is_cancelled!(self.config(), result)?;

                let mut has_predecessor_outside = graph.var_can_pre_out(*var, &result);
                if let Some(subgraph) = self.config().subgraph.as_ref() {
                    // The predecessor is only relevant if it exists within the subgraph.
                    // TODO: Cache this set.
                    let does_not_count = graph.var_can_pre_out(*var, subgraph);
                    has_predecessor_outside = has_predecessor_outside.minus(&does_not_count);
                }

                trace!(target: TARGET_BACKWARD_SUBSET, "Found {} leaving vertices for {:?}", has_predecessor_outside.approx_cardinality(), var);

                if !has_predecessor_outside.is_empty() {
                    result = result.minus(&has_predecessor_outside);
                    steps += 1;

                    debug!(target: TARGET_BACKWARD_SUBSET, "Reduced result to {}[bdd_nodes:{}].", result.approx_cardinality(), result.symbolic_size());

                    if result.as_bdd().size() > self.config().bdd_size_limit {
                        info!(target: TARGET_BACKWARD_SUBSET, "Exceeded BDD size limit.");
                        return Err(ReachabilityError::BddSizeLimitExceeded(result));
                    }

                    if steps > self.config().steps_limit {
                        info!(target: TARGET_BACKWARD_SUBSET, "Exceeded step limit.");
                        return Err(ReachabilityError::StepsLimitExceeded(result));
                    }

                    // Restart the loop.
                    continue 'reach;
                }
            }

            info!(target: TARGET_BACKWARD_SUBSET, "Done. Result: {} states.", result.exact_cardinality());
            return Ok(result);
        }
    }
}

// #[pymethods]
// impl Reachability {
//     /// Create a new [Reachability] instance with the given [AsynchronousGraph]
//     /// and otherwise default configuration.
//     #[staticmethod]
//     pub fn with_graph_py(py: Python, graph: Py<AsynchronousGraph>) -> Self {
//         Reachability {
//             config: Py::new(py, ReachabilityConfig::with_graph(graph)).unwrap(),
//             config_rs: None,
//         }
//     }
//
//     /// Create a new [Reachability] instance with the given [ReachabilityConfig].
//     #[staticmethod]
//     pub fn with_config_py(config: Py<ReachabilityConfig>) -> Self {
//         Reachability {
//             config,
//             config_rs: None,
//         }
//     }
//
//     /// Compute the *greatest superset* of the given `initial` set that is forward closed.
//     ///
//     /// Intuitively, these are all the vertices that are reachable from the `initial` set.
//     pub fn forward_closed_superset_py(
//         &self,
//         py: Python,
//         initial: &ColoredVertexSet,
//     ) -> PyResult<ColoredVertexSet> {
//         info!(target: TARGET_FORWARD_SUPERSET, "Started with {} initial states.", initial.cardinality());
//
//         let mut result = initial.clone();
//
//         let variables = self.config().sorted_variables();
//         let graph = &self.config().graph.get();
//         let subgraph = &self.config().subgraph;
//
//         if let Some(subgraph) = subgraph {
//             if !initial.is_subset(subgraph) {
//                 info!(target: TARGET_FORWARD_SUPERSET, "Initial set is not a subset of the subgraph.");
//                 return Err(ReachabilityError::InvalidSubgraph.into());
//             }
//         }
//
//         let mut steps = 0usize;
//
//         'reach: loop {
//             for var in variables.iter().rev() {
//                 result = is_cancelled!(self.config(), result)?;
//
//                 let mut successors = graph.var_post_out_resolved(*var, &result);
//                 if let Some(subgraph) = self.config().subgraph.as_ref() {
//                     successors = successors.intersect(subgraph)
//                 }
//
//                 // TODO: ohtenkay - here was approx_cardinality()
//                 trace!(target: TARGET_FORWARD_SUPERSET, "Found {} successors for {:?}", successors.cardinality(), var);
//
//                 if !successors.is_empty() {
//                     result = result.union(&successors);
//                     steps += 1;
//
//                     // TODO: ohtenkay - here was approx_cardinality()
//                     debug!(target: TARGET_FORWARD_SUPERSET, "Expanded result to {}[bdd_nodes:{}].", result.cardinality(), result.symbolic_size());
//
//                     if result.to_bdd(py).node_count() > self.config().bdd_size_limit {
//                         info!(target: TARGET_FORWARD_SUPERSET, "Exceeded BDD size limit.");
//                         return Err(ReachabilityError::BddSizeLimitExceeded(result).into());
//                     }
//
//                     if steps > self.config().steps_limit {
//                         info!(target: TARGET_FORWARD_SUPERSET, "Exceeded step limit.");
//                         return Err(ReachabilityError::StepsLimitExceeded(result).into());
//                     }
//
//                     // Restart the loop.
//                     continue 'reach;
//                 }
//             }
//
//             info!(target: TARGET_FORWARD_SUPERSET, "Done. Result: {} states.", result.cardinality());
//             return Ok(result);
//         }
//     }
//
//     /// Compute the *greatest superset* of the given `initial` set that is backward closed.
//     ///
//     /// Intuitively, these are all the vertices that can reach a vertex in the `initial` set.
//     pub fn backward_closed_superset_py(
//         &self,
//         py: Python,
//         initial: &ColoredVertexSet,
//     ) -> PyResult<ColoredVertexSet> {
//         info!(target: TARGET_BACKWARD_SUPERSET, "Started with {} initial states.", initial.cardinality());
//
//         let mut result = initial.clone();
//
//         let variables = self.config().sorted_variables();
//         let graph = &self.config().graph.get();
//         let subgraph = &self.config().subgraph;
//
//         if let Some(subgraph) = subgraph {
//             if !initial.is_subset(subgraph) {
//                 info!(target: TARGET_BACKWARD_SUPERSET, "Initial set is not a subset of the subgraph.");
//                 return Err(ReachabilityError::InvalidSubgraph.into());
//             }
//         }
//
//         let mut steps = 0usize;
//
//         'reach: loop {
//             for var in variables.iter().rev() {
//                 result = is_cancelled!(self.config(), result)?;
//
//                 let mut predecessors = graph.var_pre_out_resolved(*var, &result);
//                 if let Some(subgraph) = self.config().subgraph.as_ref() {
//                     predecessors = predecessors.intersect(subgraph)
//                 }
//
//                 // TODO: ohtenkay - here was approx_cardinality()
//                 trace!(target: TARGET_BACKWARD_SUPERSET, "Found {} predecessors for {:?}", predecessors.cardinality(), var);
//
//                 if !predecessors.is_empty() {
//                     result = result.union(&predecessors);
//                     steps += 1;
//
//                     // TODO: ohtenkay - here was approx_cardinality()
//                     debug!(target: TARGET_BACKWARD_SUPERSET, "Expanded result to {}[bdd_nodes:{}].", result.cardinality(), result.symbolic_size());
//
//                     if result.to_bdd(py).node_count() > self.config().bdd_size_limit {
//                         info!(target: TARGET_BACKWARD_SUPERSET, "Exceeded BDD size limit.");
//                         return Err(ReachabilityError::BddSizeLimitExceeded(result).into());
//                     }
//
//                     if steps > self.config().steps_limit {
//                         info!(target: TARGET_BACKWARD_SUPERSET, "Exceeded step limit.");
//                         return Err(ReachabilityError::StepsLimitExceeded(result).into());
//                     }
//
//                     // Restart the loop.
//                     continue 'reach;
//                 }
//             }
//
//             info!(target: TARGET_BACKWARD_SUPERSET, "Done. Result: {} states.", result.cardinality());
//             return Ok(result);
//         }
//     }
// }
