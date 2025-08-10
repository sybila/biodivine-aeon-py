use biodivine_lib_param_bn::{
    symbolic_async_graph::SymbolicAsyncGraph,
    trap_spaces::{NetworkColoredSpaces, SymbolicSpaceContext},
    BooleanNetwork,
};
use macros::Config;

use crate::internal::algorithms::{cancellation::CancellationHandler, configurable::Config};

use super::TrapSpacesError;

/// A configuration struct for the [TrapSpaces] algorithms.
#[derive(Clone, Config)]
pub struct TrapSpacesConfig {
    /// The symbolic graph that will be used to compute the trap spaces.
    pub graph: SymbolicAsyncGraph,

    /// The symbolic space context that will be used to compute the trap spaces.
    pub ctx: SymbolicSpaceContext,

    /// Restricts result to the given set of spaces.
    ///
    /// Default: `ctx.mk_unit_colored_spaces(&graph)`.
    pub restriction: NetworkColoredSpaces,

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

// TODO: the current API does not allow creation straight from SymbolicAsyncGraph, this is a
// temporary workaround
impl From<(SymbolicAsyncGraph, SymbolicSpaceContext)> for TrapSpacesConfig {
    /// Create a new "default" [TrapSpacesConfig] from the given [SymbolicAsyncGraph] and
    /// [SymbolicSpaceContext].
    fn from((graph, ctx): (SymbolicAsyncGraph, SymbolicSpaceContext)) -> Self {
        assert_eq!(
            graph.symbolic_context().bdd_variable_set().variable_names(),
            ctx.bdd_variable_set().variable_names()
        );
        TrapSpacesConfig {
            restriction: ctx.mk_unit_colored_spaces(&graph),
            cancellation: Default::default(),
            bdd_size_limit: usize::MAX,
            graph,
            ctx,
        }
    }
}

impl TryFrom<&BooleanNetwork> for TrapSpacesConfig {
    type Error = TrapSpacesError;

    fn try_from(bn: &BooleanNetwork) -> Result<Self, Self::Error> {
        let ctx = SymbolicSpaceContext::new(bn);
        let graph = SymbolicAsyncGraph::with_space_context(bn, &ctx)
            .map_err(TrapSpacesError::CreationFailed)?;

        Ok(Self::from((graph, ctx)))
    }
}

impl TrapSpacesConfig {
    /// Update the `restriction` property
    pub fn with_restriction(mut self, restriction: NetworkColoredSpaces) -> Self {
        self.restriction = restriction;
        self
    }

    /// Update the `bdd_size_limit` property.
    pub fn with_bdd_size_limit(mut self, bdd_size_limit: usize) -> Self {
        self.bdd_size_limit = bdd_size_limit;
        self
    }
}
