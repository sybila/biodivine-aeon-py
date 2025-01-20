use pyo3::{
    types::{PyModule, PyModuleMethods},
    Bound, PyResult,
};
use reachibility_config::ReachabilityConfig;

// mod reachibility;
mod reachibility_config;
mod reachibility_error;

pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<ReachabilityConfig>()?;
    Ok(())
}
