use pyo3::{
    Bound, PyResult,
    types::{PyModule, PyModuleMethods as _},
};

mod _impl_pyerr;
mod trap_spaces_config_python;
mod trap_spaces_impl_python;

pub use trap_spaces_config_python::PyTrapSpacesConfig;
use trap_spaces_impl_python::PyTrapSpaces;

pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyTrapSpaces>()?;
    module.add_class::<PyTrapSpacesConfig>()?;

    Ok(())
}
