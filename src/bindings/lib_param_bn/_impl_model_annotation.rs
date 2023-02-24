use crate::bindings::lib_param_bn::PyModelAnnotation;
use crate::AsNative;
use biodivine_lib_param_bn::ModelAnnotation;
use pyo3::{pymethods, PyResult};

impl Default for PyModelAnnotation {
    fn default() -> Self {
        PyModelAnnotation::new()
    }
}

#[pymethods]
impl PyModelAnnotation {
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{}", self.as_native()))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self.as_native()))
    }

    fn __getattr__(&self, index: &str) -> Option<PyModelAnnotation> {
        Some(self.as_native().get_child(&[index]).unwrap().clone().into())
    }

    #[new]
    pub fn new() -> PyModelAnnotation {
        ModelAnnotation::new().into()
    }

    #[staticmethod]
    pub fn from_model_string(data: &str) -> PyModelAnnotation {
        ModelAnnotation::from_model_string(data).into()
    }

    #[staticmethod]
    pub fn from_model_path(path: &str) -> PyModelAnnotation {
        Self::from_model_string(std::fs::read_to_string(path).unwrap().as_str())
    }
}
