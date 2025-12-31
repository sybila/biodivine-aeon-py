use crate::throw_type_error;
use biodivine_lib_param_bn::{Monotonicity, Sign};
use pyo3::types::PyString;
use pyo3::{Borrowed, Bound, FromPyObject, IntoPyObject, PyAny, PyErr, Python};
use std::convert::Infallible;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SignType(Sign);

impl SignType {
    pub fn sign(self) -> Sign {
        Sign::from(self)
    }

    pub fn monotonicity(self) -> Monotonicity {
        Monotonicity::from(self)
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for SignType {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        if let Ok(v) = obj.extract::<bool>() {
            return match v {
                true => Ok(SignType(Sign::Positive)),
                false => Ok(SignType(Sign::Negative)),
            };
        }
        if let Ok(v) = obj.extract::<String>() {
            // Activation and inhibition are included for backwards compatibility.
            match v.as_str() {
                "positive" | "+" | "activation" => return Ok(SignType(Sign::Positive)),
                "negative" | "-" | "inhibition" => return Ok(SignType(Sign::Negative)),
                _ => (),
            };
        }

        throw_type_error(format!(
            "Expected one of `positive`/`negative`/`+`/`-`. Got `{obj:?}`."
        ))
    }
}

impl<'py> IntoPyObject<'py> for SignType {
    type Target = PyString;
    type Output = Bound<'py, PyString>;
    type Error = Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        match self.0 {
            Sign::Positive => "+",
            Sign::Negative => "-",
        }
        .into_pyobject(py)
    }
}

impl From<SignType> for Sign {
    fn from(value: SignType) -> Self {
        value.0
    }
}

impl From<SignType> for Monotonicity {
    fn from(value: SignType) -> Self {
        match value.0 {
            Sign::Positive => Monotonicity::Activation,
            Sign::Negative => Monotonicity::Inhibition,
        }
    }
}

impl From<Sign> for SignType {
    fn from(value: Sign) -> Self {
        SignType(value)
    }
}

impl From<Monotonicity> for SignType {
    fn from(value: Monotonicity) -> Self {
        match value {
            Monotonicity::Activation => SignType(Sign::Positive),
            Monotonicity::Inhibition => SignType(Sign::Negative),
        }
    }
}
