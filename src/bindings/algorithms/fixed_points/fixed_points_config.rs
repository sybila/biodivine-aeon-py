use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use pyo3::pyclass;

#[pyclass(module = "biodivine_aeon", frozen)]
pub struct FixedPointsConfig {
    pub graph: SymbolicAsyncGraph,
    pub restriction: GraphColoredVertices,
    // TODO: ohtenkay - discuss the rest of the parameters.
}
