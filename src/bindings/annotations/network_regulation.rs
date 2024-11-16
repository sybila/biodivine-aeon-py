use crate::bindings::annotations::REFERENCE;
use crate::bindings::lib_param_bn::model_annotation::ModelAnnotation;
use pyo3::{pyclass, pymethods, PyRef, Python};

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
