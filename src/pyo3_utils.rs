use pyo3::basic::CompareOp;
use pyo3::{IntoPyObjectExt, Py, PyAny, PyResult, Python};

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
