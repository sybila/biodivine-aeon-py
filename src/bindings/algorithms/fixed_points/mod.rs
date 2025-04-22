use fixed_points_config::PyFixedPointsConfig;
use fixed_points_impl::PyFixedPoints;
use pyo3::{
    types::{PyModule, PyModuleMethods},
    Bound, PyResult,
};

mod _impl_pyerr;
pub mod fixed_points_config;
pub mod fixed_points_error;
pub mod fixed_points_impl;

pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyFixedPointsConfig>()?;
    module.add_class::<PyFixedPoints>()?;
    Ok(())
}
