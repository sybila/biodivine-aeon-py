use crate::bindings::lib_param_bn::PyVariableId;
use crate::throw_runtime_error;
use biodivine_lib_param_bn::VariableId;
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

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
        Ok(format!("{:?}", self.0))
    }

    fn __repr__(&self) -> PyResult<String> {
        self.__str__()
    }
}
