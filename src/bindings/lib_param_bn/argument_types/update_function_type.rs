use crate::bindings::lib_param_bn::update_function::UpdateFunction;
use crate::runtime_error;
use biodivine_lib_param_bn::{BooleanNetwork, FnUpdate};
use pyo3::{FromPyObject, PyResult};

#[derive(FromPyObject, Clone)]
pub enum UpdateFunctionType {
    Function(UpdateFunction),
    Expression(String),
}

impl UpdateFunctionType {
    pub fn resolve(&self, network: &BooleanNetwork) -> PyResult<FnUpdate> {
        match self {
            UpdateFunctionType::Expression(expression) => {
                FnUpdate::try_from_str(expression.as_str(), network).map_err(runtime_error)
            }
            UpdateFunctionType::Function(function) => Ok(function.as_native().clone()),
        }
    }
}
