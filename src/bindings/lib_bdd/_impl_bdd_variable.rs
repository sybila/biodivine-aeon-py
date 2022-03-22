use crate::bindings::lib_bdd::PyBddVariable;
use crate::throw_runtime_error;
use biodivine_lib_bdd::BddVariable;
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

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
    fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Lt => throw_runtime_error("Unsupported operation."),
            CompareOp::Le => throw_runtime_error("Unsupported operation."),
            CompareOp::Eq => Ok(self == other),
            CompareOp::Ne => Ok(self != other),
            CompareOp::Gt => throw_runtime_error("Unsupported operation."),
            CompareOp::Ge => throw_runtime_error("Unsupported operation."),
        }
    }

    fn __hash__(&self) -> isize {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish() as isize
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("BddVariable({})", self.0))
    }

    fn __repr__(&self) -> PyResult<String> {
        self.__str__()
    }
}
