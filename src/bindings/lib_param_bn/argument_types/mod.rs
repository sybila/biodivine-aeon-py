use crate::bindings::lib_param_bn::parameter_id::{ParameterId, ParameterIdResolver};
use crate::bindings::lib_param_bn::variable_id::{VariableId, VariableIdResolver};
use crate::throw_type_error;
use either::Either;
use pyo3::{FromPyObject, PyResult};
use std::fmt::{Display, Formatter};

/// `BoolType = Union[bool, int]`
pub mod bool_type;
/// `ParameterIdType = Union[ParameterId, str]`
pub mod parameter_id_type;
/// `Union[Mapping[VariableIdType, Optional[bool]], PerturbationModel]
pub mod perturbation_type;
/// Combines `IdRegulation` and `NamedRegulation` into one type.
pub mod regulation;
/// `Union[IdRegulation, NamedRegulation, str]`
pub mod regulation_type;
/// `SignType = Literal['+', '-', 'positive', 'negative']`
pub mod sign_type;
/// `Union[Mapping[VariableIdType, BoolType], VertexModel]`
pub mod subspace_valuation_type;
/// `Union[UpdateFunction, str]`
pub mod update_function_type;
/// `Union[VariableIdType, Sequence[VariableIdType]]`
pub mod variable_id_multiple_type;
/// `Union[VariableId, BddVariable, str]`
pub mod variable_id_sym_type;
/// `VariableIdType = Union[VariableId, str]`
pub mod variable_id_type;

/// `Union[VariableId, ParameterId, str]`
#[derive(FromPyObject, Debug, Clone)]
pub enum VariableOrParameterIdType {
    Variable(VariableId),
    Parameter(ParameterId),
    Name(String),
}

impl Display for VariableOrParameterIdType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            VariableOrParameterIdType::Name(name) => write!(f, "{}", name),
            VariableOrParameterIdType::Variable(id) => write!(f, "{}", id.__str__()),
            VariableOrParameterIdType::Parameter(id) => write!(f, "{}", id.__str__()),
        }
    }
}

impl VariableOrParameterIdType {
    pub fn resolve<T: VariableIdResolver + ParameterIdResolver>(
        &self,
        resolver: &T,
    ) -> PyResult<Either<biodivine_lib_param_bn::VariableId, biodivine_lib_param_bn::ParameterId>>
    {
        let id = match self {
            VariableOrParameterIdType::Variable(id) => {
                VariableIdResolver::resolve_id(resolver, id.__index__()).map(Either::Left)
            }
            VariableOrParameterIdType::Parameter(id) => {
                ParameterIdResolver::resolve_id(resolver, id.__index__()).map(Either::Right)
            }
            VariableOrParameterIdType::Name(name) => {
                VariableIdResolver::resolve_name(resolver, name)
                    .map(Either::Left)
                    .or_else(|| {
                        ParameterIdResolver::resolve_name(resolver, name).map(Either::Right)
                    })
            }
        };
        if let Some(id) = id {
            Ok(id)
        } else {
            throw_type_error(format!("Variable/Parameter {self} is not valid."))
        }
    }
}
