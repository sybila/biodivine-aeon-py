use crate::bindings::lib_param_bn::argument_types::regulation::{
    IdRegulationTuple, NamedRegulationTuple, Regulation,
};
use crate::bindings::lib_param_bn::argument_types::sign_type::SignType;
use crate::bindings::lib_param_bn::argument_types::variable_id_type::VariableIdType;
use crate::bindings::lib_param_bn::variable_id::{VariableIdResolvable, VariableIdResolver};
use crate::throw_runtime_error;
use biodivine_lib_param_bn::Regulation as RegulationNative;
use pyo3::{FromPyObject, PyResult};

#[derive(FromPyObject)]
pub enum RegulationType {
    Regulation(Regulation),
    String(String),
}

impl RegulationType {
    pub fn resolve_named<R: VariableIdResolver>(
        &self,
        resolver: &R,
    ) -> PyResult<NamedRegulationTuple> {
        match self {
            RegulationType::Regulation(regulation) => regulation.resolve_named(resolver),
            RegulationType::String(string) => Self::resolve_raw_regulation_string(string.as_str()),
        }
    }

    pub fn resolve_id<R: VariableIdResolver>(&self, resolver: &R) -> PyResult<IdRegulationTuple> {
        match self {
            RegulationType::Regulation(regulation) => regulation.resolve_id(resolver),
            RegulationType::String(string) => {
                let (source, sign, essential, target) =
                    Self::resolve_raw_regulation_string(string.as_str())?;
                let source = VariableIdType::Name(source).resolve(resolver)?;
                let target = VariableIdType::Name(target).resolve(resolver)?;
                Ok((source, sign, essential, target))
            }
        }
    }

    /// Special resolution method that only accepts names, not IDs.
    pub fn resolve_no_context(&self) -> PyResult<NamedRegulationTuple> {
        match self {
            RegulationType::Regulation(regulation) => regulation.resolve_no_context(),
            RegulationType::String(value) => Self::resolve_raw_regulation_string(value.as_str()),
        }
    }

    pub fn resolve_raw_regulation_string(value: &str) -> PyResult<NamedRegulationTuple> {
        let Some((source, sign, essential, target)) = RegulationNative::try_from_string(value)
        else {
            return throw_runtime_error(format!("Invalid regulation string: `{value}`."));
        };
        let sign = sign.map(|it| SignType::from(it).sign());
        Ok((source, sign, essential, target))
    }
}
