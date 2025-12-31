use crate::throw_type_error;
use pyo3::types::PyBool;
use pyo3::{Borrowed, FromPyObject, IntoPyObject, PyAny, PyErr, Python};
use std::convert::Infallible;

/// This is a utility wrapper for `bool` which implements `FromPyObject` and `ToPyObject` in a
/// way that is idiomatic for AEON: The object is a boolean value, but it can be automatically
/// extracted from an integer, assuming it is `0` or `1`.
///
/// Importantly, the implementation provides correct type hints and is therefore compatible
/// with other API/documentation tools.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct BoolType(bool);

impl BoolType {
    pub fn bool(self) -> bool {
        self.0
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for BoolType {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        if let Ok(v) = obj.extract::<bool>() {
            return Ok(BoolType(v));
        }
        if let Ok(v) = obj.extract::<usize>() {
            match v {
                0 => return Ok(BoolType(false)),
                1 => return Ok(BoolType(true)),
                _ => (),
            }
        }

        throw_type_error(format!(
            "Expected `True`/`False` or `1`/`0`. Got `{obj:?}`."
        ))
    }
}

impl<'py> IntoPyObject<'py> for BoolType {
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

impl From<bool> for BoolType {
    fn from(value: bool) -> Self {
        BoolType(value)
    }
}

impl From<BoolType> for bool {
    fn from(value: BoolType) -> Self {
        value.0
    }
}
