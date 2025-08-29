use pyo3::{
    Bound, PyResult,
    types::{PyModule, PyModuleMethods as _},
};

mod _impl_pyerr;
mod percolation_config_python;
mod percolation_impl_python;
mod subspace_representation;

pub use subspace_representation::SubspaceRepresentation;

use crate::internal::algorithms::percolation::{Percolation, PercolationConfig};

pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<Percolation>()?;
    module.add_class::<PercolationConfig>()?;

    Ok(())
}
