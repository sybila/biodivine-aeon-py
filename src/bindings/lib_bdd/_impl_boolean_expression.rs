use crate::bindings::lib_bdd::PyBooleanExpression;
use crate::{throw_runtime_error, AsNative};
use biodivine_lib_bdd::boolean_expression::BooleanExpression;
use pyo3::prelude::*;

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
