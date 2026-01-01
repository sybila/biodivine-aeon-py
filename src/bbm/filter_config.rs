use pyo3::prelude::PyAnyMethods;
use pyo3::types::PyDict;
use pyo3::{Borrowed, FromPyObject, PyAny, PyErr, types::PyDictMethods};
use serde::{Deserialize, Serialize};

/// A configuration structure for filtering Biodivine Boolean Models (BBM).
/// This structure allows users to specify the following filtering criteria:
/// - `min_variables`: Minimum number of variables in the model.
/// - `max_variables`: Maximum number of variables in the model.
/// - `min_inputs`: Minimum number of inputs in the model.
/// - `max_inputs`: Maximum number of inputs in the model.
/// - `min_regulations`: Minimum number of regulations in the model.
/// - `max_regulations`: Maximum number of regulations in the model.
/// - `keywords`: A list of required keywords to filter the models (all will be required).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct BbmFilterConfig {
    pub min_variables: Option<u32>,
    pub max_variables: Option<u32>,
    pub min_inputs: Option<u32>,
    pub max_inputs: Option<u32>,
    pub min_regulations: Option<u32>,
    pub max_regulations: Option<u32>,
    pub keywords: Option<Vec<String>>,
}

impl<'a, 'py> FromPyObject<'a, 'py> for BbmFilterConfig {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, 'py, PyAny>) -> Result<Self, Self::Error> {
        let obj = obj.cast::<PyDict>()?;

        // This is a bit cumbersome, but for now we can't really derive `FromPyObject` if all
        // items have a default "none" value.
        let min_variables: Option<u32> = obj
            .get_item("min_variables")?
            .map(|it| it.extract())
            .transpose()?;

        let max_variables: Option<u32> = obj
            .get_item("max_variables")?
            .map(|it| it.extract())
            .transpose()?;

        let min_inputs: Option<u32> = obj
            .get_item("min_inputs")?
            .map(|it| it.extract())
            .transpose()?;

        let max_inputs: Option<u32> = obj
            .get_item("max_inputs")?
            .map(|it| it.extract())
            .transpose()?;

        let min_regulations: Option<u32> = obj
            .get_item("min_regulations")?
            .map(|it| it.extract())
            .transpose()?;

        let max_regulations: Option<u32> = obj
            .get_item("max_regulations")?
            .map(|it| it.extract())
            .transpose()?;

        let keywords: Option<Vec<String>> = obj
            .get_item("keywords")?
            .map(|it| it.extract())
            .transpose()?;

        Ok(BbmFilterConfig {
            min_variables,
            max_variables,
            min_inputs,
            max_inputs,
            min_regulations,
            max_regulations,
            keywords,
        })
    }
}
