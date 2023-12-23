use crate::{throw_runtime_error, throw_type_error};
use pyo3::basic::CompareOp;
use pyo3::{PyAny, PyResult};

pub fn richcmp_eq_inner<T: Sized, R: Eq>(
    cmp: CompareOp,
    x: &T,
    y: &T,
    inner: fn(&T) -> R,
) -> PyResult<bool> {
    match cmp {
        CompareOp::Eq => Ok(inner(x).eq(&inner(y))),
        CompareOp::Ne => Ok(inner(x).ne(&inner(y))),
        _ => throw_runtime_error(format!(
            "`{}` cannot be ordered.",
            std::any::type_name::<T>()
        )),
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
