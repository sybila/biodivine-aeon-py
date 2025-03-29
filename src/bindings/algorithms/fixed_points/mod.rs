use fixed_points_config::FixedPointsConfigPython;
use fixed_points_impl::FixedPointsPython;
use pyo3::{
    types::{PyModule, PyModuleMethods},
    Bound, PyResult,
};

mod _impl_pyerr;
mod fixed_points_config;
mod fixed_points_error;
mod fixed_points_impl;

pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<FixedPointsConfigPython>()?;
    module.add_class::<FixedPointsPython>()?;
    Ok(())
}
