use crate::bindings::lib_param_bn::{PyBooleanNetwork, PyFnUpdate, PyParameterId, PyVariableId};
use crate::{throw_runtime_error, AsNative};
use biodivine_lib_param_bn::{BinaryOp, FnUpdate};
use pyo3::basic::CompareOp;
use pyo3::{pymethods, PyResult};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[pymethods]
impl PyFnUpdate {
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
        self.as_native().hash(&mut hasher);
        hasher.finish() as isize
    }

    fn __str__(&self) -> String {
        self.to_string(None)
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }

    /// Convert this update function to a string. If a network is supplied, the actual
    /// variable/parameter names are used. Otherwise, names are substituted for "v_id" (variables)
    /// and "p_id" (parameters).
    pub fn to_string(&self, network: Option<&PyBooleanNetwork>) -> String {
        if let Some(network) = network {
            self.as_native().to_string(network.as_native())
        } else {
            format!("{}", self.as_native())
        }
    }

    /// Build a new expression which uses the variables and parameters from the given network.
    #[new]
    pub fn new(value: &str, network: &PyBooleanNetwork) -> PyResult<PyFnUpdate> {
        match FnUpdate::try_from_str(value, network.as_native()) {
            Ok(function) => Ok(function.into()),
            Err(error) => throw_runtime_error(error),
        }
    }

    #[staticmethod]
    pub fn from_constant(value: bool) -> PyFnUpdate {
        FnUpdate::Const(value).into()
    }

    #[staticmethod]
    pub fn from_variable(id: PyVariableId) -> PyFnUpdate {
        FnUpdate::mk_var(id.into()).into()
    }

    #[staticmethod]
    pub fn from_parameter(id: PyParameterId, args: Option<Vec<PyFnUpdate>>) -> PyFnUpdate {
        let mut args_native: Vec<FnUpdate> = Vec::new();
        if let Some(args) = args {
            for a in args {
                args_native.push(a.into());
            }
        }
        FnUpdate::mk_param(id.into(), &args_native).into()
    }

    #[staticmethod]
    pub fn from_formula(operator: String, arguments: Vec<PyFnUpdate>) -> PyResult<PyFnUpdate> {
        match operator.as_str() {
            "not" => {
                assert_eq!(1, arguments.len());
                let f = FnUpdate::mk_not(arguments[0].as_native().clone());
                Ok(f.into())
            }
            "and" => {
                let args = arguments
                    .into_iter()
                    .map(|it| it.as_native().clone())
                    .collect::<Vec<_>>();
                let f = FnUpdate::mk_conjunction(&args);
                Ok(f.into())
            }
            "or" => {
                let args = arguments
                    .into_iter()
                    .map(|it| it.as_native().clone())
                    .collect::<Vec<_>>();
                let f = FnUpdate::mk_disjunction(&args);
                Ok(f.into())
            }
            "xor" => {
                assert_eq!(2, arguments.len());
                let l = arguments[0].as_native().clone();
                let r = arguments[1].as_native().clone();
                Ok(l.xor(r).into())
            }
            "iff" => {
                assert_eq!(2, arguments.len());
                let l = arguments[0].as_native().clone();
                let r = arguments[1].as_native().clone();
                Ok(l.iff(r).into())
            }
            "imp" => {
                assert_eq!(2, arguments.len());
                let l = arguments[0].as_native().clone();
                let r = arguments[1].as_native().clone();
                Ok(l.implies(r).into())
            }
            _ => throw_runtime_error(format!("Unknown operator: {operator}.")),
        }
    }

    pub fn as_constant(&self) -> Option<bool> {
        if let FnUpdate::Const(x) = self.as_native() {
            Some(*x)
        } else {
            None
        }
    }

    pub fn as_variable(&self) -> Option<PyVariableId> {
        if let FnUpdate::Var(x) = self.as_native() {
            Some((*x).into())
        } else {
            None
        }
    }

    pub fn as_parameter(&self) -> Option<(PyParameterId, Vec<PyFnUpdate>)> {
        if let FnUpdate::Param(id, args) = self.as_native() {
            let args = args
                .iter()
                .map(|it| (*it).clone().into())
                .collect::<Vec<_>>();
            Some(((*id).into(), args))
        } else {
            None
        }
    }

    pub fn as_formula(&self) -> Option<(String, Vec<PyFnUpdate>)> {
        match self.as_native() {
            FnUpdate::Not(inner) => Some(("not".to_string(), vec![inner.as_ref().clone().into()])),
            FnUpdate::Binary(BinaryOp::And, left, right) => Some((
                "and".to_string(),
                vec![left.as_ref().clone().into(), right.as_ref().clone().into()],
            )),
            FnUpdate::Binary(BinaryOp::Or, left, right) => Some((
                "or".to_string(),
                vec![left.as_ref().clone().into(), right.as_ref().clone().into()],
            )),
            FnUpdate::Binary(BinaryOp::Xor, left, right) => Some((
                "xor".to_string(),
                vec![left.as_ref().clone().into(), right.as_ref().clone().into()],
            )),
            FnUpdate::Binary(BinaryOp::Imp, left, right) => Some((
                "imp".to_string(),
                vec![left.as_ref().clone().into(), right.as_ref().clone().into()],
            )),
            FnUpdate::Binary(BinaryOp::Iff, left, right) => Some((
                "iff".to_string(),
                vec![left.as_ref().clone().into(), right.as_ref().clone().into()],
            )),
            _ => None,
        }
    }

    /// Substitutes every occurrence of `var` with `function`. Returns `None` when the substitution
    /// is impossible.
    ///
    /// A substitution is impossible when `var` appears as argument of an uninterpreted function.
    pub fn substitute_variable(&self, var: PyVariableId, function: PyFnUpdate) -> PyFnUpdate {
        self.as_native()
            .substitute_variable(var.into(), function.as_native())
            .into()
    }
}
