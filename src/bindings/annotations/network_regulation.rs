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
    pub fn get_references(self_: PyRef<'_, Self>, py: Python) -> Option<Vec<String>> {
        self_.as_ref().__getitem__(REFERENCE).get_lines(py)
    }

    #[setter]
    pub fn set_references(self_: PyRef<'_, Self>, py: Python, references: Option<Vec<String>>) {
        self_
            .as_ref()
            .__getitem__(REFERENCE)
            .set_lines(py, references);
    }
}

impl NetworkRegulationAnnotation {
    /// Extend an annotation object that represents a regulation with the data from another
    /// regulation annotation. For now, this only includes references.
    pub fn extend_with(main: &mut NativeAnnotation, sub: &NativeAnnotation) {
        if let Some(data) = sub.get_value(&[REFERENCE]) {
            main.append_value(&[REFERENCE], format!("\n{}", data).as_str());
        }
    }
}
