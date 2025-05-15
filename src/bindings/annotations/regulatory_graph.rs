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
    /// Returns the annotation for a given variable in the associated `RegulatoryGraph`.
    ///
    /// Retrieves a `NetworkVariableAnnotation` for the specified variable by name.  
    /// Fails if the variable does not exist in the graph.
    ///
    /// # Examples
    ///
    /// ```python
    /// # Python usage example
    /// annotation = graph_annotation.variable(variable)
    /// assert isinstance(annotation, NetworkVariableAnnotation)
    /// ```    pub fn variable(
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
    /// Retrieves the annotation for a regulation between two variables in the associated `RegulatoryGraph`.
    ///
    /// Returns a `NetworkRegulationAnnotation` for the specified source and target variables if the regulation exists, or raises a runtime error if it does not.
    ///
    /// # Examples
    ///
    /// ```python
    /// # Python usage example (assuming `reg_ann` is a RegulatoryGraphAnnotation instance)
    /// ann = reg_ann.regulation(source_var, target_var)
    /// ```    pub fn regulation(
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
    /// Returns the list of reference annotation lines, if present.
    ///
    /// # Examples
    ///
    /// ```
    /// let refs = annotation.get_references(py);
    /// if let Some(lines) = refs {
    ///     assert!(lines.iter().all(|line| line.is_ascii()));
    /// }
    /// ```
    pub fn get_references(self_: PyRef<'_, Self>, py: Python) -> Option<Vec<String>> {
        self_.as_ref().__getitem__(REFERENCE).get_lines(py)
    }

    #[setter]
    /// Sets the reference annotation lines for the regulatory graph.
    ///
    /// Updates the "reference" annotation entry with the provided list of reference strings, or clears it if `None` is given.
    ///
    /// # Examples
    ///
    /// ```
    /// annotation.set_references(py, Some(vec!["PMID:12345".to_string(), "PMID:67890".to_string()]));
    /// ```
    pub fn set_references(self_: PyRef<'_, Self>, py: Python, references: Option<Vec<String>>) {
        self_
            .as_ref()
            .__getitem__(REFERENCE)
            .set_lines(py, references);
    }

    #[getter]
    /// Returns the taxon annotation value, if present.
    ///
    /// # Examples
    ///
    /// ```
    /// let annotation = RegulatoryGraphAnnotation::from(reg_graph);
    /// let taxon = annotation.get_taxon(py);
    /// assert!(taxon.is_none() || taxon.is_some());
    /// ```
    pub fn get_taxon(self_: PyRef<'_, Self>, py: Python) -> Option<String> {
        self_.as_ref().__getitem__(TAXON).get_value(py)
    }

    #[setter]
    /// Sets the "taxon" annotation value for the regulatory graph.
    ///
    /// # Examples
    ///
    /// ```
    /// annotation.set_taxon(py, Some("Homo sapiens".to_string()));
    /// ```
    pub fn set_taxon(self_: PyRef<'_, Self>, py: Python, taxon: Option<String>) {
        self_.as_ref().__getitem__(TAXON).set_value(py, taxon);
    }

    #[getter]
    /// Returns the value of the "name" annotation, if present.
    ///
    /// # Examples
    ///
    /// ```
    /// let annotation = RegulatoryGraphAnnotation::from(reg_graph);
    /// let name = annotation.get_name(py);
    /// assert!(name.is_none() || name.is_some());
    /// ```
    pub fn get_name(self_: PyRef<'_, Self>, py: Python) -> Option<String> {
        self_.as_ref().__getitem__(NAME).get_value(py)
    }

    #[setter]
    /// Sets the "name" annotation value for the regulatory graph.
    ///
    /// # Examples
    ///
    /// ```
    /// annotation.set_name(py, Some("My Regulatory Graph".to_string()));
    /// ```
    pub fn set_name(self_: PyRef<'_, Self>, py: Python, name: Option<String>) {
        self_.as_ref().__getitem__(NAME).set_value(py, name);
    }

    #[getter]
    /// Returns the description annotation for the regulatory graph, if available.
    ///
    /// # Examples
    ///
    /// ```
    /// let annotation = RegulatoryGraphAnnotation::from(graph);
    /// let description = annotation.get_description(py);
    /// assert!(description.is_none() || description.is_some());
    /// ```
    pub fn get_description(self_: PyRef<'_, Self>, py: Python) -> Option<String> {
        self_.as_ref().__getitem__(DESCRIPTION).get_value(py)
    }

    #[setter]
    /// Sets the "description" annotation value for the regulatory graph.
    ///
    /// # Examples
    ///
    /// ```
    /// annotation.set_description(py, Some("Gene regulatory network for E. coli".to_string()));
    /// ```
    pub fn set_description(self_: PyRef<'_, Self>, py: Python, description: Option<String>) {
        self_
            .as_ref()
            .__getitem__(DESCRIPTION)
            .set_value(py, description);
    }
}

impl From<Py<RegulatoryGraph>> for RegulatoryGraphAnnotation {
    /// Creates a `RegulatoryGraphAnnotation` from a Python `RegulatoryGraph` reference.
    ///
    /// # Examples
    ///
    /// ```
    /// let py_graph: Py<RegulatoryGraph> = ...; // obtained from Python
    /// let annotation = RegulatoryGraphAnnotation::from(py_graph);
    /// ```
    fn from(value: Py<RegulatoryGraph>) -> Self {
        RegulatoryGraphAnnotation(value)
    }
}
