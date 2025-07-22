use pyo3::PyResult;
use pyo3::prelude::*;

mod class;
mod classification;

pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<class::Class>()?;
    module.add_class::<classification::Classification>()?;
    Ok(())
}
