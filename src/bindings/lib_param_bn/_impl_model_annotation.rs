use crate::bindings::lib_param_bn::PyModelAnnotation;
use crate::{throw_runtime_error, AsNative};
use biodivine_lib_param_bn::ModelAnnotation;
use pyo3::{pymethods, PyResult};
use std::collections::HashMap;
use std::mem::swap;

impl Default for PyModelAnnotation {
    fn default() -> Self {
        PyModelAnnotation::new(None)
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
    #[pyo3(signature = (value = None))]
    pub fn new(value: Option<String>) -> PyModelAnnotation {
        if let Some(value) = value {
            ModelAnnotation::with_value(value).into()
        } else {
            ModelAnnotation::new().into()
        }
    }

    #[staticmethod]
    pub fn from_model_string(data: &str) -> PyModelAnnotation {
        ModelAnnotation::from_model_string(data).into()
    }

    #[staticmethod]
    pub fn from_model_path(path: &str) -> PyModelAnnotation {
        Self::from_model_string(std::fs::read_to_string(path).unwrap().as_str())
    }

    pub fn get_value(&self, path: Vec<&str>) -> Option<String> {
        self.as_native().get_value(&path).cloned()
    }

    pub fn ensure_value(&mut self, path: Vec<&str>, value: &str) -> bool {
        self.as_native_mut().ensure_value(&path, value)
    }

    pub fn clear_value(&mut self, path: Vec<&str>) -> Option<String> {
        // This can be simplified once a fix to `get_mut_child` is published.
        fn clear_recursive(annotation: &mut ModelAnnotation, path: &[&str]) -> Option<String> {
            if path.is_empty() {
                let value = annotation.value_mut();
                let mut dropped: Option<String> = None;
                swap(value, &mut dropped);
                dropped
            } else {
                let child = path[0];
                if let Some(child) = annotation.children_mut().get_mut(child) {
                    clear_recursive(child, &path[1..])
                } else {
                    None
                }
            }
        }
        clear_recursive(self.as_native_mut(), &path)
    }

    pub fn append_value(&mut self, path: Vec<&str>, value: &str) {
        self.as_native_mut().append_value(&path, value)
    }

    pub fn clone_child(&self, path: Vec<&str>) -> Option<PyModelAnnotation> {
        self.as_native()
            .get_child(&path)
            .map(|it| it.clone().into())
    }

    pub fn ensure_child(&mut self, path: Vec<&str>, child: PyModelAnnotation) -> bool {
        let slot = self.as_native_mut().ensure_child(&path);
        let annotation: ModelAnnotation = child.into();
        if annotation != *slot {
            *slot = annotation;
            true
        } else {
            false
        }
    }

    pub fn clear_child(&mut self, path: Vec<&str>) -> PyResult<Option<PyModelAnnotation>> {
        if path.is_empty() {
            return throw_runtime_error("Cannot erase annotation root.");
        }

        // This can be simplified once a fix to `get_mut_child` is published.
        fn clear_recursive(
            annotation: &mut ModelAnnotation,
            path: &[&str],
        ) -> Option<ModelAnnotation> {
            if path.len() == 1 {
                let to_clear = path[0];
                annotation.children_mut().remove(to_clear)
            } else {
                let child = path[0];
                if let Some(child) = annotation.children_mut().get_mut(child) {
                    clear_recursive(child, &path[1..])
                } else {
                    None
                }
            }
        }

        Ok(clear_recursive(self.as_native_mut(), &path).map(|it| it.into()))
    }

    pub fn value(&self) -> Option<String> {
        self.as_native().value().cloned()
    }

    pub fn clone_children(&self) -> HashMap<String, PyModelAnnotation> {
        self.as_native()
            .children()
            .iter()
            .map(|(k, v)| (k.clone(), PyModelAnnotation::from(v.clone())))
            .collect()
    }
}
