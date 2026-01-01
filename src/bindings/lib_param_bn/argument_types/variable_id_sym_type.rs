use crate::bindings::lib_bdd::bdd_variable::BddVariable;
use crate::bindings::lib_param_bn::variable_id::{
    VariableId, VariableIdResolvable, VariableIdResolver,
};
use crate::{AsNative, throw_index_error};
use pyo3::{FromPyObject, PyResult};
use std::fmt::{Display, Formatter};

/// `Union[VariableId, BddVariable, str]`
#[derive(FromPyObject, Debug, Clone)]
pub enum VariableIdSymType {
    Id(VariableId),
    Sym(BddVariable),
    Name(String),
}

impl Display for VariableIdSymType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            VariableIdSymType::Name(name) => write!(f, "{}", name),
            VariableIdSymType::Sym(id) => write!(f, "{}", id.__str__()),
            VariableIdSymType::Id(id) => write!(f, "{}", id.__str__()),
        }
    }
}

impl VariableIdResolvable for VariableIdSymType {
    fn resolve<T: VariableIdResolver>(
        &self,
        resolver: &T,
    ) -> PyResult<biodivine_lib_param_bn::VariableId> {
        let id = match self {
            VariableIdSymType::Id(id) => resolver.resolve_id(id.__index__()),
            VariableIdSymType::Sym(id) => resolver.resolve_symbolic(*id.as_native()),
            VariableIdSymType::Name(name) => resolver.resolve_name(name),
        };
        if let Some(id) = id {
            Ok(id)
        } else {
            throw_index_error(format!("Variable `{self}` is not valid."))
        }
    }
}
