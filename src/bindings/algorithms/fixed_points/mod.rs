use pyo3::{
    Bound, PyResult,
    types::{PyModule, PyModuleMethods as _},
};

mod _impl_pyerr;
mod fixed_points_config_python;
mod fixed_points_impl_python;

pub use fixed_points_config_python::PyFixedPointsConfig;
use fixed_points_impl_python::PyFixedPoints;

pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyFixedPoints>()?;
    module.add_class::<PyFixedPointsConfig>()?;

    Ok(())
}
