use crate::bindings::lib_param_bn::argument_types::bool_type::BoolType;
use crate::bindings::lib_param_bn::argument_types::sign_type::SignType;
use crate::bindings::lib_param_bn::argument_types::variable_id_type::VariableIdType;
use crate::bindings::lib_param_bn::variable_id::{
    VariableId, VariableIdResolvable, VariableIdResolver,
};
use biodivine_lib_param_bn::Sign;
use biodivine_lib_param_bn::VariableId as VariableIdNative;
use pyo3::prelude::PyDictMethods;
use pyo3::types::PyDict;
use pyo3::{Bound, FromPyObject, IntoPyObject, PyErr, PyResult, Python};

/// (source, monotonicity, essential, target)
pub type NamedRegulationTuple = (String, Option<Sign>, bool, String);

pub type IdRegulationTuple = (VariableIdNative, Option<Sign>, bool, VariableIdNative);

/// Maps to both `IdRegulation` and `NamedRegulation`, including redundant
/// keys for backwards compatibility.
#[derive(FromPyObject)]
pub struct Regulation {
    #[pyo3(item)]
    pub source: VariableIdType,
    #[pyo3(item)]
    pub target: VariableIdType,
    #[pyo3(item, default = None)]
    pub sign: Option<SignType>,
    #[pyo3(item, default = None)]
    pub monotonicity: Option<SignType>,
    #[pyo3(item, default = None)]
    pub essential: Option<BoolType>,
    #[pyo3(item, default = None)]
    pub observable: Option<BoolType>,
}

/// A dedicated type that can be converted back into a python dictionary (converting the whole
/// `Regulation` is tedious due to the extra fields).
pub struct RegulationOutput {
    pub source: VariableId,
    pub target: VariableId,
    pub sign: Option<SignType>,
    pub essential: Option<bool>,
}

impl Regulation {
    pub fn resolve_id<R: VariableIdResolver>(&self, resolver: &R) -> PyResult<IdRegulationTuple> {
        let source = self.source.resolve(resolver)?;
        let target = self.target.resolve(resolver)?;
        let sign = self.sign.or(self.monotonicity).map(|it| it.sign());
        let essential: Option<bool> = self.essential.or(self.observable).map(|it| it.into());
        Ok((source, sign, essential.unwrap_or(true), target))
    }

    pub fn resolve_named<R: VariableIdResolver>(
        &self,
        resolver: &R,
    ) -> PyResult<NamedRegulationTuple> {
        let (source, sign, essential, target) = self.resolve_id(resolver)?;
        let source_name = resolver.get_name(source);
        let target_name = resolver.get_name(target);
        Ok((source_name, sign, essential, target_name))
    }

    pub fn resolve_no_context(&self) -> PyResult<NamedRegulationTuple> {
        let source = self.source.resolve_no_context()?;
        let target = self.target.resolve_no_context()?;
        let sign = self.sign.or(self.monotonicity).map(|it| it.sign());
        let essential: Option<bool> = self.essential.or(self.observable).map(|it| it.into());
        Ok((source, sign, essential.unwrap_or(true), target))
    }
}

impl From<IdRegulationTuple> for RegulationOutput {
    fn from(value: IdRegulationTuple) -> Self {
        RegulationOutput {
            source: VariableId::from(value.0),
            target: VariableId::from(value.3),
            sign: value.1.map(SignType::from),
            essential: Some(value.2),
        }
    }
}

impl From<&biodivine_lib_param_bn::Regulation> for RegulationOutput {
    fn from(value: &biodivine_lib_param_bn::Regulation) -> Self {
        RegulationOutput {
            source: VariableId::from(value.regulator),
            target: VariableId::from(value.target),
            sign: value.monotonicity.map(SignType::from),
            essential: Some(value.observable),
        }
    }
}

impl<'py> IntoPyObject<'py> for RegulationOutput {
    type Target = PyDict;
    type Output = Bound<'py, PyDict>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let dict = PyDict::new(py);
        dict.set_item("source", self.source)?;
        dict.set_item("target", self.target)?;
        dict.set_item("sign", self.sign)?;
        dict.set_item("essential", self.essential)?;
        Ok(dict)
    }
}
