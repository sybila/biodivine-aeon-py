mod bbm_model;
mod biodivine_boolean_models;

use pyo3::prelude::{PyModule, PyModuleMethods};
use pyo3::{Bound, PyResult};

pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<biodivine_boolean_models::BiodivineBooleanModels>()?;
    module.add_class::<bbm_model::BbmModel>()?;

    Ok(())
}
