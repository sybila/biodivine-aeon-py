use pyo3::{
    pyclass, pymethods, types::PyAnyMethods, Bound, IntoPyObjectExt, Py, PyAny, PyResult, Python,
};
use serde::{Deserialize, Serialize};

use crate::throw_runtime_error;

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
#[pyclass(module = "biodivine_aeon")]
pub struct BbmFilterConfig {
    pub min_variables: Option<u32>,
    pub max_variables: Option<u32>,
    pub min_inputs: Option<u32>,
    pub max_inputs: Option<u32>,
    pub min_regulations: Option<u32>,
    pub max_regulations: Option<u32>,
    pub keywords: Vec<String>,
}

#[pymethods]
impl BbmFilterConfig {
    /// Create a new, empty configuration.
    #[new]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a configuration value by key (Python-style indexing).
    pub fn __getitem__(&self, key: &str) -> PyResult<Option<Py<PyAny>>> {
        Python::with_gil(|py| match key {
            "min_variables" => Ok(self.min_variables.map(|v| v.into_py_any(py).unwrap())),
            "max_variables" => Ok(self.max_variables.map(|v| v.into_py_any(py).unwrap())),
            "min_inputs" => Ok(self.min_inputs.map(|v| v.into_py_any(py).unwrap())),
            "max_inputs" => Ok(self.max_inputs.map(|v| v.into_py_any(py).unwrap())),
            "min_regulations" => Ok(self.min_regulations.map(|v| v.into_py_any(py).unwrap())),
            "max_regulations" => Ok(self.max_regulations.map(|v| v.into_py_any(py).unwrap())),
            "keywords" => Ok(Some(self.keywords.clone().into_py_any(py).unwrap())),
            _ => throw_runtime_error(format!("Invalid key: {}", key)),
        })
    }

    /// Set a configuration value by key (Python-style indexing).
    pub fn __setitem__(&mut self, key: &str, value: &Bound<'_, PyAny>) -> PyResult<()> {
        match key {
            "min_variables" => {
                self.min_variables = value.extract::<u32>().ok();
            }
            "max_variables" => {
                self.max_variables = value.extract::<u32>().ok();
            }
            "min_inputs" => {
                self.min_inputs = value.extract::<u32>().ok();
            }
            "max_inputs" => {
                self.max_inputs = value.extract::<u32>().ok();
            }
            "min_regulations" => {
                self.min_regulations = value.extract::<u32>().ok();
            }
            "max_regulations" => {
                self.max_regulations = value.extract::<u32>().ok();
            }
            "keywords" => {
                if let Ok(list) = value.extract::<Vec<String>>() {
                    self.keywords = list;
                } else {
                    return Err(pyo3::exceptions::PyValueError::new_err(
                        "Expected a list of strings for 'keywords'",
                    ));
                }
            }
            _ => {
                return Err(pyo3::exceptions::PyKeyError::new_err(format!(
                    "Invalid key: {}",
                    key
                )));
            }
        }
        Ok(())
    }

    fn __str__(&self) -> String {
        format!(
            "BbmFilterConfig(variables=<{},{}>, inputs=<{},{}>, regulations=<{},{}>, keywords={:?})",
            self.min_variables
                .map_or("None".to_string(), |v| v.to_string()),
            self.max_variables
                .map_or("None".to_string(), |v| v.to_string()),
            self.min_inputs
                .map_or("None".to_string(), |v| v.to_string()),
            self.max_inputs
                .map_or("None".to_string(), |v| v.to_string()),
            self.min_regulations
                .map_or("None".to_string(), |v| v.to_string()),
            self.max_regulations
                .map_or("None".to_string(), |v| v.to_string()),
            self.keywords,
        )
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }

    fn __copy__(&self) -> BbmFilterConfig {
        self.clone()
    }

    fn __deepcopy__(&self, _memo: &Bound<'_, PyAny>) -> BbmFilterConfig {
        self.__copy__()
    }

    /// Load the configuration from a JSON file.
    #[staticmethod]
    pub fn from_json_file(path: &str) -> Result<Self, std::io::Error> {
        let file_content = std::fs::read_to_string(path)?;
        Self::from_json(&file_content)
    }

    /// Load the configuration from a JSON string.
    #[staticmethod]
    pub fn from_json(json_str: &str) -> Result<Self, std::io::Error> {
        let config: BbmFilterConfig = serde_json::from_str(json_str)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(config)
    }
}
