use std::collections::HashSet;

use pyo3::{pyclass, pymethods, Py};

use crate::bindings::lib_param_bn::{
    symbolic::{asynchronous_graph::AsynchronousGraph, set_colored_vertex::ColoredVertexSet},
    variable_id::VariableId,
};

/// A configuration struct for the [Reachability] algorithms.
#[pyclass(module = "biodivine_aeon", frozen)]
pub struct ReachabilityConfig {
    /// The symbolic graph that will be used to compute the successors and predecessors of
    /// individual states.
    pub graph: Py<AsynchronousGraph>,

    /// Restricts the reachability operation to the given set of vertices. This also includes
    /// edges! For example, if a vertex `x` only has outgoing edges into vertices outside the
    /// `subgraph`, it would be considered a fixed-point.
    ///
    /// The initial set must be a subset of the subgraph vertices.
    ///
    /// Default: `None`.
    pub subgraph: Option<ColoredVertexSet>,

    /// Specifies the set of variables that can be updated by the reachability process.
    /// Remaining variables stay constant, because they are never updated.
    ///
    /// This can be used to implement "reachability within a subspace" that is faster than
    /// providing a `subgraph`, since the variables that are constant in the subspace never
    /// need to be updated. Alternatively, this can be used for various "multi-stage"
    /// schemes, for example to start with only a small component of the whole network and
    /// then gradually expand to the whole variable set.
    ///
    /// Default: `graph.network_variables()`.
    pub variables: HashSet<VariableId>,

    // /// A `CancellationHandler` that can be used to stop the algorithm externally.
    // ///
    // /// Default: [CancelTokenNever].
    // pub cancellation: Box<dyn CancellationHandler>,
    /// The maximum BDD size of the reachable set.
    ///
    /// Note that the algorithm can use other auxiliary BDDs that do not
    /// count towards this limit.
    ///
    /// Default: `usize::MAX`.
    pub bdd_size_limit: usize,

    /// The maximum number of steps that the algorithm can take before terminating.
    ///
    /// A step is a single extension or reduction of the reachable set of vertices.
    ///
    /// Default: `usize::MAX`.
    pub steps_limit: usize,
}

#[pymethods]
impl ReachabilityConfig {
    #[new]
    pub fn new(graph: Py<AsynchronousGraph>) -> Self {
        ReachabilityConfig {
            subgraph: None,
            variables: graph.get().network_variables().into_iter().collect(),
            bdd_size_limit: usize::MAX,
            steps_limit: usize::MAX,
            graph,
        }
    }

    // TODO: ohtenkay - remove, only here for testing
    pub fn get_graph(&self) -> &Py<AsynchronousGraph> {
        &self.graph
    }

    // TODO: ohtenkay - remove, only here for testing
    pub fn get_variables(&self) -> HashSet<VariableId> {
        self.variables.clone()
    }
}
