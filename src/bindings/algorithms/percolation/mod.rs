use pyo3::{
    types::{PyModule, PyModuleMethods},
    Bound, PyResult,
};

mod _impl_pyerr;
mod percolation_config;
mod percolation_config_python;
mod percolation_error;
mod percolation_impl;
mod percolation_impl_python;
mod subspace_representation;

pub use percolation_config::PercolationConfig;
pub use percolation_error::PercolationError;
pub use percolation_impl::Percolation;
pub use subspace_representation::SubspaceRepresentation;

pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<Percolation>()?;
    module.add_class::<PercolationConfig>()?;

    Ok(())
}
