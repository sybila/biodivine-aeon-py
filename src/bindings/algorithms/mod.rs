use std::collections::HashSet;

use biodivine_lib_param_bn::{
    symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph},
    VariableId,
};
use pyo3::{types::PyModule, Bound, PyResult};

/// A configuration struct for the [Reachability] algorithms.
// #[derive(Clone)]
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

pub fn register(_module: &Bound<'_, PyModule>) -> PyResult<()> {
    Ok(())
}
