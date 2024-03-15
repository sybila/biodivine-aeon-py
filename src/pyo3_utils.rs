use crate::throw_type_error;
use biodivine_lib_param_bn::Sign;
use pyo3::basic::CompareOp;
use pyo3::{IntoPy, Py, PyAny, PyResult, Python};

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

pub fn resolve_boolean(value: &PyAny) -> PyResult<bool> {
    if let Ok(value) = value.extract::<bool>() {
        return Ok(value);
    }
    if let Ok(value) = value.extract::<usize>() {
        if value == 0 {
            return Ok(false);
        }
        if value == 1 {
            return Ok(true);
        }
    }
    throw_type_error(format!(
        "Expected `True`/`False` or `1`/`0`. Found `{}`.",
        value.str()?.to_str()?
    ))
}

pub fn resolve_sign(value: &PyAny) -> PyResult<Sign> {
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
