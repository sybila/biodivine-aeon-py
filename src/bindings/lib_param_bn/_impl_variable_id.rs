use crate::bindings::lib_param_bn::PyVariableId;
use biodivine_lib_param_bn::VariableId;
use pyo3::prelude::*;

impl From<PyVariableId> for VariableId {
    fn from(value: PyVariableId) -> Self {
        value.0
    }
}

impl From<VariableId> for PyVariableId {
    fn from(value: VariableId) -> Self {
        PyVariableId(value)
    }
}

#[pymethods]
impl PyVariableId {
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("BnVariableId({})", self.0))
    }

    fn __repr__(&self) -> PyResult<String> {
        self.__str__()
    }
}
