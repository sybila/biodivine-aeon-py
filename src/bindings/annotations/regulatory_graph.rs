use crate::bindings::annotations::network_regulation::NetworkRegulationAnnotation;
use crate::bindings::annotations::network_variable::NetworkVariableAnnotation;
use crate::bindings::annotations::REFERENCE;
use crate::bindings::lib_param_bn::model_annotation::ModelAnnotation;
use crate::bindings::lib_param_bn::regulatory_graph::RegulatoryGraph;
use crate::throw_runtime_error;
use pyo3::{pyclass, pymethods, Bound, Py, PyAny, PyRef, PyResult, Python};

pub const VARIABLE: &str = "variable";
pub const REGULATION: &str = "regulation";
const TAXON: &str = "taxon";
const NAME: &str = "name";
const DESCRIPTION: &str = "description";

/// An extension of `ModelAnnotation` that provides type-safe access to all annotation data
/// that is officially supported by AEON as annotations of a `RegulatoryGraph`.
///
/// Note that you can still access any "raw" annotation data the same way you would on
/// any instance of `ModelAnnotation`.
#[pyclass(module="biodivine_aeon", extends=ModelAnnotation)]
pub struct RegulatoryGraphAnnotation(Py<RegulatoryGraph>);

#[pymethods]
impl RegulatoryGraphAnnotation {
    /// Get a reference to `NetworkVariableAnnotation` for the given variable.
    ///
    /// Note that this function fails if such variable does not exist in
    /// the associated `RegulatoryGraph`.
    pub fn variable(
        self_: PyRef<'_, Self>,
        py: Python,
        variable: &Bound<'_, PyAny>,
    ) -> PyResult<Py<NetworkVariableAnnotation>> {
        let name = self_.0.borrow(py).get_variable_name(variable)?;
        let variable_annotations = self_.as_ref().__getitem__(VARIABLE);
        let variable_data = variable_annotations.__getitem__(name.as_str());
        let tuple = (NetworkVariableAnnotation(), variable_data);
        Py::new(py, tuple)
    }

    /// Get a reference to `NetworkRegulationAnnotation` for the given pair of variables.
    ///
    /// Note that this function fails if such regulation does not exist
    /// in the associated `RegulatoryGraph`.
    pub fn regulation(
        self_: PyRef<'_, Self>,
        py: Python,
        source: &Bound<'_, PyAny>,
        target: &Bound<'_, PyAny>,
    ) -> PyResult<Py<NetworkRegulationAnnotation>> {
        let source_name = self_.0.borrow(py).get_variable_name(source)?;
        let target_name = self_.0.borrow(py).get_variable_name(target)?;
        let regulation = self_.0.borrow(py).find_regulation(py, source, target)?;
        if regulation.is_none() {
            return throw_runtime_error("Regulation does not exist.");
        }
        let regulation_annotations = self_.as_ref().__getitem__(REGULATION);
        let source_data = regulation_annotations.__getitem__(source_name.as_str());
        let target_data = source_data.__getitem__(target_name.as_str());
        let tuple = (NetworkRegulationAnnotation(), target_data);
        Py::new(py, tuple)
    }

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

    #[getter]
    pub fn get_taxon(self_: PyRef<'_, Self>, py: Python) -> Option<String> {
        self_.as_ref().__getitem__(TAXON).get_value(py)
    }

    #[setter]
    pub fn set_taxon(self_: PyRef<'_, Self>, py: Python, taxon: Option<String>) {
        self_.as_ref().__getitem__(TAXON).set_value(py, taxon);
    }

    #[getter]
    pub fn get_name(self_: PyRef<'_, Self>, py: Python) -> Option<String> {
        self_.as_ref().__getitem__(NAME).get_value(py)
    }

    #[setter]
    pub fn set_name(self_: PyRef<'_, Self>, py: Python, name: Option<String>) {
        self_.as_ref().__getitem__(NAME).set_value(py, name);
    }

    #[getter]
    pub fn get_description(self_: PyRef<'_, Self>, py: Python) -> Option<String> {
        self_.as_ref().__getitem__(DESCRIPTION).get_value(py)
    }

    #[setter]
    pub fn set_description(self_: PyRef<'_, Self>, py: Python, description: Option<String>) {
        self_
            .as_ref()
            .__getitem__(DESCRIPTION)
            .set_value(py, description);
    }
}

impl From<Py<RegulatoryGraph>> for RegulatoryGraphAnnotation {
    fn from(value: Py<RegulatoryGraph>) -> Self {
        RegulatoryGraphAnnotation(value)
    }
}
