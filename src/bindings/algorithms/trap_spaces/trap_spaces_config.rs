use biodivine_lib_param_bn::{
    symbolic_async_graph::SymbolicAsyncGraph,
    trap_spaces::{NetworkColoredSpaces, SymbolicSpaceContext},
    BooleanNetwork,
};
use pyo3::pyclass;

use crate::bindings::algorithms::cancellation::CancellationHandler;

/// A configuration struct for the [TrapSpaces] algorithms.
#[pyclass(module = "biodivine_aeon", frozen)]
#[derive(Clone)]
pub struct TrapSpacesConfig {
    pub graph: SymbolicAsyncGraph,
    pub ctx: SymbolicSpaceContext,
    pub restriction: NetworkColoredSpaces,

    /// A `CancellationHandler` that can be used to stop the algorithm externally.
    ///
    /// Default: [CancelTokenNever].
    pub cancellation: Box<dyn CancellationHandler>,
}

impl TrapSpacesConfig {
    // TODO: discuss - add this for everything?
    pub fn from_boolean_network(bn: BooleanNetwork) -> Result<Self, String> {
        let graph = SymbolicAsyncGraph::new(&bn)?;
        let ctx = SymbolicSpaceContext::new(&bn);

        Ok(TrapSpacesConfig {
            restriction: ctx.mk_unit_colored_spaces(&graph),
            cancellation: Default::default(),
            graph,
            ctx,
        })
    }

    // TODO: discuss - is it possible to create this config using only the graph? mbe some default
    // values, possibly doing impl From<SymbolicContext> for SymbolicSpaceContext?
    // /// Create a new "default" [TrapSpacesConfig] for the given [SymbolicAsyncGraph].
    // pub fn with_graph(graph: SymbolicAsyncGraph) -> Self {
    //     TrapSpacesConfig {
    //         ctx: graph.symbolic_space_context(),
    //         restriction: graph.unit_colored_vertices().clone(),
    //         cancellation: Default::default(),
    //         graph,
    //     }
    // }

    /// Update the `ctx` property
    pub fn with_ctx(mut self, ctx: SymbolicSpaceContext) -> Self {
        self.ctx = ctx;
        self
    }

    /// Update the `restriction` property
    pub fn with_restriction(mut self, restriction: NetworkColoredSpaces) -> Self {
        self.restriction = restriction;
        self
    }

    /// Update the `cancellation` property, automatically wrapping the [CancellationHandler]
    /// in a `Box`.
    pub fn with_cancellation<C: CancellationHandler + 'static>(mut self, cancellation: C) -> Self {
        self.cancellation = Box::new(cancellation);
        self
    }
}
