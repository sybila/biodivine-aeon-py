use crate::bindings::annotations::REFERENCE;
use crate::bindings::lib_param_bn::model_annotation::ModelAnnotation;
use biodivine_lib_param_bn::ModelAnnotation as NativeAnnotation;
use pyo3::{pyclass, pymethods, PyRef, Python};

/// Typed annotation data associated with a network regulation.
#[pyclass(module="biodivine_aeon", extends=ModelAnnotation)]
pub struct NetworkRegulationAnnotation();

#[pymethods]
impl NetworkRegulationAnnotation {
    #[getter]
    /// Returns the list of reference lines associated with this network regulation annotation, if present.
    ///
    /// # Examples
    ///
    /// ```
    /// # use biodivine_aeon::bindings::annotations::network_regulation::NetworkRegulationAnnotation;
    /// # use pyo3::Python;
    /// Python::with_gil(|py| {
    ///     let annotation = NetworkRegulationAnnotation::new();
    ///     assert!(annotation.get_references(py).is_none());
    /// });
    /// ```
    pub fn get_references(self_: PyRef<'_, Self>, py: Python) -> Option<Vec<String>> {
        self_.as_ref().__getitem__(REFERENCE).get_lines(py)
    }

    #[setter]
    /// Sets the reference lines for this annotation.
    ///
    /// Updates the `REFERENCE` field of the annotation with the provided lines, replacing any existing references.
    ///
    /// # Examples
    ///
    /// ```
    /// annotation.set_references(py, Some(vec!["Ref1".to_string(), "Ref2".to_string()]));
    /// ```
    pub fn set_references(self_: PyRef<'_, Self>, py: Python, references: Option<Vec<String>>) {
        self_
            .as_ref()
            .__getitem__(REFERENCE)
            .set_lines(py, references);
    }
}

impl NetworkRegulationAnnotation {
    /// Extend an annotation object that represents a regulation with the data from another
    /// ```    pub fn extend_with(main: &mut NativeAnnotation, sub: &NativeAnnotation) {
        if let Some(data) = sub.get_value(&[REFERENCE]) {
            main.append_value(&[REFERENCE], format!("\n{}", data).as_str());
        }
    }
}
