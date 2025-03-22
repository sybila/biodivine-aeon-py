use pyo3::{types::PyModule, Bound, PyResult};

pub mod cancellation;
pub mod fixed_points;
pub mod reachability;

pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    pyo3_log::init();
    reachability::register(module)?;
    fixed_points::register(module)?;
    Ok(())
}
