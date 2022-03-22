use crate::bindings::lib_bdd::PyBooleanExpression;
use crate::{throw_runtime_error, AsNative};
use biodivine_lib_bdd::boolean_expression::BooleanExpression;
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

impl From<BooleanExpression> for PyBooleanExpression {
    fn from(value: BooleanExpression) -> Self {
        PyBooleanExpression(value)
    }
}

impl From<PyBooleanExpression> for BooleanExpression {
    fn from(value: PyBooleanExpression) -> Self {
        value.0
    }
}

impl AsNative<BooleanExpression> for PyBooleanExpression {
    fn as_native(&self) -> &BooleanExpression {
        &self.0
    }

    fn as_native_mut(&mut self) -> &mut BooleanExpression {
        &mut self.0
    }
}

#[pymethods]
impl PyBooleanExpression {
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
        self.as_native().to_string().hash(&mut hasher);
        hasher.finish() as isize
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{}", self.0))
    }

    fn __repr__(&self) -> PyResult<String> {
        self.__str__()
    }

    /// Parse a string into a `BooleanExpression`. Raises an exception when the expression
    /// is invalid.
    #[staticmethod]
    pub fn parse(value: &str) -> PyResult<PyBooleanExpression> {
        let parsed = BooleanExpression::try_from(value);
        match parsed {
            Ok(e) => Ok(e.into()),
            Err(message) => throw_runtime_error(message),
        }
    }
}
