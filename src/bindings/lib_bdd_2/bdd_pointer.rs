use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use macros::Wrapper;
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::types::PyTuple;
use crate::throw_type_error;

/// A numeric identifier of a single node in a `Bdd`.
///
/// It essentially behaves like a type-safe integer value with some extra utility methods:
///
/// ```python
/// x = BddPointer()  # Default should be zero.
/// y = BddPointer(True)
/// z = BddPointer(10)
/// assert z == eval(repr(z))
/// assert x != z
/// assert x < y < z
/// assert str(x) == "node_0"
/// assert int(x) == 0
/// assert x == BddPointer.zero()
/// assert y == BddPointer.one()
/// assert (x.as_bool() is not None) and not x.as_bool()
/// assert (y.as_bool() is not None) and y.as_bool()
/// assert z.as_bool() is None
/// assert x.is_terminal() and x.is_zero()
/// assert y.is_terminal() and y.is_one()
/// assert not (z.is_terminal() or z.is_one() or z.is_zero())
/// ```
///
/// The value of `BddPointer` are frozen (i.e. immutable).
///
#[pyclass(module = "biodivine_aeon", frozen)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Wrapper)]
pub struct BddPointer(biodivine_lib_bdd::BddPointer);

#[pymethods]
impl BddPointer {

    #[new]
    #[pyo3(signature = (value = None))]
    fn new(value: Option<&PyAny>) -> PyResult<BddPointer> {
        let Some(value) = value else {
            return Ok(Self::zero());
        };

        if let Ok(value) = value.extract::<usize>() {
            return Ok(biodivine_lib_bdd::BddPointer::from_index(value).into());
        }
        if let Ok(value) = value.extract::<bool>() {
            return Ok(biodivine_lib_bdd::BddPointer::from_bool(value).into());
        }

        throw_type_error("Expected `int` or `bool`.")
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> bool {
        op.matches(self.cmp(other))
    }

    fn __str__(&self) -> String {
        format!("node_{}", self.0)
    }

    fn __repr__(&self) -> String {
        format!("BddPointer({})", self.0)
    }

    fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    fn __index__(&self) -> usize {
        self.0.to_index()
    }


    fn __getnewargs__<'a>(&self, py: Python<'a>) -> &'a PyTuple {
        PyTuple::new(py, &[self.0.to_index()])
    }

    /// Returns the `BddPointer` referencing the `0` terminal node.
    #[staticmethod]
    fn zero() -> BddPointer {
        biodivine_lib_bdd::BddPointer::zero().into()
    }

    /// Returns the `BddPointer` referencing the `1` terminal node.
    #[staticmethod]
    fn one() -> BddPointer {
        biodivine_lib_bdd::BddPointer::one().into()
    }

    /// Returns `True` if this `BddPointer` refers to the `0` terminal node.
    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    /// Returns `True` if this `BddPointer` refers to the `1` terminal node.
    fn is_one(&self) -> bool {
        self.0.is_one()
    }

    /// Returns `True` if this `BddPointer` refers to the `0` or `1` terminal node.
    fn is_terminal(&self) -> bool {
        self.0.is_terminal()
    }

    /// Try to convert this `BddPointer` to `bool` (if it is terminal),
    /// or `None` if it represents a decision node.
    fn as_bool(&self) -> Option<bool> {
        self.0.as_bool()
    }

}