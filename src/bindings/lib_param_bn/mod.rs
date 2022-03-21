use biodivine_lib_param_bn::{ParameterId, RegulatoryGraph, VariableId};
use pyo3::prelude::*;

mod _impl_variable_id;
mod _impl_parameter_id;
mod _impl_regulatory_graph;

pub(crate) fn register(module: &PyModule) -> PyResult<()> {
    module.add_class::<PyVariableId>()?;
    module.add_class::<PyParameterId>()?;
    module.add_class::<PyRegulatoryGraph>()?;
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