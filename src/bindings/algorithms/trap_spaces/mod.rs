use pyo3::{
    types::{PyModule, PyModuleMethods},
    Bound, PyResult,
};
use trap_spaces_config::PyTrapSpacesConfig;
use trap_spaces_impl::PyTrapSpaces;

mod _impl_pyerr;
mod _impl_symbolic_space_context;
pub mod trap_spaces_config;
pub mod trap_spaces_error;
mod trap_spaces_impl;

pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyTrapSpaces>()?;
    module.add_class::<PyTrapSpacesConfig>()?;
    Ok(())
}
