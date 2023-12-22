use crate::throw_type_error;
use pyo3::{PyAny, PyResult};

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
    throw_type_error("Expected `True`/`False` or `1`/`0`.")
}
