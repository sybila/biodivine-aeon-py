use biodivine_lib_param_bn::{
    symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph},
    BooleanNetwork,
};
use macros::Config;

use crate::bindings::algorithms::{cancellation::CancellationHandler, configurable::Config};

use super::FixedPointsError;

/// A configuration struct for the [FixedPoints] algorithms.
#[derive(Clone, Config)]
pub struct FixedPointsConfig {
    /// The symbolic graph that will be used to compute the fixed points.
    pub graph: SymbolicAsyncGraph,

    /// Restricts result to the given set of vertices.
    ///
    /// Default: `graph.unit_colored_vertices()`.
    pub restriction: GraphColoredVertices,

    /// A `CancellationHandler` that can be used to stop the algorithm externally.
    ///
    /// Default: [CancelTokenNever].
    pub cancellation: Box<dyn CancellationHandler>,

    /// The maximum size of the BDD used in the merging process.
    ///
    /// Note that the algorithm can use other auxiliary BDDs that do not
    /// count towards this limit.
    ///
    /// Default: `usize::MAX`.
    pub bdd_size_limit: usize,
}

impl From<SymbolicAsyncGraph> for FixedPointsConfig {
    /// Create a new "default" [FixedPointsConfig] from the given [SymbolicAsyncGraph].
    fn from(graph: SymbolicAsyncGraph) -> Self {
        FixedPointsConfig {
            restriction: graph.mk_unit_colored_vertices(),
            cancellation: Default::default(),
            bdd_size_limit: usize::MAX,
            graph,
        }
    }
}

impl TryFrom<&BooleanNetwork> for FixedPointsConfig {
    type Error = FixedPointsError;

    /// Create a new "default" [FixedPointsConfig] from the given [BooleanNetwork].
    fn try_from(boolean_network: &BooleanNetwork) -> Result<Self, Self::Error> {
        let graph =
            SymbolicAsyncGraph::new(boolean_network).map_err(FixedPointsError::CreationFailed)?;

        Ok(Self::from(graph))
    }
}

impl FixedPointsConfig {
    /// Update the `restriction` property
    pub fn with_restriction(mut self, restriction: GraphColoredVertices) -> Self {
        self.restriction = restriction;
        self
    }

    /// Update the `bdd_size_limit` property.
    pub fn with_bdd_size_limit(mut self, bdd_size_limit: usize) -> Self {
        self.bdd_size_limit = bdd_size_limit;
        self
    }
}
