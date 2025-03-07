use biodivine_lib_param_bn::{
    symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph},
    VariableId,
};
use pyo3::{pyclass, pymethods, Py};
use std::collections::HashSet;

use crate::{
    bindings::{
        algorithms::cancellation_handler::{CancelTokenNever, CancellationHandler},
        lib_param_bn::symbolic::asynchronous_graph::AsynchronousGraph,
    },
    AsNative,
};

/// A configuration struct for the [Reachability] algorithms.
#[pyclass(module = "biodivine_aeon", frozen)]
#[derive(Clone)]
pub struct ReachabilityConfig {
    /// The symbolic graph that will be used to compute the successors and predecessors of
    /// individual states.
    pub graph: SymbolicAsyncGraph,

    /// Restricts the reachability operation to the given set of vertices. This also includes
    /// edges! For example, if a vertex `x` only has outgoing edges into vertices outside the
    /// `subgraph`, it would be considered a fixed-point.
    ///
    /// The initial set must be a subset of the subgraph vertices.
    ///
    /// Default: `None`.
    pub subgraph: Option<GraphColoredVertices>,

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

    /// A `CancellationHandler` that can be used to stop the algorithm externally.
    ///
    /// Default: [CancelTokenNever].
    pub cancellation: Box<dyn CancellationHandler>,

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

impl ReachabilityConfig {
    /// Create a new "default" [ReachabilityConfig] for the given [SymbolicAsyncGraph].
    pub fn with_graph(graph: SymbolicAsyncGraph) -> Self {
        ReachabilityConfig {
            variables: HashSet::from_iter(graph.variables()),
            subgraph: None,
            cancellation: Box::new(CancelTokenNever),
            bdd_size_limit: usize::MAX,
            steps_limit: usize::MAX,
            graph,
        }
    }

    /// Return the variables sorted in ascending order.
    pub fn sorted_variables(&self) -> Vec<VariableId> {
        let mut variables = Vec::from_iter(self.variables.clone());
        variables.sort();
        variables
    }

    /// Update the `cancellation` property, automatically wrapping the [CancellationHandler]
    /// in a `Box`.
    pub fn set_cancellation<C: CancellationHandler + 'static>(&mut self, cancellation: C) {
        self.cancellation = Box::new(cancellation);
    }
}

impl CancellationHandler for ReachabilityConfig {
    fn is_cancelled(&self) -> bool {
        self.cancellation.is_cancelled()
    }
}

// TODO: ohtenkay - make this optional with a feature flag
#[pymethods]
impl ReachabilityConfig {
    #[staticmethod]
    pub fn with_graph_py(graph: Py<AsynchronousGraph>) -> Self {
        ReachabilityConfig::with_graph(graph.get().as_native().clone())
    }
}
