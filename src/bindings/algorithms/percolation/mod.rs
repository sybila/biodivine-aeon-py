use pyo3::{
    types::{PyModule, PyModuleMethods},
    Bound, PyResult,
};

use percolation_config::PercolationConfig;
use percolation_impl::Percolation;

mod _impl_pyerr;
mod percolation_config;
mod percolation_error;
mod percolation_impl;

pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<Percolation>()?;
    module.add_class::<PercolationConfig>()?;
    Ok(())
}
