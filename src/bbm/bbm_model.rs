use pyo3::{Bound, Py, PyAny, PyErr, PyResult, Python, pyclass, pymethods};
use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize};

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
    #[pyo3(get)]
    pub id: String,
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub url_publication: Option<String>,
    #[pyo3(get)]
    pub url_model: Option<String>,
    #[pyo3(get)]
    pub keywords: Vec<String>,
    #[pyo3(get)]
    pub variables: u32,
    #[pyo3(get)]
    pub inputs: u32,
    #[pyo3(get)]
    pub regulations: u32,
    #[pyo3(get)]
    pub notes: Option<String>,
    #[pyo3(get)]
    pub bib: Option<String>,
    #[pyo3(get)]
    #[serde(deserialize_with = "deserialize_model_data", rename = "modelData")]
    pub raw_model_data: String, // Deserialize directly into a String
}

/// Custom deserialization function for the `modelData` field.
/// Converts the raw byte data from the JSON response into a UTF-8 string.
fn deserialize_model_data<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct RawModelData {
        #[serde(rename = "type")]
        _data_type: String,
        data: Vec<u8>,
    }

    let raw: RawModelData = RawModelData::deserialize(deserializer)?;
    String::from_utf8(raw.data)
        .map_err(|e| de::Error::custom(format!("Failed to convert model data to string: {}", e)))
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
            "BbmModel(id={}, name={}, variables={}, inputs={}, regulations={}, network={:?})",
            self.id, self.name, self.variables, self.inputs, self.regulations, self.raw_model_data,
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
        BooleanNetwork::from_aeon(py, &self.raw_model_data).map_err(runtime_error)
    }

    /// Extract a `BooleanNetwork` instance from this model, setting all inputs
    /// to a given constant value `const_value`.
    fn to_bn_inputs_const(&self, py: Python, const_value: bool) -> PyResult<Py<BooleanNetwork>> {
        let py_bn = BooleanNetwork::from_aeon(py, &self.raw_model_data).map_err(runtime_error)?;
        let mut bn = py_bn.borrow_mut(py).clone();
        for variable in bn.as_native().inputs(const_value) {
            let update_fn = biodivine_lib_param_bn::FnUpdate::mk_true();
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
