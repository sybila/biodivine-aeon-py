use crate::bindings::lib_bdd::PyBooleanExpression;
use crate::{throw_runtime_error, AsNative};
use biodivine_lib_bdd::boolean_expression::BooleanExpression;
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

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

    #[new]
    pub fn new(value: &str) -> PyResult<PyBooleanExpression> {
        let parsed = BooleanExpression::try_from(value);
        match parsed {
            Ok(e) => Ok(e.into()),
            Err(message) => throw_runtime_error(message),
        }
    }

    #[staticmethod]
    pub fn from_constant(value: bool) -> PyBooleanExpression {
        BooleanExpression::Const(value).into()
    }

    #[staticmethod]
    pub fn from_variable(value: String) -> PyBooleanExpression {
        BooleanExpression::Variable(value).into()
    }

    #[staticmethod]
    pub fn from_formula(
        operator: String,
        arguments: Vec<PyBooleanExpression>,
    ) -> PyResult<PyBooleanExpression> {
        match operator.as_str() {
            "not" => {
                assert_eq!(1, arguments.len());
                Ok(BooleanExpression::Not(Box::new(arguments[0].as_native().clone())).into())
            }
            "and" => {
                assert_eq!(2, arguments.len());
                Ok(BooleanExpression::And(
                    Box::new(arguments[0].as_native().clone()),
                    Box::new(arguments[1].as_native().clone()),
                )
                .into())
            }
            "or" => {
                assert_eq!(2, arguments.len());
                Ok(BooleanExpression::Or(
                    Box::new(arguments[0].as_native().clone()),
                    Box::new(arguments[1].as_native().clone()),
                )
                .into())
            }
            "xor" => {
                assert_eq!(2, arguments.len());
                Ok(BooleanExpression::Xor(
                    Box::new(arguments[0].as_native().clone()),
                    Box::new(arguments[1].as_native().clone()),
                )
                .into())
            }
            "iff" => {
                assert_eq!(2, arguments.len());
                Ok(BooleanExpression::Iff(
                    Box::new(arguments[0].as_native().clone()),
                    Box::new(arguments[1].as_native().clone()),
                )
                .into())
            }
            "imp" => {
                assert_eq!(2, arguments.len());
                Ok(BooleanExpression::Imp(
                    Box::new(arguments[0].as_native().clone()),
                    Box::new(arguments[1].as_native().clone()),
                )
                .into())
            }
            _ => throw_runtime_error(format!("Unknown operator: {operator}.")),
        }
    }

    pub fn as_constant(&self) -> Option<bool> {
        if let BooleanExpression::Const(x) = self.as_native() {
            Some(*x)
        } else {
            None
        }
    }

    pub fn as_variable(&self) -> Option<String> {
        if let BooleanExpression::Variable(x) = self.as_native() {
            Some(x.clone())
        } else {
            None
        }
    }

    pub fn as_formula(&self) -> Option<(String, Vec<PyBooleanExpression>)> {
        match self.as_native() {
            BooleanExpression::Not(inner) => {
                Some(("not".to_string(), vec![inner.as_ref().clone().into()]))
            }
            BooleanExpression::And(left, right) => Some((
                "and".to_string(),
                vec![left.as_ref().clone().into(), right.as_ref().clone().into()],
            )),
            BooleanExpression::Or(left, right) => Some((
                "or".to_string(),
                vec![left.as_ref().clone().into(), right.as_ref().clone().into()],
            )),
            BooleanExpression::Xor(left, right) => Some((
                "xor".to_string(),
                vec![left.as_ref().clone().into(), right.as_ref().clone().into()],
            )),
            BooleanExpression::Imp(left, right) => Some((
                "imp".to_string(),
                vec![left.as_ref().clone().into(), right.as_ref().clone().into()],
            )),
            BooleanExpression::Iff(left, right) => Some((
                "iff".to_string(),
                vec![left.as_ref().clone().into(), right.as_ref().clone().into()],
            )),
            _ => None,
        }
    }
}
