use crate::bindings::annotations::REFERENCE;
use crate::bindings::lib_param_bn::model_annotation::ModelAnnotation;
use biodivine_lib_param_bn::ModelAnnotation as NativeAnnotation;
use pyo3::{pyclass, pymethods, Py, PyRef, PyResult, Python};

const GENE_NAME: &str = "gene_name";
const ID: &str = "id";
const UNIPROT: &str = "uniprot";
const GEO_CC: &str = "geo_cc";
const GEO_MF: &str = "geo_mf";
const GEO_BP: &str = "geo_bp";
const NCBI: &str = "ncbi";
const LAYOUT: &str = "layout";
const POSITION: &str = "position";

/// Typed annotation data associated with a network variable.
#[pyclass(module="biodivine_aeon", extends=ModelAnnotation)]
pub struct NetworkVariableAnnotation();

/// Part of variable annotation data that represents various variable identifiers
/// across different sources.
#[pyclass(module="biodivine_aeon", extends=ModelAnnotation)]
pub struct VariableIdsAnnotation();

/// Part of variable annotation data that represents various layout-associated properties.
#[pyclass(module="biodivine_aeon", extends=ModelAnnotation)]
pub struct VariableLayoutAnnotation();

#[pymethods]
impl NetworkVariableAnnotation {
    #[getter]
    /// Returns the gene names annotation as an optional list of strings.
    ///
    /// # Examples
    ///
    /// ```
    /// # use your_crate::NetworkVariableAnnotation;
    /// # use pyo3::Python;
    /// # Python::with_gil(|py| {
    /// let annotation = NetworkVariableAnnotation::new();
    /// let gene_names = annotation.get_gene_names(py);
    /// assert!(gene_names.is_none() || gene_names.is_some());
    /// # });
    /// ```
    pub fn get_gene_names(self_: PyRef<'_, Self>, py: Python) -> Option<Vec<String>> {
        self_.as_ref().__getitem__(GENE_NAME).get_lines(py)
    }

    #[setter]
    /// Sets the gene names annotation as an optional list of strings.
    ///
    /// If `gene_names` is `Some`, updates the annotation with the provided list; if `None`, clears the gene names.
    ///
    /// # Examples
    ///
    /// ```
    /// net_var_ann.set_gene_names(py, Some(vec!["GeneA".to_string(), "GeneB".to_string()]));
    /// net_var_ann.set_gene_names(py, None); // Removes gene names
    /// ```
    pub fn set_gene_names(self_: PyRef<'_, Self>, py: Python, gene_names: Option<Vec<String>>) {
        self_
            .as_ref()
            .__getitem__(GENE_NAME)
            .set_lines(py, gene_names);
    }

    #[getter]
    /// Returns the variable identifier annotation as a `VariableIdsAnnotation` instance.
    ///
    /// This method retrieves the "id" annotation and wraps it in a `VariableIdsAnnotation` object for structured access to identifier fields.
    ///
    /// # Examples
    ///
    /// ```python
    /// ids_ann = net_var_ann.get_ids()
    /// uniprot_ids = ids_ann.get_uniprot()
    /// ```
    pub fn get_ids(self_: PyRef<'_, Self>, py: Python) -> PyResult<Py<VariableIdsAnnotation>> {
        let ann = self_.as_ref().__getitem__(ID);
        Py::new(py, (VariableIdsAnnotation(), ann))
    }

    #[setter]
    /// Sets the variable IDs annotation for this network variable.
    ///
    /// Replaces the existing "id" annotation with the provided `VariableIdsAnnotation` instance.
    ///
    /// # Examples
    ///
    /// ```python
    /// ids = VariableIdsAnnotation()
    /// ids.uniprot = ["P12345"]
    /// net_var_ann.set_ids(ids)
    /// ```
    pub fn set_ids(
        self_: PyRef<'_, Self>,
        py: Python,
        gene_names: PyRef<'_, VariableIdsAnnotation>,
    ) {
        let ann = gene_names.as_ref().clone();
        self_.as_ref().__setitem__(ID, ann, py);
    }

