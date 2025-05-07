use pyo3::{
    types::{PyModule, PyModuleMethods},
    Bound, PyResult,
};

mod _impl_pyerr;
mod _impl_symbolic_space_context;
mod trap_spaces_config;
mod trap_spaces_config_python;
mod trap_spaces_error;
mod trap_spaces_impl;
mod trap_spaces_impl_python;

pub use _impl_symbolic_space_context::SymbolicSpaceContextExt;
pub use trap_spaces_config::TrapSpacesConfig;
pub use trap_spaces_error::TrapSpacesError;
pub use trap_spaces_impl::TrapSpaces;

pub use trap_spaces_config_python::PyTrapSpacesConfig;
use trap_spaces_impl_python::PyTrapSpaces;

pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyTrapSpaces>()?;
    module.add_class::<PyTrapSpacesConfig>()?;

    Ok(())
}
