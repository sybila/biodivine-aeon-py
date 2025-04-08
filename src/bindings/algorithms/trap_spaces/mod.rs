use pyo3::{
    types::{PyModule, PyModuleMethods},
    Bound, PyResult,
};
use trap_spaces_config::TrapSpacesConfig;
use trap_spaces_impl::TrapSpaces;

mod _impl_pyerr;
mod trap_spaces_config;
mod trap_spaces_error;
mod trap_spaces_impl;

pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<TrapSpaces>()?;
    module.add_class::<TrapSpacesConfig>()?;
    Ok(())
}
