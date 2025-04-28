use pyo3::{
    types::{PyModule, PyModuleMethods},
    Bound, PyResult,
};

use percolation_config::PercolationConfig;
use percolation_impl::Percolation;
use subspace_representation::SubspaceRepresentation;

mod _impl_pyerr;
pub mod percolation_config;
pub mod percolation_error;
mod percolation_impl;
mod subspace_representation;

pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<Percolation>()?;
    module.add_class::<PercolationConfig>()?;
    module.add_class::<SubspaceRepresentation>()?;

    Ok(())
}
