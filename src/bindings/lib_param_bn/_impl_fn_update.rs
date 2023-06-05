use crate::bindings::lib_param_bn::{PyBooleanNetwork, PyFnUpdate, PyParameterId, PyVariableId};
use crate::{throw_runtime_error, AsNative};
use biodivine_lib_param_bn::{BinaryOp, BooleanNetwork, FnUpdate, VariableId};
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
        fn recursive(update: &FnUpdate, network: Option<&BooleanNetwork>) -> String {
            match update {
                FnUpdate::Const(value) => {
                    if *value {
                        "true".to_string()
                    } else {
                        "false".to_string()
                    }
                }
                FnUpdate::Var(id) => {
                    if let Some(network) = network {
                        network.get_variable_name(*id).clone()
                    } else {
                        format!("v_{}", id.to_index())
                    }
                }
                FnUpdate::Param(id, args) => {
                    if let Some(network) = network {
                        let name = network.get_parameter(*id).get_name();
                        if args.is_empty() {
                            name.clone()
                        } else {
                            let args = args
                                .iter()
                                .map(|x| network.get_variable_name(*x).clone())
                                .collect::<Vec<_>>();
                            format!("{}({})", name, args.join(", "))
                        }
                    } else if args.is_empty() {
                        format!("p_{}", id.to_index())
                    } else {
                        let args = args.iter().map(|x| format!("{:?}", *x)).collect::<Vec<_>>();
                        format!("p_{}({})", id.to_index(), args.join(", "))
                    }
                }
                FnUpdate::Not(inner) => {
                    format!("!{}", recursive(inner, network))
                }
                FnUpdate::Binary(op, left, right) => {
                    format!(
                        "({} {} {})",
                        recursive(left, network),
                        op,
                        recursive(right, network)
                    )
                }
            }
        }

        recursive(self.as_native(), network.map(|it| it.as_native()))
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
    pub fn from_parameter(id: PyParameterId, args: Option<Vec<PyVariableId>>) -> PyFnUpdate {
        let mut args_native: Vec<VariableId> = Vec::new();
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
                Ok(FnUpdate::Not(Box::new(arguments[0].as_native().clone())).into())
            }
            "and" => {
                assert_eq!(2, arguments.len());
                Ok(FnUpdate::Binary(
                    BinaryOp::And,
                    Box::new(arguments[0].as_native().clone()),
                    Box::new(arguments[1].as_native().clone()),
                )
                .into())
            }
            "or" => {
                assert_eq!(2, arguments.len());
                Ok(FnUpdate::Binary(
                    BinaryOp::Or,
                    Box::new(arguments[0].as_native().clone()),
                    Box::new(arguments[1].as_native().clone()),
                )
                .into())
            }
            "xor" => {
                assert_eq!(2, arguments.len());
                Ok(FnUpdate::Binary(
                    BinaryOp::Xor,
                    Box::new(arguments[0].as_native().clone()),
                    Box::new(arguments[1].as_native().clone()),
                )
                .into())
            }
            "iff" => {
                assert_eq!(2, arguments.len());
                Ok(FnUpdate::Binary(
                    BinaryOp::Iff,
                    Box::new(arguments[0].as_native().clone()),
                    Box::new(arguments[1].as_native().clone()),
                )
                .into())
            }
            "imp" => {
                assert_eq!(2, arguments.len());
                Ok(FnUpdate::Binary(
                    BinaryOp::Imp,
                    Box::new(arguments[0].as_native().clone()),
                    Box::new(arguments[1].as_native().clone()),
                )
                .into())
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

    pub fn as_parameter(&self) -> Option<(PyParameterId, Vec<PyVariableId>)> {
        if let FnUpdate::Param(id, args) = self.as_native() {
            let args = args.iter().map(|it| (*it).into()).collect::<Vec<_>>();
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
}
