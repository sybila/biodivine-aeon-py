use macros::Wrapper;
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::types::PyTuple;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// A numeric identifier of a single *explicit parameter* used within a `BooleanNetwork`.
///
/// It essentially behaves like a type-safe integer value:
/// ```python
/// a = ParameterId(0)
/// b = ParameterId(1)
/// assert a == eval(repr(a))
/// assert a != b
/// assert a < b
/// assert a <= a
/// assert str(a) == "p_0"
/// assert int(a) == 0
/// d = {a: True, b: False}
/// assert d[a] != d[b]
/// ```
///
/// The value of `ParameterIdId` is frozen (i.e. immutable).
///
/// See also `ParameterIdType`.
///
#[pyclass(module = "biodivine_aeon", frozen)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Wrapper)]
pub struct ParameterId(biodivine_lib_param_bn::ParameterId);

#[pymethods]
impl ParameterId {
    #[new]
    #[pyo3(signature = (value = 0))]
    pub fn new(value: usize) -> Self {
        Self(biodivine_lib_param_bn::ParameterId::from_index(value))
    }

    pub fn __richcmp__(&self, other: &Self, op: CompareOp) -> bool {
        op.matches(self.cmp(other))
    }

    pub fn __str__(&self) -> String {
        format!("p_{}", self.0.to_index())
    }

    pub fn __repr__(&self) -> String {
        format!("ParameterId({})", self.0.to_index())
    }

    pub fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    pub fn __index__(&self) -> usize {
        self.0.to_index()
    }

    pub fn __getnewargs__<'a>(&self, py: Python<'a>) -> Bound<'a, PyTuple> {
        PyTuple::new_bound(py, [self.0.to_index()])
    }
}
