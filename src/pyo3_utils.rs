use crate::throw_type_error;
use biodivine_lib_param_bn::{Monotonicity, Sign};
use pyo3::basic::CompareOp;
use pyo3::prelude::PyAnyMethods;
use pyo3::types::{PyBool, PyString};
use pyo3::{Bound, FromPyObject, IntoPy, Py, PyAny, PyObject, PyResult, Python, ToPyObject};

/// Compare the equality of two objects based on the given `cmp` operator and a "key" returned by the `key` function
/// for each of the objects.
pub fn richcmp_eq_by_key<T: Sized, R: Eq>(
    py: Python,
    cmp: CompareOp,
    x: &T,
    y: &T,
    key: fn(&T) -> R,
) -> Py<PyAny> {
    match cmp {
        CompareOp::Eq => key(x).eq(&key(y)).into_py(py),
        CompareOp::Ne => key(x).ne(&key(y)).into_py(py),
        _ => py.NotImplemented(),
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
    pub fn bool(&self) -> bool {
        self.0
    }
}

impl<'py> FromPyObject<'py> for BoolLikeValue {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        if let Ok(v) = ob.extract::<bool>() {
            return Ok(BoolLikeValue(v));
        }
        if let Ok(v) = ob.extract::<usize>() {
            match v {
                0 => return Ok(BoolLikeValue(false)),
                1 => return Ok(BoolLikeValue(true)),
                _ => (),
            }
        }

        throw_type_error(format!("Expected `True`/`False` or `1`/`0`. Got `{}`.", ob))
    }
}

impl ToPyObject for BoolLikeValue {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        PyBool::new_bound(py, self.0).to_object(py)
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

impl<'py> FromPyObject<'py> for SignValue {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        if let Ok(v) = ob.extract::<bool>() {
            return match v {
                true => Ok(SignValue(Sign::Positive)),
                false => Ok(SignValue(Sign::Negative)),
            };
        }
        if let Ok(v) = ob.extract::<String>() {
            // Activation and inhibition are included for backwards compatibility.
            match v.as_str() {
                "positive" | "+" | "activation" => return Ok(SignValue(Sign::Positive)),
                "negative" | "-" | "inhibition" => return Ok(SignValue(Sign::Negative)),
                _ => (),
            };
        }

        throw_type_error(format!(
            "Expected one of `positive`/`negative`/`+`/`-`. Got `{}`.",
            ob
        ))
    }
}

impl ToPyObject for SignValue {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        let str_value = match self.0 {
            Sign::Positive => "+",
            Sign::Negative => "-",
        };
        PyString::new_bound(py, str_value).to_object(py)
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

pub fn resolve_sign(value: &Bound<'_, PyAny>) -> PyResult<Sign> {
    if let Ok(value) = value.extract::<bool>() {
        return if value {
            Ok(Sign::Positive)
        } else {
            Ok(Sign::Negative)
        };
    }
    if let Ok(value) = value.extract::<String>() {
        // Activation and inhibition are included for backwards compatibility.
        return match value.as_str() {
            "positive" | "+" | "activation" => Ok(Sign::Positive),
            "negative" | "-" | "inhibition" => Ok(Sign::Negative),
            _ => throw_type_error("Expected one of `positive`/`negative`/`+`/`-`."),
        };
    }
    throw_type_error("Expected `positive`/`negative`/`+`/`-` or `bool`.")
}
