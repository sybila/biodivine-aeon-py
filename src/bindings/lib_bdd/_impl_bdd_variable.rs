use crate::bindings::lib_bdd::PyBddVariable;
use crate::AsNative;
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[pymethods]
impl PyBddVariable {
    fn __hash__(&self) -> isize {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish() as isize
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> bool {
        let left = self.as_native();
        let right = other.as_native();
        match op {
            CompareOp::Lt => left < right,
            CompareOp::Le => left <= right,
            CompareOp::Eq => left == right,
            CompareOp::Ne => left != right,
            CompareOp::Gt => left > right,
            CompareOp::Ge => left >= right,
        }
    }

    fn __str__(&self) -> String {
        format!("BddVariable({})", self.0)
    }

    fn __repr__(&self) -> String {
        format!("<{}>", self.__str__())
    }
}
