use macros::Wrapper;
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::types::PyTuple;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// A numeric identifier of a single *network variable* used within a `BooleanNetwork`
/// or a `RegulatoryGraph`.
///
/// It essentially behaves like a type-safe integer value:
/// ```python
/// a = VariableId(0)
/// b = VariableId(1)
/// assert a == eval(repr(a))
/// assert a != b
/// assert a < b
/// assert a <= a
/// assert str(a) == "v_0"
/// assert int(a) == 0
/// d = {a: True, b: False}
/// assert d[a] != d[b]
/// ```
///
/// The value of `VariableId` is frozen (i.e. immutable).
///
/// See also `VariableIdType`: In most cases where the ID can be "inferred from context",
/// a name can be also used to identify a network variable.
///  
#[pyclass(module = "biodivine_aeon", frozen)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Wrapper)]
pub struct VariableId(biodivine_lib_param_bn::VariableId);

#[pymethods]
impl VariableId {
    #[new]
    #[pyo3(signature = (value = 0))]
    pub fn new(value: usize) -> Self {
        Self(biodivine_lib_param_bn::VariableId::from_index(value))
    }

    pub fn __richcmp__(&self, other: &Self, op: CompareOp) -> bool {
        op.matches(self.cmp(other))
    }

    pub fn __str__(&self) -> String {
        format!("v_{}", self.0.to_index())
    }

    pub fn __repr__(&self) -> String {
        format!("VariableId({})", self.0.to_index())
    }

    pub fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    pub fn __index__(&self) -> usize {
        self.0.to_index()
    }

    pub fn __getnewargs__<'a>(&self, py: Python<'a>) -> PyResult<Bound<'a, PyTuple>> {
        PyTuple::new(py, [self.0.to_index()])
    }
}