    #[getter]
    /// Returns the list of reference strings associated with the annotation, if present.
    ///
    /// # Examples
    ///
    /// ```
    /// let refs = annotation.get_references(py);
    /// if let Some(refs) = refs {
    ///     assert!(refs.contains(&"PMID:123456".to_string()));
    /// }
    /// ```
    pub fn get_references(self_: PyRef<'_, Self>, py: Python) -> Option<Vec<String>> {
        self_.as_ref().__getitem__(REFERENCE).get_lines(py)
    }

    #[setter]
    /// Sets the references annotation as an optional list of strings.
    ///
    /// # Examples
    ///
    /// ```
    /// net_var_ann.set_references(Some(vec!["PMID:12345".to_string(), "PMID:67890".to_string()]));
    /// ```
    pub fn set_references(self_: PyRef<'_, Self>, py: Python, references: Option<Vec<String>>) {
        self_
            .as_ref()
            .__getitem__(REFERENCE)
            .set_lines(py, references);
    }

    #[getter]
    /// Returns the layout annotation as a `VariableLayoutAnnotation` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// # use your_crate::{NetworkVariableAnnotation, VariableLayoutAnnotation};
    /// let net_var_ann = NetworkVariableAnnotation::new();
    /// let py = pyo3::Python::acquire_gil();
    /// let layout = net_var_ann.get_layout(py.python()).unwrap();
    /// assert!(layout.is_instance_of::<VariableLayoutAnnotation>(py.python()).unwrap());
    /// ```
    pub fn get_layout(
        self_: PyRef<'_, Self>,
        py: Python,
    ) -> PyResult<Py<VariableLayoutAnnotation>> {
        let ann = self_.as_ref().__getitem__(LAYOUT);
        Py::new(py, (VariableLayoutAnnotation(), ann))
    }

    #[setter]
    /// Sets the layout annotation for the network variable using the provided `VariableLayoutAnnotation`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use your_crate::{NetworkVariableAnnotation, VariableLayoutAnnotation};
    /// # use pyo3::Python;
    /// Python::with_gil(|py| {
    ///     let net_var_ann = NetworkVariableAnnotation::new();
    ///     let layout_ann = VariableLayoutAnnotation::new();
    ///     net_var_ann.set_layout(py, layout_ann);
    /// });
    /// ```
    pub fn set_layout(
        self_: PyRef<'_, Self>,
        py: Python,
        gene_names: PyRef<'_, VariableLayoutAnnotation>,
    ) {
        let ann = gene_names.as_ref().clone();
        self_.as_ref().__setitem__(LAYOUT, ann, py);
    }
}

impl NetworkVariableAnnotation {
    /// Merge two native model annotation instances which are assumed to annotate network variables.
    ///
    /// What this will do is:
    ///  - Concatenate gene names.
    ///  - Concatenate references.
    ///  - Concatenate ids/uniprot.
    ///  - Concatenate ids/ncbi.
    ///  - Concatenate ids/geo_cc.
    ///  - Concatenate ids/geo_mf.
    ///  - Concatenate ids/geo_bp.
    ///
    /// Merges annotation data from `sub` into `main` by concatenating gene names, references, and various ID fields, preserving layout information only from `main`.
    ///
    /// For each supported annotation path (gene names, references, UniProt, NCBI, and Gene Ontology IDs), if `sub` contains data, it is appended to the corresponding field in `main` with a newline separator. Layout-related data is not merged.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut main = NativeAnnotation::default();
    /// let mut sub = NativeAnnotation::default();
    /// main.set_value(&[GENE_NAME], "GeneA");
    /// sub.set_value(&[GENE_NAME], "GeneB");
    /// NetworkVariableAnnotation::extend_with(&mut main, &sub);
    /// assert!(main.get_value(&[GENE_NAME]).unwrap().contains("GeneA"));
    /// assert!(main.get_value(&[GENE_NAME]).unwrap().contains("GeneB"));
    /// ```    pub fn extend_with(main: &mut NativeAnnotation, sub: &NativeAnnotation) {
        let concat_paths: [&[&str]; 7] = [
            &[GENE_NAME],
            &[REFERENCE],
            &[ID, UNIPROT],
            &[ID, NCBI],
            &[ID, GEO_CC],
            &[ID, GEO_MF],
            &[ID, GEO_BP],
        ];
        for path in concat_paths {
            if let Some(data) = sub.get_value(path) {
                main.append_value(path, format!("\n{}", data).as_str());
            }
        }
    }
}

