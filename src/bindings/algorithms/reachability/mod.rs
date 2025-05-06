use pyo3::{
    types::{PyModule, PyModuleMethods},
    Bound, PyResult,
};

mod _impl_pyerr;
mod reachability_config;
mod reachability_config_python;
mod reachability_error;
mod reachability_impl;

pub use reachability_config::ReachabilityConfig;
pub use reachability_error::ReachabilityError;
pub use reachability_impl::Reachability;

pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<Reachability>()?;
    module.add_class::<ReachabilityConfig>()?;

    Ok(())
}
