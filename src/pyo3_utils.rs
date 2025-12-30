use crate::throw_type_error;
use biodivine_lib_param_bn::{Monotonicity, Sign};
use pyo3::basic::CompareOp;
use pyo3::types::{PyBool, PyString};
use pyo3::{
    Borrowed, Bound, FromPyObject, IntoPyObject, IntoPyObjectExt, Py, PyAny, PyErr, PyResult,
    Python,
};
use std::convert::Infallible;

/// Compare the equality of two objects based on the given `cmp` operator and a "key" returned by the `key` function
/// for each of the objects.
pub fn richcmp_eq_by_key<T: Sized, R: Eq>(
    py: Python,
    cmp: CompareOp,
    x: &T,
    y: &T,
    key: fn(&T) -> R,
) -> PyResult<Py<PyAny>> {
    match cmp {
        CompareOp::Eq => key(x).eq(&key(y)).into_py_any(py),
        CompareOp::Ne => key(x).ne(&key(y)).into_py_any(py),
        _ => Ok(py.NotImplemented()),
    }
}

/// This is a utility wrapper for `bool` which implements `FromPyObject` and `ToPyObject` in a
/// way that is idiomatic for AEON: The object is a boolean value, but it can be automatically
/// extracted from an integer, assuming it is `0` or `1`.
///
/// Importantly, the implementation provides correct type hints and is therefore compatible
/// with other API/documentation tools.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct BoolLikeValue(bool);

impl BoolLikeValue {
    pub fn bool(self) -> bool {
        self.0
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for BoolLikeValue {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        if let Ok(v) = obj.extract::<bool>() {
            return Ok(BoolLikeValue(v));
        }
        if let Ok(v) = obj.extract::<usize>() {
            match v {
                0 => return Ok(BoolLikeValue(false)),
                1 => return Ok(BoolLikeValue(true)),
                _ => (),
            }
        }

        throw_type_error(format!(
            "Expected `True`/`False` or `1`/`0`. Got `{obj:?}`."
        ))
    }
}

impl<'py> IntoPyObject<'py> for BoolLikeValue {
    //fn to_object(&self, py: Python<'_>) -> PyObject {
    //    PyBool::new_bound(py, self.0).to_object(py)
    //}

    type Target = PyBool;
    type Output = Borrowed<'py, 'py, PyBool>;
    type Error = Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(PyBool::new(py, self.0))
    }
}

impl From<bool> for BoolLikeValue {
    fn from(value: bool) -> Self {
        BoolLikeValue(value)
    }
}

impl From<BoolLikeValue> for bool {
    fn from(value: BoolLikeValue) -> Self {
        value.0
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SignValue(Sign);

impl SignValue {
    pub fn sign(self) -> Sign {
        Sign::from(self)
    }

    pub fn monotonicity(self) -> Monotonicity {
        Monotonicity::from(self)
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for SignValue {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        if let Ok(v) = obj.extract::<bool>() {
            return match v {
                true => Ok(SignValue(Sign::Positive)),
                false => Ok(SignValue(Sign::Negative)),
            };
        }
        if let Ok(v) = obj.extract::<String>() {
            // Activation and inhibition are included for backwards compatibility.
            match v.as_str() {
                "positive" | "+" | "activation" => return Ok(SignValue(Sign::Positive)),
                "negative" | "-" | "inhibition" => return Ok(SignValue(Sign::Negative)),
                _ => (),
            };
        }

        throw_type_error(format!(
            "Expected one of `positive`/`negative`/`+`/`-`. Got `{obj:?}`."
        ))
    }
}

impl<'py> IntoPyObject<'py> for SignValue {
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

impl From<SignValue> for Sign {
    fn from(value: SignValue) -> Self {
        value.0
    }
}

impl From<SignValue> for Monotonicity {
    fn from(value: SignValue) -> Self {
        match value.0 {
            Sign::Positive => Monotonicity::Activation,
            Sign::Negative => Monotonicity::Inhibition,
        }
    }
}

impl From<Sign> for SignValue {
    fn from(value: Sign) -> Self {
        SignValue(value)
    }
}

impl From<Monotonicity> for SignValue {
    fn from(value: Monotonicity) -> Self {
        match value {
            Monotonicity::Activation => SignValue(Sign::Positive),
            Monotonicity::Inhibition => SignValue(Sign::Negative),
        }
    }
}