#[pymethods]
impl VariableIdsAnnotation {
    #[getter]
    /// Returns the UniProt identifiers as an optional list of strings.
    ///
    /// If the annotation contains UniProt IDs, each is returned as a separate string in the list. Returns `None` if no UniProt IDs are present.
    ///
    /// # Examples
    ///
    /// ```
    /// # use your_crate::VariableIdsAnnotation;
    /// let ids = VariableIdsAnnotation::new();
    /// assert_eq!(ids.get_uniprot(py), None);
    /// ```
    pub fn get_uniprot(self_: PyRef<'_, Self>, py: Python) -> Option<Vec<String>> {
        self_.as_ref().__getitem__(UNIPROT).get_lines(py)
    }

    #[setter]
    /// Sets the UniProt identifiers for the variable annotation.
    ///
    /// # Arguments
    ///
    /// * `data` - An optional list of UniProt identifier strings to associate with the annotation. If `None`, removes any existing UniProt identifiers.
    ///
    /// # Examples
    ///
    /// ```
    /// ids.set_uniprot(py, Some(vec!["P12345".to_string(), "Q67890".to_string()]));
    /// ```
    pub fn set_uniprot(self_: PyRef<'_, Self>, py: Python, data: Option<Vec<String>>) {
        self_.as_ref().__getitem__(UNIPROT).set_lines(py, data);
    }

    #[getter]
    /// Returns the Gene Ontology cellular component (GO:CC) identifiers as an optional list of strings.
    ///
    /// # Examples
    ///
    /// ```
    /// # use your_crate::VariableIdsAnnotation;
    /// # use pyo3::Python;
    /// Python::with_gil(|py| {
    ///     let annotation = VariableIdsAnnotation::new();
    ///     let geo_cc = annotation.get_geo_cc(py);
    ///     assert!(geo_cc.is_none() || geo_cc.unwrap().iter().all(|id| id.is_ascii()));
    /// });
    /// ```
    pub fn get_geo_cc(self_: PyRef<'_, Self>, py: Python) -> Option<Vec<String>> {
        self_.as_ref().__getitem__(GEO_CC).get_lines(py)
    }

    #[setter]
    /// Sets the Gene Ontology cellular component identifiers for the variable.
    ///
    /// # Arguments
    ///
    /// * `data` - An optional list of strings representing GO cellular component IDs. If `None`, the field is cleared.
    ///
    /// # Examples
    ///
    /// ```
    /// ids_ann.set_geo_cc(py, Some(vec!["GO:0005737".to_string(), "GO:0005829".to_string()]));
    /// ```
    pub fn set_geo_cc(self_: PyRef<'_, Self>, py: Python, data: Option<Vec<String>>) {
        self_.as_ref().__getitem__(GEO_CC).set_lines(py, data);
    }

    #[getter]
    /// Returns the Gene Ontology molecular function (GO:MF) identifiers as an optional list of strings.
    ///
    /// # Examples
    ///
    /// ```
    /// # use your_crate::VariableIdsAnnotation;
    /// # use pyo3::Python;
    /// Python::with_gil(|py| {
    ///     let annotation = VariableIdsAnnotation::new();
    ///     assert_eq!(annotation.get_geo_mf(py), None);
    /// });
    /// ```
    pub fn get_geo_mf(self_: PyRef<'_, Self>, py: Python) -> Option<Vec<String>> {
        self_.as_ref().__getitem__(GEO_MF).get_lines(py)
    }

    #[setter]
    /// Sets the Gene Ontology molecular function (GO:MF) identifiers for the variable.
    ///
    /// # Arguments
    ///
    /// * `data` - An optional list of strings representing GO:MF identifiers. If `None`, the field is cleared.
    ///
    /// # Examples
    ///
    /// ```
    /// ids_ann.set_geo_mf(py, Some(vec!["GO:0003674".to_string(), "GO:0005488".to_string()]));
    /// ```
    pub fn set_geo_mf(self_: PyRef<'_, Self>, py: Python, data: Option<Vec<String>>) {
        self_.as_ref().__getitem__(GEO_MF).set_lines(py, data);
    }

