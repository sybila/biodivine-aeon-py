use crate::bindings::lib_param_bn::parameter_id::{ParameterId, ParameterIdResolver};
use crate::throw_type_error;
use pyo3::{FromPyObject, PyResult};
use std::fmt::{Display, Formatter};

/// `Union[ParameterId, str]`
#[derive(FromPyObject, Debug, Clone)]
pub enum ParameterIdType {
    Id(ParameterId),
    Name(String),
}

impl Display for ParameterIdType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParameterIdType::Name(name) => write!(f, "{}", name),
            ParameterIdType::Id(id) => write!(f, "{}", id.__str__()),
        }
    }
}

impl ParameterIdType {
    pub fn resolve<T: ParameterIdResolver>(
        &self,
        resolver: &T,
    ) -> PyResult<biodivine_lib_param_bn::ParameterId> {
        let id = match self {
            ParameterIdType::Id(id) => resolver.resolve_id(id.__index__()),
            ParameterIdType::Name(name) => resolver.resolve_name(name),
        };
        if let Some(id) = id {
            Ok(id)
        } else {
            throw_type_error(format!("Parameter {self} is not valid."))
        }
    }
}
