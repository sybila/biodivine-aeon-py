use biodivine_lib_bdd::BddVariable;
use pyo3::prelude::*;
use crate::bindings::lib_bdd::PyBddVariable;

impl From<BddVariable> for PyBddVariable {
    fn from(value: BddVariable) -> Self {
        PyBddVariable(value)
    }
}

impl From<PyBddVariable> for BddVariable {
    fn from(value: PyBddVariable) -> Self {
        value.0
    }
}

#[pymethods]
impl PyBddVariable {
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("BddVariable({})", self.0))
    }

    fn __repr__(&self) -> PyResult<String> {
        self.__str__()
    }
}