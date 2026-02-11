use pyo3::prelude::PyAnyMethods;
use pyo3::{Bound, Py, PyAny, PyErr, PyResult, Python, pyclass, pymethods};
use serde::de::Deserializer;
use serde::{Deserialize, Serialize};
use std::ffi::CString;

use crate::bindings::lib_param_bn::boolean_network::BooleanNetwork;
use crate::{AsNative, runtime_error};

use super::sampling_utils::pick_random_instances;

/// A representation for model data provided by the Biodivine Boolean Models
/// database endpoint. It is used to deserialize the model data from the JSON
/// response.
///
/// The `raw_model_data` field is a string representation of the Boolean network
/// in AEON format. The rest of the fields are metadata about the model.
#[pyclass(module = "biodivine_aeon")]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BbmModel {
    /// A unique identifier.
    #[pyo3(get)]
    #[serde(deserialize_with = "deserialize_string_or_int")]
    pub id: String,
    /// A descriptive name of the model.
    #[pyo3(get)]
    pub name: String,
    /// URL to the paper where the model was published.
    #[pyo3(get)]
    pub url_publication: String,
    /// URL to the actual model data (if it exists).
    #[pyo3(get)]
    pub url_model: Vec<String>,
    /// List of keywords. See the BBM repository for the full list.
    #[pyo3(get)]
    pub keywords: Vec<String>,
    /// The number of BN variables (excluding inputs).
    #[pyo3(get)]
    pub variables: u32,
    /// The number of inputs (unspecified constant variables).
    #[pyo3(get)]
    pub inputs: u32,
    /// The number of network regulations.
    #[pyo3(get)]
    pub regulations: u32,
    /// Some additional notes.
    #[pyo3(get)]
    pub notes: Option<String>,
    /// A bibliographic entry.
    #[pyo3(get)]
    pub bib: Option<String>,
}

#[pymethods]
impl BbmModel {
    pub fn __str__(&self) -> String {
        format!(
            "BbmModel(id={}, name={}, variables={}, inputs={}, regulations={})",
            self.id, self.name, self.variables, self.inputs, self.regulations
        )
    }

    pub fn __repr__(&self) -> String {
        format!(
            "BbmModel(id={}, name={}, variables={}, inputs={}, regulations={})",
            self.id, self.name, self.variables, self.inputs, self.regulations,
        )
    }

    pub fn __copy__(&self) -> BbmModel {
        self.clone()
    }

    pub fn __deepcopy__(&self, _memo: &Bound<'_, PyAny>) -> BbmModel {
        self.__copy__()
    }

    /// Extract a `BooleanNetwork` instance from this model.
    /// Leave the inputs as they are (free if not set in the model).
    pub fn to_bn_default(&self, py: Python) -> PyResult<Py<BooleanNetwork>> {
        Self::fetch_model_data(self.id.as_str(), py)
    }

    /// Extract a `BooleanNetwork` instance from this model, setting all inputs
    /// to a given constant value `const_value`.
    fn to_bn_inputs_const(&self, py: Python, const_value: bool) -> PyResult<Py<BooleanNetwork>> {
        let py_bn = Self::fetch_model_data(self.id.as_str(), py)?;
        let mut bn = py_bn.borrow_mut(py).clone();
        for variable in bn.as_native().inputs(true) {
            let update_fn = if const_value {
                biodivine_lib_param_bn::FnUpdate::mk_true()
            } else {
                biodivine_lib_param_bn::FnUpdate::mk_false()
            };
            bn.as_native_mut()
                .set_update_function(variable, Some(update_fn))
                .map_err(runtime_error)?;
        }
        bn.export_to_python(py)
    }

    /// Extract a `BooleanNetwork` instance from this model, setting all inputs
    /// to `true`.
    pub fn to_bn_inputs_true(&self, py: Python) -> PyResult<Py<BooleanNetwork>> {
        self.to_bn_inputs_const(py, true)
    }

    /// Extract a `BooleanNetwork` instance from this model, setting all inputs
    /// to `false`.
    pub fn to_bn_inputs_false(&self, py: Python) -> PyResult<Py<BooleanNetwork>> {
        self.to_bn_inputs_const(py, false)
    }

    /// Extract a given number of unique `BooleanNetwork` instances from this model,
    /// setting all input values randomly.
    #[pyo3(signature = (instance_count=1, random_seed=42))]
    pub fn to_bn_inputs_random(
        &self,
        py: Python,
        instance_count: usize,
        random_seed: u64,
    ) -> PyResult<Vec<Py<BooleanNetwork>>> {
        let py_bn_inputs_free = self.to_bn_default(py)?;
        let bn = py_bn_inputs_free.borrow_mut(py).clone();
        let instantiated_bns: Vec<Py<BooleanNetwork>> =
            pick_random_instances(bn.as_native(), instance_count, random_seed)
                .map_err(runtime_error)?
                .into_iter()
                .map(|bn: biodivine_lib_param_bn::BooleanNetwork| {
                    BooleanNetwork::from(bn).export_to_python(py)
                })
                .collect::<Result<Vec<Py<BooleanNetwork>>, PyErr>>()
                .map_err(runtime_error)?;
        Ok(instantiated_bns)
    }
}

impl BbmModel {
    pub fn fetch_model_data(id: &str, py: Python) -> PyResult<Py<BooleanNetwork>> {
        let url = format!("https://bbm.sybila.fi.muni.cz/api/models/{id}/aeon");
        let code =
            format!("__import__('urllib.request').request.urlopen('{url}').read().decode('utf-8')");
        let code = CString::new(code)?;
        let result = py.eval(code.as_c_str(), None, None)?;
        let result = result.extract::<String>()?;
        BooleanNetwork::from_aeon(py, result.as_str())
    }
}

fn deserialize_string_or_int<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value: serde_json::Value = Deserialize::deserialize(deserializer)?;

    match value {
        serde_json::Value::Number(n) => Ok(n.to_string()),
        serde_json::Value::String(s) => Ok(s),
        _ => Err(serde::de::Error::custom("Expected a string or a number")),
    }
}
