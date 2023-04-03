use crate::bindings::lib_bdd::{PyBooleanExpression, PyBooleanExpressionRef};
use crate::{throw_runtime_error, throw_type_error, AsNative};
use biodivine_lib_bdd::boolean_expression::BooleanExpression;
use biodivine_lib_bdd::boolean_expression::BooleanExpression::Imp;
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use BooleanExpression::{And, Iff, Or, Xor};

#[allow(clippy::wrong_self_convention)]
#[pymethods]
impl PyBooleanExpression {
    #[new]
    pub fn new(value: &str) -> PyResult<PyBooleanExpression> {
        let parsed = BooleanExpression::try_from(value);
        match parsed {
            Ok(e) => Ok(e.into()),
            Err(message) => throw_runtime_error(message),
        }
    }

    pub fn __str__(&self) -> String {
        format!("{}", self.0)
    }

    pub fn __repr__(&self) -> String {
        format!("BooleanExpression(\"{}\")", self.__str__())
    }

    #[pyo3(signature = (valuation = None, **kwargs))]
    pub fn __call__(&self, valuation: Option<&PyDict>, kwargs: Option<&PyDict>) -> PyResult<bool> {
        match (valuation, kwargs) {
            (Some(_), Some(_)) => {
                throw_runtime_error("Cannot use both explicit and named arguments.")
            }
            (None, None) => throw_runtime_error("Missing valuation."),
            (Some(v), None) | (None, Some(v)) => eval(self.as_native(), v),
        }
    }

    pub fn __eq__(&self, other: &PyAny) -> PyResult<bool> {
        let py = other.py();
        if let Ok(expression) = other.extract::<Py<PyBooleanExpression>>() {
            let expression = expression.borrow(py);
            Ok(self.as_native() == expression.as_native())
        } else if let Ok(reference) = other.extract::<Py<PyBooleanExpressionRef>>() {
            let reference = reference.borrow(py);
            Ok(self.as_native() == reference.reference)
        } else {
            throw_type_error("Expected BooleanExpression or BooleanExpressionRef.")
        }
    }

