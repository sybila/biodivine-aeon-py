use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use crate::{AsNative, throw_runtime_error};
use crate::bindings::lib_bdd_2::bdd_variable_set::BddVariableSet;


#[pyclass(module = "biodivine_aeon", frozen)]
#[derive(Clone)]
pub struct Bdd {
    ctx: Py<BddVariableSet>,
    value: biodivine_lib_bdd::Bdd,
}

#[pymethods]
impl Bdd {

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self.value.eq(&other.value)),
            CompareOp::Ne => Ok(self.value.ne(&other.value)),
            _ => throw_runtime_error("`BddVariableSet` cannot be ordered."),
        }
    }

}

impl AsNative<biodivine_lib_bdd::Bdd> for Bdd {
    fn as_native(&self) -> &biodivine_lib_bdd::Bdd {
        &self.value
    }

    fn as_native_mut(&mut self) -> &mut biodivine_lib_bdd::Bdd {
        &mut self.value
    }
}

impl Bdd {

    pub fn new(ctx: PyRef<'_, BddVariableSet>, value: biodivine_lib_bdd::Bdd) -> Bdd {
        Bdd {
            ctx: ctx.into(),
            value
        }
    }

}