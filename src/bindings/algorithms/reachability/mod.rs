use pyo3::{
    types::{PyModule, PyModuleMethods as _},
    Bound, PyResult,
};

use crate::internal::algorithms::reachability::{Reachability, ReachabilityConfig};

mod _impl_pyerr;
mod reachability_config_python;
mod reachablility_impl_python;

pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<Reachability>()?;
    module.add_class::<ReachabilityConfig>()?;

    Ok(())
}