    pub fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Lt => throw_runtime_error("Unsupported operation."),
            CompareOp::Le => throw_runtime_error("Unsupported operation."),
            CompareOp::Eq => self.__eq__(other),
            CompareOp::Ne => self.__eq__(other).map(|it| !it),
            CompareOp::Gt => throw_runtime_error("Unsupported operation."),
            CompareOp::Ge => throw_runtime_error("Unsupported operation."),
        }
    }

    pub fn __hash__(&self) -> isize {
        let mut hasher = DefaultHasher::new();
        self.as_native().to_string().hash(&mut hasher);
        hasher.finish() as isize
    }

    #[staticmethod]
    pub fn mk_const(value: bool) -> PyBooleanExpression {
        BooleanExpression::Const(value).into()
    }

    #[staticmethod]
    pub fn mk_var(name: String) -> PyBooleanExpression {
        BooleanExpression::Variable(name).into()
    }

    #[staticmethod]
    pub fn mk_not(value: PyBooleanExpression) -> PyBooleanExpression {
        BooleanExpression::Not(Box::new(value.as_native().clone())).into()
    }

    #[staticmethod]
    pub fn mk_and(left: PyBooleanExpression, right: PyBooleanExpression) -> PyBooleanExpression {
        And(
            Box::new(left.as_native().clone()),
            Box::new(right.as_native().clone()),
        )
        .into()
    }

    #[staticmethod]
    pub fn mk_or(left: PyBooleanExpression, right: PyBooleanExpression) -> PyBooleanExpression {
        Or(
            Box::new(left.as_native().clone()),
            Box::new(right.as_native().clone()),
        )
        .into()
    }

    #[staticmethod]
    pub fn mk_imp(left: PyBooleanExpression, right: PyBooleanExpression) -> PyBooleanExpression {
        Imp(
            Box::new(left.as_native().clone()),
            Box::new(right.as_native().clone()),
        )
        .into()
    }

    #[staticmethod]
    pub fn mk_iff(left: PyBooleanExpression, right: PyBooleanExpression) -> PyBooleanExpression {
        Iff(
            Box::new(left.as_native().clone()),
            Box::new(right.as_native().clone()),
        )
        .into()
    }

    #[staticmethod]
    pub fn mk_xor(left: PyBooleanExpression, right: PyBooleanExpression) -> PyBooleanExpression {
        Xor(
            Box::new(left.as_native().clone()),
            Box::new(right.as_native().clone()),
        )
        .into()
    }

    pub fn is_const(&self) -> bool {
        matches!(self.as_native(), &BooleanExpression::Const(_))
    }

    pub fn is_var(&self) -> bool {
        matches!(self.as_native(), &BooleanExpression::Variable(_))
    }

    pub fn is_not(&self) -> bool {
        matches!(self.as_native(), &BooleanExpression::Not(_))
    }

    pub fn is_and(&self) -> bool {
        matches!(self.as_native(), &BooleanExpression::And(_, _))
    }

    pub fn is_or(&self) -> bool {
        matches!(self.as_native(), &BooleanExpression::Or(_, _))
    }

    pub fn is_imp(&self) -> bool {
        matches!(self.as_native(), &BooleanExpression::Imp(_, _))
    }

    pub fn is_iff(&self) -> bool {
        matches!(self.as_native(), &BooleanExpression::Iff(_, _))
    }

    pub fn is_xor(&self) -> bool {
        matches!(self.as_native(), &BooleanExpression::Xor(_, _))
    }

    pub fn is_literal(&self) -> bool {
        match self.as_native() {
            BooleanExpression::Variable(_) => true,
            BooleanExpression::Not(inner) => {
                matches!(**inner, BooleanExpression::Variable(_))
            }
            _ => false,
        }
    }

    pub fn is_binary(&self) -> bool {
        !matches!(
            self.as_native(),
            BooleanExpression::Const(_)
                | BooleanExpression::Variable(_)
                | BooleanExpression::Not(_)
        )
    }

    pub fn as_const(&self) -> Option<bool> {
        match self.as_native() {
            BooleanExpression::Const(x) => Some(*x),
            _ => None,
        }
    }

    pub fn as_var(&self) -> Option<String> {
        match self.as_native() {
            BooleanExpression::Variable(x) => Some(x.clone()),
            _ => None,
        }
    }

    pub fn as_not(self_: Py<PyBooleanExpression>, py: Python) -> Option<PyBooleanExpressionRef> {
        let child = {
            let my_ref = self_.borrow(py);
            match my_ref.as_native() {
                BooleanExpression::Not(x) => {
                    let child_static: &'static BooleanExpression =
                        unsafe { (x.as_ref() as *const BooleanExpression).as_ref().unwrap() };
                    Some(child_static)
                }
                _ => None,
            }
        };
        child.map(|child| PyBooleanExpressionRef {
            root: self_,
            reference: child,
        })
    }

    pub fn as_and(
        self_: Py<PyBooleanExpression>,
        py: Python,
    ) -> Option<(PyBooleanExpressionRef, PyBooleanExpressionRef)> {
        let child = {
            let my_ref = self_.borrow(py);
            match my_ref.as_native() {
                And(l, r) => {
                    let l_static: &'static BooleanExpression =
                        unsafe { (l.as_ref() as *const BooleanExpression).as_ref().unwrap() };
                    let r_static: &'static BooleanExpression =
                        unsafe { (r.as_ref() as *const BooleanExpression).as_ref().unwrap() };
                    Some((l_static, r_static))
                }
                _ => None,
            }
        };
        child.map(|(l, r)| {
            let l = PyBooleanExpressionRef {
                root: self_.clone(),
                reference: l,
            };
            let r = PyBooleanExpressionRef {
                root: self_,
                reference: r,
            };
            (l, r)
        })
    }

    pub fn as_or(
        self_: Py<PyBooleanExpression>,
        py: Python,
    ) -> Option<(PyBooleanExpressionRef, PyBooleanExpressionRef)> {
        let child = {
            let my_ref = self_.borrow(py);
            match my_ref.as_native() {
                Or(l, r) => {
                    let l_static: &'static BooleanExpression =
                        unsafe { (l.as_ref() as *const BooleanExpression).as_ref().unwrap() };
                    let r_static: &'static BooleanExpression =
                        unsafe { (r.as_ref() as *const BooleanExpression).as_ref().unwrap() };
                    Some((l_static, r_static))
                }
                _ => None,
            }
        };
        child.map(|(l, r)| {
            let l = PyBooleanExpressionRef {
                root: self_.clone(),
                reference: l,
            };
            let r = PyBooleanExpressionRef {
                root: self_,
                reference: r,
            };
            (l, r)
        })
    }

    pub fn as_imp(
        self_: Py<PyBooleanExpression>,
        py: Python,
    ) -> Option<(PyBooleanExpressionRef, PyBooleanExpressionRef)> {
        let child = {
            let my_ref = self_.borrow(py);
            match my_ref.as_native() {
                Imp(l, r) => {
                    let l_static: &'static BooleanExpression =
                        unsafe { (l.as_ref() as *const BooleanExpression).as_ref().unwrap() };
                    let r_static: &'static BooleanExpression =
                        unsafe { (r.as_ref() as *const BooleanExpression).as_ref().unwrap() };
                    Some((l_static, r_static))
                }
                _ => None,
            }
        };
        child.map(|(l, r)| {
            let l = PyBooleanExpressionRef {
                root: self_.clone(),
                reference: l,
            };
            let r = PyBooleanExpressionRef {
                root: self_,
                reference: r,
            };
            (l, r)
        })
    }

    pub fn as_iff(
        self_: Py<PyBooleanExpression>,
        py: Python,
    ) -> Option<(PyBooleanExpressionRef, PyBooleanExpressionRef)> {
        let child = {
            let my_ref = self_.borrow(py);
            match my_ref.as_native() {
                Iff(l, r) => {
                    let l_static: &'static BooleanExpression =
                        unsafe { (l.as_ref() as *const BooleanExpression).as_ref().unwrap() };
                    let r_static: &'static BooleanExpression =
                        unsafe { (r.as_ref() as *const BooleanExpression).as_ref().unwrap() };
                    Some((l_static, r_static))
                }
                _ => None,
            }
        };
        child.map(|(l, r)| {
            let l = PyBooleanExpressionRef {
                root: self_.clone(),
                reference: l,
            };
            let r = PyBooleanExpressionRef {
                root: self_,
                reference: r,
            };
            (l, r)
        })
    }

    pub fn as_xor(
        self_: Py<PyBooleanExpression>,
        py: Python,
    ) -> Option<(PyBooleanExpressionRef, PyBooleanExpressionRef)> {
        let child = {
            let my_ref = self_.borrow(py);
            match my_ref.as_native() {
                Xor(l, r) => {
                    let l_static: &'static BooleanExpression =
                        unsafe { (l.as_ref() as *const BooleanExpression).as_ref().unwrap() };
                    let r_static: &'static BooleanExpression =
                        unsafe { (r.as_ref() as *const BooleanExpression).as_ref().unwrap() };
                    Some((l_static, r_static))
                }
                _ => None,
            }
        };
        child.map(|(l, r)| {
            let l = PyBooleanExpressionRef {
                root: self_.clone(),
                reference: l,
            };
            let r = PyBooleanExpressionRef {
                root: self_,
                reference: r,
            };
            (l, r)
        })
    }

    pub fn as_literal(&self) -> Option<(String, bool)> {
        match self.as_native() {
            BooleanExpression::Variable(name) => Some((name.clone(), true)),
            BooleanExpression::Not(inner) => match inner.as_ref() {
                BooleanExpression::Variable(name) => Some((name.clone(), false)),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn as_binary(
        self_: Py<PyBooleanExpression>,
        py: Python,
    ) -> Option<(PyBooleanExpressionRef, PyBooleanExpressionRef)> {
        let child = {
            let my_ref = self_.borrow(py);
            match my_ref.as_native() {
                And(l, r) | Or(l, r) | Imp(l, r) | Iff(l, r) | Xor(l, r) => {
                    let l_static: &'static BooleanExpression =
                        unsafe { (l.as_ref() as *const BooleanExpression).as_ref().unwrap() };
                    let r_static: &'static BooleanExpression =
                        unsafe { (r.as_ref() as *const BooleanExpression).as_ref().unwrap() };
                    Some((l_static, r_static))
                }
                _ => None,
            }
        };
        child.map(|(l, r)| {
            let l = PyBooleanExpressionRef {
                root: self_.clone(),
                reference: l,
            };
            let r = PyBooleanExpressionRef {
                root: self_,
                reference: r,
            };
            (l, r)
        })
    }

    pub fn support_set(&self) -> HashSet<String> {
        fn recursive(e: &BooleanExpression, result: &mut HashSet<String>) {
            match e {
                BooleanExpression::Const(_) => (),
                BooleanExpression::Variable(name) => {
                    result.insert(name.clone());
                }
                BooleanExpression::Not(inner) => recursive(inner, result),
                And(l, r) | Or(l, r) | Imp(l, r) | Iff(l, r) | Xor(l, r) => {
                    recursive(l, result);
                    recursive(r, result);
                }
            };
        }
        let mut result = HashSet::new();
        recursive(self.as_native(), &mut result);
        result
    }

    pub fn clone(&self) -> PyBooleanExpression {
        self.as_native().clone().into()
    }
}

impl PyBooleanExpressionRef {
    fn as_native(&self) -> &'static BooleanExpression {
        self.reference
    }
}

#[allow(clippy::wrong_self_convention)]
#[pymethods]
impl PyBooleanExpressionRef {
    pub fn __str__(&self) -> String {
        format!("{}", self.reference)
    }

    pub fn __repr__(&self) -> String {
        format!("BooleanExpression(\"{}\")", self.__str__())
    }

    #[pyo3(signature = (valuation = None, **kwargs))]
    pub fn __call__(&self, valuation: Option<&PyDict>, kwargs: Option<&PyDict>) -> PyResult<bool> {
        match (valuation, kwargs) {
            (Some(_), Some(_)) => {
                throw_runtime_error("Cannot use both explicit and named arguments.")
            }
            (None, None) => throw_runtime_error("Missing valuation."),
            (Some(v), None) | (None, Some(v)) => eval(self.as_native(), v),
        }
    }

    pub fn __eq__(&self, other: &PyAny) -> PyResult<bool> {
        let py = other.py();
        if let Ok(expression) = other.extract::<Py<PyBooleanExpression>>() {
            let expression = expression.borrow(py);
            Ok(self.reference == expression.as_native())
        } else if let Ok(reference) = other.extract::<Py<PyBooleanExpressionRef>>() {
            let reference = reference.borrow(py);
            Ok(self.reference == reference.reference)
        } else {
            throw_type_error("Expected BooleanExpression or BooleanExpressionRef.")
        }
    }

    pub fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Lt => throw_runtime_error("Unsupported operation."),
            CompareOp::Le => throw_runtime_error("Unsupported operation."),
            CompareOp::Eq => self.__eq__(other),
            CompareOp::Ne => self.__eq__(other).map(|it| !it),
            CompareOp::Gt => throw_runtime_error("Unsupported operation."),
            CompareOp::Ge => throw_runtime_error("Unsupported operation."),
        }
    }

    pub fn __hash__(&self) -> isize {
        let mut hasher = DefaultHasher::new();
        self.reference.to_string().hash(&mut hasher);
        hasher.finish() as isize
    }

    pub fn is_const(&self) -> bool {
        matches!(self.as_native(), &BooleanExpression::Const(_))
    }

    pub fn is_var(&self) -> bool {
        matches!(self.as_native(), &BooleanExpression::Variable(_))
    }

    pub fn is_not(&self) -> bool {
        matches!(self.as_native(), &BooleanExpression::Not(_))
    }

    pub fn is_and(&self) -> bool {
        matches!(self.as_native(), &BooleanExpression::And(_, _))
    }

    pub fn is_or(&self) -> bool {
        matches!(self.as_native(), &BooleanExpression::Or(_, _))
    }

    pub fn is_imp(&self) -> bool {
        matches!(self.as_native(), &BooleanExpression::Imp(_, _))
    }

    pub fn is_iff(&self) -> bool {
        matches!(self.as_native(), &BooleanExpression::Iff(_, _))
    }

    pub fn is_xor(&self) -> bool {
        matches!(self.as_native(), &BooleanExpression::Xor(_, _))
    }

    pub fn is_literal(&self) -> bool {
        match self.as_native() {
            BooleanExpression::Variable(_) => true,
            BooleanExpression::Not(inner) => {
                matches!(**inner, BooleanExpression::Variable(_))
            }
            _ => false,
        }
    }

    pub fn is_binary(&self) -> bool {
        !matches!(
            self.as_native(),
            BooleanExpression::Const(_)
                | BooleanExpression::Variable(_)
                | BooleanExpression::Not(_)
        )
    }

    pub fn as_const(&self) -> Option<bool> {
        match self.as_native() {
            BooleanExpression::Const(x) => Some(*x),
            _ => None,
        }
    }

    pub fn as_var(&self) -> Option<String> {
        match self.as_native() {
            BooleanExpression::Variable(x) => Some(x.clone()),
            _ => None,
        }
    }

    pub fn as_not(&self) -> Option<PyBooleanExpressionRef> {
        let child = {
            match self.as_native() {
                BooleanExpression::Not(x) => {
                    let child_static: &'static BooleanExpression =
                        unsafe { (x.as_ref() as *const BooleanExpression).as_ref().unwrap() };
                    Some(child_static)
                }
                _ => None,
            }
        };
        child.map(|child| PyBooleanExpressionRef {
            root: self.root.clone(),
            reference: child,
        })
    }

    pub fn as_and(&self) -> Option<(PyBooleanExpressionRef, PyBooleanExpressionRef)> {
        let child = {
            match self.as_native() {
                And(l, r) => {
                    let l_static: &'static BooleanExpression =
                        unsafe { (l.as_ref() as *const BooleanExpression).as_ref().unwrap() };
                    let r_static: &'static BooleanExpression =
                        unsafe { (r.as_ref() as *const BooleanExpression).as_ref().unwrap() };
                    Some((l_static, r_static))
                }
                _ => None,
            }
        };
        child.map(|(l, r)| {
            let l = PyBooleanExpressionRef {
                root: self.root.clone(),
                reference: l,
            };
            let r = PyBooleanExpressionRef {
                root: self.root.clone(),
                reference: r,
            };
            (l, r)
        })
    }

    pub fn as_or(&self) -> Option<(PyBooleanExpressionRef, PyBooleanExpressionRef)> {
        let child = {
            match self.as_native() {
                Or(l, r) => {
                    let l_static: &'static BooleanExpression =
                        unsafe { (l.as_ref() as *const BooleanExpression).as_ref().unwrap() };
                    let r_static: &'static BooleanExpression =
                        unsafe { (r.as_ref() as *const BooleanExpression).as_ref().unwrap() };
                    Some((l_static, r_static))
                }
                _ => None,
            }
        };
        child.map(|(l, r)| {
            let l = PyBooleanExpressionRef {
                root: self.root.clone(),
                reference: l,
            };
            let r = PyBooleanExpressionRef {
                root: self.root.clone(),
                reference: r,
            };
            (l, r)
        })
    }

    pub fn as_imp(&self) -> Option<(PyBooleanExpressionRef, PyBooleanExpressionRef)> {
        let child = {
            match self.as_native() {
                Imp(l, r) => {
                    let l_static: &'static BooleanExpression =
                        unsafe { (l.as_ref() as *const BooleanExpression).as_ref().unwrap() };
                    let r_static: &'static BooleanExpression =
                        unsafe { (r.as_ref() as *const BooleanExpression).as_ref().unwrap() };
                    Some((l_static, r_static))
                }
                _ => None,
            }
        };
        child.map(|(l, r)| {
            let l = PyBooleanExpressionRef {
                root: self.root.clone(),
                reference: l,
            };
            let r = PyBooleanExpressionRef {
                root: self.root.clone(),
                reference: r,
            };
            (l, r)
        })
    }

    pub fn as_iff(&self) -> Option<(PyBooleanExpressionRef, PyBooleanExpressionRef)> {
        let child = {
            match self.as_native() {
                Iff(l, r) => {
                    let l_static: &'static BooleanExpression =
                        unsafe { (l.as_ref() as *const BooleanExpression).as_ref().unwrap() };
                    let r_static: &'static BooleanExpression =
                        unsafe { (r.as_ref() as *const BooleanExpression).as_ref().unwrap() };
                    Some((l_static, r_static))
                }
                _ => None,
            }
        };
        child.map(|(l, r)| {
            let l = PyBooleanExpressionRef {
                root: self.root.clone(),
                reference: l,
            };
            let r = PyBooleanExpressionRef {
                root: self.root.clone(),
                reference: r,
            };
            (l, r)
        })
    }

    pub fn as_xor(&self) -> Option<(PyBooleanExpressionRef, PyBooleanExpressionRef)> {
        let child = {
            match self.as_native() {
                Xor(l, r) => {
                    let l_static: &'static BooleanExpression =
                        unsafe { (l.as_ref() as *const BooleanExpression).as_ref().unwrap() };
                    let r_static: &'static BooleanExpression =
                        unsafe { (r.as_ref() as *const BooleanExpression).as_ref().unwrap() };
                    Some((l_static, r_static))
                }
                _ => None,
            }
        };
        child.map(|(l, r)| {
            let l = PyBooleanExpressionRef {
                root: self.root.clone(),
                reference: l,
            };
            let r = PyBooleanExpressionRef {
                root: self.root.clone(),
                reference: r,
            };
            (l, r)
        })
    }

    pub fn as_literal(&self) -> Option<(String, bool)> {
        match self.as_native() {
            BooleanExpression::Variable(name) => Some((name.clone(), true)),
            BooleanExpression::Not(inner) => match inner.as_ref() {
                BooleanExpression::Variable(name) => Some((name.clone(), false)),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn as_binary(&self) -> Option<(PyBooleanExpressionRef, PyBooleanExpressionRef)> {
        let child = {
            match self.as_native() {
                And(l, r) | Or(l, r) | Imp(l, r) | Iff(l, r) | Xor(l, r) => {
                    let l_static: &'static BooleanExpression =
                        unsafe { (l.as_ref() as *const BooleanExpression).as_ref().unwrap() };
                    let r_static: &'static BooleanExpression =
                        unsafe { (r.as_ref() as *const BooleanExpression).as_ref().unwrap() };
                    Some((l_static, r_static))
                }
                _ => None,
            }
        };
        child.map(|(l, r)| {
            let l = PyBooleanExpressionRef {
                root: self.root.clone(),
                reference: l,
            };
            let r = PyBooleanExpressionRef {
                root: self.root.clone(),
                reference: r,
            };
            (l, r)
        })
    }

    pub fn support_set(&self) -> HashSet<String> {
        fn recursive(e: &BooleanExpression, result: &mut HashSet<String>) {
            match e {
                BooleanExpression::Const(_) => (),
                BooleanExpression::Variable(name) => {
                    result.insert(name.clone());
                }
                BooleanExpression::Not(inner) => recursive(inner, result),
                And(l, r) | Or(l, r) | Imp(l, r) | Iff(l, r) | Xor(l, r) => {
                    recursive(l, result);
                    recursive(r, result);
                }
            };
        }
        let mut result = HashSet::new();
        recursive(self.as_native(), &mut result);
        result
    }

    pub fn clone(&self) -> PyBooleanExpression {
        self.reference.clone().into()
    }
}

fn eval(e: &BooleanExpression, valuation: &PyDict) -> PyResult<bool> {
    match e {
        BooleanExpression::Const(x) => Ok(*x),
        BooleanExpression::Variable(name) => {
            let Some(value) = valuation.get_item(name) else {
                return throw_runtime_error(format!("Missing value of {}.", name))
            };
            value.extract::<bool>()
        }
        BooleanExpression::Not(inner) => {
            let inner = eval(inner, valuation)?;
            Ok(!inner)
        }
        And(left, right) => {
            let left = eval(left, valuation)?;
            let right = eval(right, valuation)?;
            Ok(left && right)
        }
        Or(left, right) => {
            let left = eval(left, valuation)?;
            let right = eval(right, valuation)?;
            Ok(left || right)
        }
        Xor(left, right) => {
            let left = eval(left, valuation)?;
            let right = eval(right, valuation)?;
            Ok(left != right)
        }
        Imp(left, right) => {
            let left = eval(left, valuation)?;
            let right = eval(right, valuation)?;
            Ok(!left || right)
        }
        Iff(left, right) => {
            let left = eval(left, valuation)?;
            let right = eval(right, valuation)?;
            Ok(left == right)
        }
    }
}
