use biodivine_lib_param_bn::symbolic_async_graph::{
    GraphColoredVertices, GraphColors, GraphVertices, SymbolicAsyncGraph,
};
use biodivine_lib_param_bn::{BooleanNetwork, ParameterId, RegulatoryGraph, VariableId};
use pyo3::prelude::*;

mod _impl_boolean_network;
mod _impl_graph_colored_vertices;
mod _impl_graph_colors;
mod _impl_graph_vertices;
mod _impl_parameter_id;
mod _impl_regulatory_graph;
mod _impl_symbolic_async_graph;
mod _impl_variable_id;

pub(crate) fn register(module: &PyModule) -> PyResult<()> {
    module.add_class::<PyVariableId>()?;
    module.add_class::<PyParameterId>()?;
    module.add_class::<PyRegulatoryGraph>()?;
    module.add_class::<PyBooleanNetwork>()?;
    module.add_class::<PyGraphColors>()?;
    module.add_class::<PyGraphVertices>()?;
    module.add_class::<PyGraphColoredVertices>()?;
    module.add_class::<PySymbolicAsyncGraph>()?;
    Ok(())
}

#[pyclass(name = "VariableId")]
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct PyVariableId(VariableId);

#[pyclass(name = "ParameterId")]
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct PyParameterId(ParameterId);

#[pyclass(name = "RegulatoryGraph")]
#[derive(Clone)]
pub struct PyRegulatoryGraph(RegulatoryGraph);

#[pyclass(name = "BooleanNetwork")]
#[derive(Clone)]
pub struct PyBooleanNetwork(BooleanNetwork);

#[pyclass(name = "ColorSet")]
#[derive(Clone)]
pub struct PyGraphColors(GraphColors);

#[pyclass(name = "VertexSet")]
#[derive(Clone)]
pub struct PyGraphVertices(GraphVertices);

#[pyclass(name = "ColoredVertexSet")]
#[derive(Clone)]
pub struct PyGraphColoredVertices(GraphColoredVertices);

#[pyclass(name = "SymbolicAsyncGraph")]
#[derive(Clone)]
pub struct PySymbolicAsyncGraph(SymbolicAsyncGraph);
