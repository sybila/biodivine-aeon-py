use pyo3::{
    types::{PyModule, PyModuleMethods},
    Bound, PyResult,
};
use reachability::Reachability;
use reachability_config::ReachabilityConfig;

mod _impl_pyerr;
mod cancellation;
mod cancellation_error;
mod cancellation_handler;
mod reachability;
mod reachability_config;
mod reachability_config_py;
mod reachability_error;

pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<ReachabilityConfig>()?;
    module.add_class::<Reachability>()?;
    Ok(())
}
