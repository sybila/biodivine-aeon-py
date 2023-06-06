use biodivine_lib_bdd::Bdd;
use biodivine_lib_param_bn::symbolic_async_graph::projected_iteration::{
    MixedProjection, MixedProjectionIterator,
};
use biodivine_lib_param_bn::symbolic_async_graph::{
    GraphColoredVertices, GraphColors, GraphVertexIterator, GraphVertices, IterableVertices,
    SymbolicAsyncGraph, SymbolicContext,
};
use biodivine_lib_param_bn::{
    BooleanNetwork, FnUpdate, ModelAnnotation, ParameterId, RegulatoryGraph, VariableId,
};
use macros::Wrapper;
use pyo3::prelude::*;

mod _impl_boolean_network;
mod _impl_fixed_points;
mod _impl_fn_update;
mod _impl_graph_colored_vertices;
mod _impl_graph_colors;
mod _impl_graph_vertices;
mod _impl_model_annotation;
mod _impl_parameter_id;
mod _impl_regulatory_graph;
mod _impl_symbolic_async_graph;
mod _impl_symbolic_context;
mod _impl_symbolic_projection;
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
    module.add_class::<PyModelAnnotation>()?;
    module.add_class::<PyGraphVertexIterator>()?;
    module.add_class::<PySymbolicContext>()?;
    module.add_class::<PyFnUpdate>()?;
    module.add_class::<PyFixedPoints>()?;
    module.add_class::<PySymbolicProjection>()?;
    Ok(())
}

#[pyclass(name = "VariableId")]
#[derive(Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd, Wrapper)]
pub struct PyVariableId(VariableId);

#[pyclass(name = "ParameterId")]
#[derive(Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd, Wrapper)]
pub struct PyParameterId(ParameterId);

#[pyclass(name = "RegulatoryGraph", subclass)]
#[derive(Clone, Wrapper)]
pub struct PyRegulatoryGraph(RegulatoryGraph);

#[pyclass(name = "BooleanNetwork", extends=PyRegulatoryGraph)]
#[derive(Clone, Wrapper)]
pub struct PyBooleanNetwork(BooleanNetwork);

#[pyclass(name = "ColorSet")]
#[derive(Clone, Wrapper)]
pub struct PyGraphColors(GraphColors);

#[pyclass(name = "VertexSet")]
#[derive(Clone, Wrapper)]
pub struct PyGraphVertices(GraphVertices);

#[pyclass(name = "ColoredVertexSet")]
#[derive(Clone, Wrapper)]
pub struct PyGraphColoredVertices(GraphColoredVertices);

#[pyclass(name = "SymbolicAsyncGraph")]
#[derive(Clone, Wrapper)]
pub struct PySymbolicAsyncGraph(SymbolicAsyncGraph);

#[pyclass(name = "ModelAnnotation")]
#[derive(Clone, Wrapper)]
pub struct PyModelAnnotation(ModelAnnotation);

#[pyclass(name = "FixedPoints")]
pub struct PyFixedPoints();

#[pyclass(name = "GraphVertexIterator")]
pub struct PyGraphVertexIterator(Box<IterableVertices>, GraphVertexIterator<'static>);

#[pyclass(name = "SymbolicContext")]
#[derive(Clone, Wrapper)]
pub struct PySymbolicContext(SymbolicContext);

#[pyclass(name = "UpdateFunction")]
#[derive(Clone, Wrapper, Eq, PartialEq, Hash)]
pub struct PyFnUpdate(FnUpdate);

#[pyclass(name = "SymbolicProjection")]
pub struct PySymbolicProjection(
    Box<SymbolicAsyncGraph>,
    Box<Bdd>,
    Box<MixedProjection<'static>>,
    MixedProjectionIterator<'static, 'static>,
);
