use super::bbm_model::BbmModel;
use super::filter_config::BbmFilterConfig;
use crate::bindings::lib_param_bn::boolean_network::BooleanNetwork;
use crate::runtime_error;
use pyo3::ffi::c_str;
use pyo3::prelude::PyAnyMethods;
use pyo3::{Py, PyResult, Python, pyclass, pymethods};
use regex::Regex;
use serde_json::Value;
use std::ffi::CString;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};

thread_local! {
    /// Used for retrieving IDs from model references.
    static ID_REGEX: Regex = Regex::new(r"bbm-(\d{3})").unwrap();
    /// Remember if the dataset has been downloaded already.
    static INITIALIZED: AtomicBool = const { AtomicBool::new(false) };
}

/// The API does not return the model ID right now, but the ID does appear in the bib entry.
/// We can extract it from there!
fn extract_id(input: &Option<String>) -> Option<usize> {
    let input = input.as_ref()?;

    ID_REGEX.with(|regex| {
        if let Some(captures) = regex.captures(input) {
            let digits_str = &captures[1];
            return usize::from_str(digits_str).ok();
        }
        None
    })
}

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
    /// Fetch the JSON data from the BBM API endpoint, process it, and return a list of
    /// all models. The returned models can then be used for further filtering or
    /// processing.
    #[staticmethod]
    #[pyo3(signature = (url=None))]
    fn fetch_all_model_data(py: Python, url: Option<String>) -> PyResult<Vec<BbmModel>> {
        // The advantage of this approach is that it correctly responds to Python cancellation.
        let url = url.unwrap_or_else(|| "https://bbm.sybila.fi.muni.cz/api/models/".to_string());

        // If the data is not initialized yet, fetch it and save it to a global Python variable.
        INITIALIZED.with(|initialized| {
            if !initialized.load(Ordering::SeqCst) {
                let code = format!("_AEON_PY_MODELS_CACHE = __import__('urllib.request').request.urlopen('{url}').read().decode('utf-8')");
                let code = CString::new(code).unwrap();
                let result = py.run(code.as_c_str(), None, None);
                initialized.store(true, Ordering::SeqCst);
                result
            } else {
                Ok(())
            }
        })?;

        // Retrieve downloaded data from the global Python variable
        let text = py
            .eval(c_str!("_AEON_PY_MODELS_CACHE"), None, None)?
            .extract::<String>()?;

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

    /// Retrieve a Boolean network model from the Biodivine Boolean Models database
    /// using the provided model ID. You can use either the "classical" BBM model numbers
    /// (`015`, `077, etc.) or dedicated unique ID strings used in the model database.
    #[staticmethod]
    #[pyo3(signature = (id, inline_inputs=false))]
    pub fn fetch_network(
        py: Python,
        id: &str,
        inline_inputs: bool,
    ) -> PyResult<Py<BooleanNetwork>> {
        // MAX is hopefully never going to be a valid ID
        let num_id = id.parse::<usize>().unwrap_or(usize::MAX);
        let models_list = Self::fetch_all_model_data(py, None)?;
        let model = models_list
            .into_iter()
            .find(|m| m.id == id || extract_id(&m.bib).is_some_and(|it| it == num_id))
            .ok_or(runtime_error("Model not found in BBM database."))?;

        let py_bn = BooleanNetwork::from_aeon(py, &model.raw_model_data).map_err(runtime_error)?;

        let py_bn = match inline_inputs {
            true => py_bn.borrow_mut(py).inline_inputs(py, true, true)?,
            false => py_bn,
        };
        Ok(py_bn)
    }

    /// Retrieve a model from the Biodivine Boolean Models database using the
    /// provided model ID. You can use either the "classical" BBM model numbers
    /// (`015`, `077, etc.) or dedicated unique ID strings used in the model database.
    #[staticmethod]
    pub fn fetch_model(py: Python, id: &str) -> PyResult<BbmModel> {
        let num_id = id.parse::<usize>().unwrap_or(usize::MAX);
        let models_list = Self::fetch_all_model_data(py, None)?;
        // The numerical ID is not given in the model right now, but the models are
        // listed in-order, so
        let model = models_list
            .into_iter()
            .find(|m| m.id == id || extract_id(&m.bib).is_some_and(|it| it == num_id))
            .ok_or(runtime_error("Model not found in BBM database."))?;

        Ok(model)
    }

    /// Fetch a list of IDs of BBM database models that satisfy given conditions.
    /// These IDs can be used to retrieve the models using the `fetch_network` method.
    ///
    /// See the [BbmFilterConfig] class for how to prepare the filtering options.
    /// If no filtering config is provided, all model IDs are retrieved. This is
    /// the same as providing an empty config.
    ///
    /// At the moment, the model ID is a string ID used by the endpoint, which
    /// is different from the numerical id classically used in the BBM database.
    /// This is a temporary solution until the BBM endpoint is updated to provide
    /// the numerical id as well.
    #[staticmethod]
    #[pyo3(signature = (config=None))]
    pub fn fetch_ids(py: Python, config: Option<BbmFilterConfig>) -> PyResult<Vec<String>> {
        let models_list = Self::fetch_all_model_data(py, None)?;

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
                        && cfg
                            .keywords
                            .as_ref()
                            .is_none_or(|v| v.iter().all(|k| m.keywords.contains(k)))
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
