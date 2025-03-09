use pyo3::{types::PyModule, Bound, PyResult};

pub mod cancellation;
pub mod reachability;

pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    reachability::register(module)?;
    Ok(())
}
