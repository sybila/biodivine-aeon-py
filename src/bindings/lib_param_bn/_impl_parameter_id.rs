use super::PyParameterId;
use biodivine_lib_param_bn::ParameterId;
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[pymethods]
impl PyParameterId {
    fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Lt => Ok(self < other),
            CompareOp::Le => Ok(self <= other),
            CompareOp::Eq => Ok(self == other),
            CompareOp::Ne => Ok(self != other),
            CompareOp::Gt => Ok(self > other),
            CompareOp::Ge => Ok(self >= other),
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

    #[staticmethod]
    pub fn from_index(value: usize) -> PyParameterId {
        ParameterId::from_index(value).into()
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn into_index(&self) -> usize {
        self.0.to_index()
    }
}