    #[getter]
    /// Returns the Gene Ontology biological process (GO:BP) identifiers as an optional list of strings.
    ///
    /// # Examples
    ///
    /// ```
    /// let ids = variable_ids_annotation.get_geo_bp(py);
    /// if let Some(bp_terms) = ids {
    ///     assert!(bp_terms.iter().all(|s| s.starts_with("GO:")));
    /// }
    /// ```
    pub fn get_geo_bp(self_: PyRef<'_, Self>, py: Python) -> Option<Vec<String>> {
        self_.as_ref().__getitem__(GEO_BP).get_lines(py)
    }

    #[setter]
    /// Sets the Gene Ontology biological process (GO:BP) identifiers for the variable.
    ///
    /// # Arguments
    ///
    /// * `data` - An optional list of strings representing GO:BP identifiers. If `None`, the field is cleared.
    ///
    /// # Examples
    ///
    /// ```
    /// ids_ann.set_geo_bp(py, Some(vec!["GO:0008150".to_string(), "GO:0009987".to_string()]));
    /// ```
    pub fn set_geo_bp(self_: PyRef<'_, Self>, py: Python, data: Option<Vec<String>>) {
        self_.as_ref().__getitem__(GEO_BP).set_lines(py, data);
    }

    #[getter]
    /// Returns the NCBI identifiers as an optional list of strings.
    ///
    /// # Examples
    ///
    /// ```
    /// let ids = variable_ids_annotation.get_ncbi(py);
    /// if let Some(ncbi_ids) = ids {
    ///     assert!(ncbi_ids.iter().all(|id| id.is_ascii()));
    /// }
    /// ```
    pub fn get_ncbi(self_: PyRef<'_, Self>, py: Python) -> Option<Vec<String>> {
        self_.as_ref().__getitem__(NCBI).get_lines(py)
    }

    #[setter]
    /// Sets the NCBI identifier lines for the annotation.
    ///
    /// Updates the "ncbi" field with the provided list of strings, or clears it if `None` is given.
    ///
    /// # Examples
    ///
    /// ```
    /// ids.set_ncbi(py, Some(vec!["12345".to_string(), "67890".to_string()]));
    /// ```
    pub fn set_ncbi(self_: PyRef<'_, Self>, py: Python, data: Option<Vec<String>>) {
        self_.as_ref().__getitem__(NCBI).set_lines(py, data);
    }
}

#[pymethods]
impl VariableLayoutAnnotation {
    #[getter]
    /// Retrieves the 2D position annotation as an optional tuple of floats.
    ///
    /// Returns the "position" annotation as a tuple `(x, y)` if present and parsable; otherwise, returns `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// let layout = VariableLayoutAnnotation::new();
    /// layout.set_position(Some((1.5, 2.5)));
    /// assert_eq!(layout.get_position(py), Some((1.5, 2.5)));
    /// ```
    pub fn get_position(self_: PyRef<'_, Self>, py: Python) -> Option<(f64, f64)> {
        self_
            .as_ref()
            .__getitem__(POSITION)
            .get_value(py)
            .map(|str| {
                let mut parts = str.split(',');
                let x = parts
                    .next()
                    .and_then(|x| x.parse::<f64>().ok())
                    .unwrap_or(0.0);
                let y = parts
                    .next()
                    .and_then(|x| x.parse::<f64>().ok())
                    .unwrap_or(0.0);
                (x, y)
            })
    }

    #[setter]
    /// Sets the position annotation as a comma-separated string from an optional tuple of two floats.
    ///
    /// If `data` is `Some((x, y))`, the position is stored as the string `"x,y"`. If `None`, the position annotation is cleared.
    ///
    /// # Examples
    ///
    /// ```
    /// layout.set_position(py, Some((1.5, 2.5)));
    /// // The position annotation is now "1.5,2.5"
    ///
    /// layout.set_position(py, None);
    /// // The position annotation is removed
    /// ```
    pub fn set_position(self_: PyRef<'_, Self>, py: Python, data: Option<(f64, f64)>) {
        let data = data.map(|(a, b)| format!("{},{}", a, b));
        self_.as_ref().__getitem__(POSITION).set_value(py, data);
    }
}
