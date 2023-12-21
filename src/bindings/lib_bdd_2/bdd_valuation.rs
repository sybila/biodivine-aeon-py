use pyo3::prelude::*;
use crate::AsNative;
use crate::bindings::lib_bdd_2::bdd_variable_set::BddVariableSet;

/// Represents a *complete* valuation of all variables in a `Bdd`.
///
/// This can be seen as a safer and slightly more efficient `Dict[BddVariable, bool]` where
/// we enforce a continuous range of `BddVariable` keys.
#[pyclass(module = "biodivine_aeon")]
#[derive(Clone)]
pub struct BddValuation {
    ctx: Py<BddVariableSet>,
    value: biodivine_lib_bdd::BddValuation,
}

#[pymethods]
impl BddValuation {

    #[new]
    fn new(py: Python, ctx: Py<BddVariableSet>, values: Vec<bool>) -> BddValuation {
        unimplemented!()
    }

    fn __getnewargs__<'a>(&self, py: Python<'a>) -> (Py<BddVariableSet>, Vec<bool>) {
        unimplemented!()
    }

}

impl AsNative<biodivine_lib_bdd::BddValuation> for BddValuation {
    fn as_native(&self) -> &biodivine_lib_bdd::BddValuation {
        &self.value
    }

    fn as_native_mut(&mut self) -> &mut biodivine_lib_bdd::BddValuation {
        &mut self.value
    }
}

#[pyclass(module = "biodivine_aeon")]
#[derive(Clone)]
pub struct BddPartialValuation {
    ctx: Py<BddVariableSet>,
    value: biodivine_lib_bdd::BddPartialValuation,
}

impl AsNative<biodivine_lib_bdd::BddPartialValuation> for BddPartialValuation {
    fn as_native(&self) -> &biodivine_lib_bdd::BddPartialValuation {
        &self.value
    }

    fn as_native_mut(&mut self) -> &mut biodivine_lib_bdd::BddPartialValuation {
        &mut self.value
    }
}