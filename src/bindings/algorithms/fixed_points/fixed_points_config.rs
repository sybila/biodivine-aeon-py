use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use pyo3::pyclass;

#[pyclass(module = "biodivine_aeon", frozen)]
pub struct FixedPointsConfig {
    pub graph: SymbolicAsyncGraph,
    pub restriction: GraphColoredVertices,
    // TODO: ohtenkay - discuss the rest of the parameters.
}

impl FixedPointsConfig {
    /// Create a new "default" [FixedPointsCongfig] for the given [SymbolicAsyncGraph].
    pub fn with_graph(graph: SymbolicAsyncGraph) -> Self {
        FixedPointsConfig {
            restriction: graph.unit_colored_vertices().clone(),
            graph,
        }
    }

    /// Update the `restriction` property
    pub fn with_restriction(mut self, restriction: GraphColoredVertices) -> Self {
        self.restriction = restriction;
        self
    }
}
