use pyo3::{types::PyModule, Bound, PyResult};

pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    Ok(())
}
