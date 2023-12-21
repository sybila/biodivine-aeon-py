use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use macros::Wrapper;
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::types::PyTuple;

/// A numeric identifier of a single decision variable used within a `Bdd`.
/// 
/// It essentially behaves like a type-safe integer value:
/// ```python
/// a = BddVariable(0)
/// b = BddVariable(1)
/// assert a == eval(repr(a))
/// assert a != b
/// assert a < b
/// assert a <= a
/// assert str(a) == "x_0"
/// assert int(a) == 0
/// d = {a: True, b: False}
/// assert d[a] != d[b]
/// ```
///
/// The value of `BddVariable` are frozen (i.e. immutable).
///
/// See also `BddVariableType`.
///  
#[pyclass(module = "biodivine_aeon", frozen)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Wrapper)]
pub struct BddVariable(biodivine_lib_bdd::BddVariable);

#[pymethods]
impl BddVariable {
    
    #[new]
    #[pyo3(signature = (value = 0))]
    pub fn new(value: usize) -> BddVariable {
        BddVariable(biodivine_lib_bdd::BddVariable::from_index(value))
    }

    pub fn __richcmp__(&self, other: &Self, op: CompareOp) -> bool {
        op.matches(self.cmp(other))
    }

    pub fn __str__(&self) -> String {
        format!("x_{}", self.0)
    }

    pub fn __repr__(&self) -> String {
        format!("BddVariable({})", self.0)
    }

    pub fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    pub fn __index__(&self) -> usize {
        self.0.to_index()
    }

    pub fn __getnewargs__<'a>(&self, py: Python<'a>) -> &'a PyTuple {
        PyTuple::new(py, &[self.0.to_index()])
    }

}