use crate::bindings::lib_param_bn::argument_types::variable_id_type::VariableIdType;
use pyo3::FromPyObject;

#[derive(FromPyObject, Debug, Clone)]
pub enum VariableIdMultipleType {
    One(VariableIdType),
    Many(Vec<VariableIdType>),
}

impl From<VariableIdMultipleType> for Vec<VariableIdType> {
    fn from(value: VariableIdMultipleType) -> Self {
        match value {
            VariableIdMultipleType::One(one) => vec![one],
            VariableIdMultipleType::Many(many) => many,
        }
    }
}
