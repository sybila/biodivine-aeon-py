use pyo3::{pyclass, pymethods, Py, PyResult, Python};
use reqwest::blocking::get;
use serde_json::Value;

use crate::bindings::lib_param_bn::boolean_network::BooleanNetwork;
use crate::runtime_error;

use super::bbm_model::BbmModel;
use super::filter_config::BbmFilterConfig;

/// Class to represent the Biodivine Boolean Models database and provide
/// all the functionality that can be performed on it.
///
/// BBM models are fetched from the database using the `fetch_network` method.
#[pyclass(module = "biodivine_aeon", frozen)]
pub struct BiodivineBooleanModels {
    _dummy: (),
}

#[pymethods]
impl BiodivineBooleanModels {
    /// Fetch the JSON data from BBM API endpoint, process it, and return a list of
    /// all models. The returned models can then be used for further filtering or
    /// processing.
    #[staticmethod]
    fn fetch_all_model_data() -> PyResult<Vec<BbmModel>> {
        let url = "https://bbm.sybila.fi.muni.cz/api/models/";
        let response = get(url)
            .map_err(|e| runtime_error(format!("Request to BBM endpoint failed: {}", e)))?;
        let text = response
            .text()
            .map_err(|e| runtime_error(format!("Failed to read the BBM response text: {}", e)))?;

        let json: Value = serde_json::from_str(&text)
            .map_err(|e| runtime_error(format!("Failed to parse JSON: {}", e)))?;
        let data_json = json
            .get("data")
            .ok_or_else(|| runtime_error("No 'data' key in the JSON response."))?
            .clone();

        let models_json: Vec<BbmModel> = serde_json::from_value(data_json)
            .map_err(|e| runtime_error(format!("Failed to deserialize BbmModel: {}", e)))?;
        Ok(models_json)
    }

    /// Fetch a Boolean network from the Biodivine Boolean Models database
    /// using the provided model ID.
    ///
    /// At the moment, the model ID is a string ID used by the endpoint, which
    /// is not the same as the numerical id clasically used in the BBM database.
    /// This is a temporary solution until the BBM endpoint is updated to provide
    /// the numerical id as well.
    #[staticmethod]
    pub fn fetch_network(py: Python, id: &str) -> PyResult<Py<BooleanNetwork>> {
        let models_list = Self::fetch_all_model_data()?;
        let model = models_list
            .into_iter()
            .find(|m| m.id == id)
            .ok_or(runtime_error("Model not found in BBM database."))?;

        let bn = BooleanNetwork::from_aeon(py, &model.model_data).map_err(runtime_error)?;
        Ok(bn)
    }

    /// Fetch a list of IDs of BBM database models that satisfy given conditions.
    /// These IDs can be used to fetch the models using the `fetch_network` method.
    ///
    /// See the [BbmFilterConfig] class for how to prepare the filtering options.
    ///
    /// At the moment, the model ID is a string ID used by the endpoint, which
    /// is not the same as the numerical id clasically used in the BBM database.
    /// This is a temporary solution until the BBM endpoint is updated to provide
    /// the numerical id as well.
    #[staticmethod]
    #[pyo3(signature = (config=None))]
    pub fn fetch_ids(config: Option<BbmFilterConfig>) -> PyResult<Vec<String>> {
        let models_list = Self::fetch_all_model_data()?;

        // Filter the models based on the provided configuration
        let models_list = match config {
            Some(cfg) => models_list
                .into_iter()
                .filter(|m| {
                    cfg.min_variables.is_none_or(|v| m.variables >= v)
                        && cfg.max_variables.is_none_or(|v| m.variables <= v)
                        && cfg.min_inputs.is_none_or(|v| m.inputs >= v)
                        && cfg.max_inputs.is_none_or(|v| m.inputs <= v)
                        && cfg.min_regulations.is_none_or(|v| m.regulations >= v)
                        && cfg.max_regulations.is_none_or(|v| m.regulations <= v)
                        && cfg.keywords.iter().all(|k| m.keywords.contains(k))
                })
                .collect::<Vec<BbmModel>>(),
            None => models_list,
        };

        // Extract the IDs from the filtered models
        let ids = models_list
            .into_iter()
            .map(|m| m.id)
            .collect::<Vec<String>>();
        Ok(ids)
    }
}
