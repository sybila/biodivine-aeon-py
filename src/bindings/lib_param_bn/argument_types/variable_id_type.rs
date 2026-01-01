use crate::bindings::lib_param_bn::variable_id::{
    VariableId, VariableIdResolvable, VariableIdResolver,
};
use crate::{throw_index_error, throw_type_error};
use pyo3::{FromPyObject, PyResult};
use std::fmt::{Display, Formatter};

/// `Union[VariableId, str]`
#[derive(FromPyObject, Debug, Clone, Hash, PartialEq, Eq)]
pub enum VariableIdType {
    Id(VariableId),
    Name(String),
}

impl Display for VariableIdType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            VariableIdType::Name(name) => write!(f, "{}", name),
            VariableIdType::Id(id) => write!(f, "{}", id.__str__()),
        }
    }
}

impl VariableIdResolvable for VariableIdType {
    fn resolve<T: VariableIdResolver>(
        &self,
        resolver: &T,
    ) -> PyResult<biodivine_lib_param_bn::VariableId> {
        let id = match self {
            VariableIdType::Id(id) => resolver.resolve_id(id.__index__()),
            VariableIdType::Name(name) => resolver.resolve_name(name),
        };
        if let Some(id) = id {
            Ok(id)
        } else {
            throw_index_error(format!("Variable `{self}` is not valid."))
        }
    }
}

impl VariableIdType {
    /// Special resolution method that only accepts names, not IDs. (used to avoid duplication
    /// in larger structs that sometimes can involve resolvable variables, but not always)
    pub fn resolve_no_context(&self) -> PyResult<String> {
        match self {
            VariableIdType::Id(_) => throw_type_error("Expected variable name, got ID."),
            VariableIdType::Name(name) => Ok(name.clone()),
        }
    }
}
