use crate::throw_type_error;
use pyo3::basic::CompareOp;
use pyo3::{IntoPy, Py, PyAny, PyResult, Python};

pub fn richcmp_eq_inner<T: Sized, R: Eq>(
    py: Python,
    cmp: CompareOp,
    x: &T,
    y: &T,
    inner: fn(&T) -> R,
) -> Py<PyAny> {
    match cmp {
        CompareOp::Eq => inner(x).eq(&inner(y)).into_py(py),
        CompareOp::Ne => inner(x).ne(&inner(y)).into_py(py),
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
    throw_type_error("Expected `True`/`False` or `1`/`0`.")
}
