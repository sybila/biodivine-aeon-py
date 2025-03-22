use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
use pyo3::pyclass;

#[pyclass(module = "biodivine_aeon", frozen)]
pub struct FixedPointsConfig {
    pub graph: SymbolicAsyncGraph,
    // TODO: ohtenkay - discuss the rest of the parameters.
    // TODO: ohtenkay - should restrictions be a part of the config?
}
