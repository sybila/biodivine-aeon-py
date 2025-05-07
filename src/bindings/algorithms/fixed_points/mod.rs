use pyo3::{
    types::{PyModule, PyModuleMethods},
    Bound, PyResult,
};

mod _impl_pyerr;
mod fixed_points_config;
mod fixed_points_config_python;
mod fixed_points_error;
mod fixed_points_impl;
mod fixed_points_impl_python;

pub use fixed_points_config::FixedPointsConfig;
pub use fixed_points_error::FixedPointsError;
pub use fixed_points_impl::FixedPoints;

pub use fixed_points_config_python::PyFixedPointsConfig;
use fixed_points_impl_python::PyFixedPoints;

pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyFixedPoints>()?;
    module.add_class::<PyFixedPointsConfig>()?;

    Ok(())
}
