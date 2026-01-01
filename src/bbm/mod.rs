mod bbm_model;
mod biodivine_boolean_models;
mod filter_config;
mod sampling_utils;

use pyo3::prelude::{PyModule, PyModuleMethods};
use pyo3::{Bound, PyResult};

pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<biodivine_boolean_models::BiodivineBooleanModels>()?;
    module.add_class::<bbm_model::BbmModel>()?;

    Ok(())
}
