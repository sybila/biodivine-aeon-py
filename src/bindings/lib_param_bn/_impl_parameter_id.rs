use super::PyParameterId;
use biodivine_lib_param_bn::ParameterId;
use pyo3::prelude::*;

impl From<PyParameterId> for ParameterId {
    fn from(value: PyParameterId) -> Self {
        value.0
    }
}

impl From<ParameterId> for PyParameterId {
    fn from(value: ParameterId) -> Self {
        PyParameterId(value)
    }
}

#[pymethods]
impl PyParameterId {
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("BnParameterId({:?})", self.0))
    }

    fn __repr__(&self) -> PyResult<String> {
        self.__str__()
    }
}
