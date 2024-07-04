use pyo3::prelude::{PyModule, PyModuleMethods};
use pyo3::{Bound, PyResult};

pub mod hctl_formula;
pub mod model_checking;

pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<hctl_formula::HctlFormula>()?;
    module.add_class::<model_checking::ModelChecking>()?;
    Ok(())
}
