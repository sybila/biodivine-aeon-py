use crate::{throw_runtime_error, throw_type_error};
use macros::Wrapper;
use pyo3::prelude::*;

#[pyclass(module = "biodivine_aeon", frozen)]
#[derive(Clone, Debug, Eq, PartialEq, Wrapper)]
pub struct BooleanExpression(biodivine_lib_bdd::boolean_expression::BooleanExpression);

impl BooleanExpression {
    pub fn resolve_expression(value: &PyAny) -> PyResult<BooleanExpression> {
        if let Ok(expression) = value.extract::<BooleanExpression>() {
            return Ok(expression);
        }
        if let Ok(value) = value.extract::<&str>() {
            return match biodivine_lib_bdd::boolean_expression::BooleanExpression::try_from(value) {
                Ok(expression) => Ok(BooleanExpression(expression)),
                Err(message) => {
                    throw_runtime_error(format!("Invalid expression: \"{}\".", message))
                }
            };
        }
        throw_type_error("Expected `BooleanExpression` or `str`.")
    }
}
