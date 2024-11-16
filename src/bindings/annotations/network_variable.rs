use crate::bindings::annotations::REFERENCE;
use crate::bindings::lib_param_bn::model_annotation::ModelAnnotation;
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

#[pyclass(module="biodivine_aeon", extends=ModelAnnotation)]
pub struct NetworkVariableAnnotation();

#[pyclass(module="biodivine_aeon", extends=ModelAnnotation)]
pub struct VariableIdsAnnotation();

#[pyclass(module="biodivine_aeon", extends=ModelAnnotation)]
pub struct VariableLayoutAnnotation();

#[pymethods]
impl NetworkVariableAnnotation {
    #[getter]
    pub fn get_gene_names(self_: PyRef<'_, Self>, py: Python) -> Option<Vec<String>> {
        self_.as_ref().__getitem__(GENE_NAME).get_lines(py)
    }

    #[setter]
    pub fn set_gene_names(self_: PyRef<'_, Self>, py: Python, gene_names: Option<Vec<String>>) {
        self_
            .as_ref()
            .__getitem__(GENE_NAME)
            .set_lines(py, gene_names);
    }

    #[getter]
    pub fn get_ids(self_: PyRef<'_, Self>, py: Python) -> PyResult<Py<VariableIdsAnnotation>> {
        let ann = self_.as_ref().__getitem__(ID);
        Py::new(py, (VariableIdsAnnotation(), ann))
    }

    #[setter]
    pub fn set_ids(
        self_: PyRef<'_, Self>,
        py: Python,
        gene_names: PyRef<'_, VariableIdsAnnotation>,
    ) {
        let ann = gene_names.as_ref().clone();
        self_.as_ref().__setitem__(ID, ann, py);
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
    pub fn get_layout(
        self_: PyRef<'_, Self>,
        py: Python,
    ) -> PyResult<Py<VariableLayoutAnnotation>> {
        let ann = self_.as_ref().__getitem__(LAYOUT);
        Py::new(py, (VariableLayoutAnnotation(), ann))
    }

    #[setter]
    pub fn set_layout(
        self_: PyRef<'_, Self>,
        py: Python,
        gene_names: PyRef<'_, VariableLayoutAnnotation>,
    ) {
        let ann = gene_names.as_ref().clone();
        self_.as_ref().__setitem__(LAYOUT, ann, py);
    }
}

#[pymethods]
impl VariableIdsAnnotation {
    #[getter]
    pub fn get_uniprot(self_: PyRef<'_, Self>, py: Python) -> Option<Vec<String>> {
        self_.as_ref().__getitem__(UNIPROT).get_lines(py)
    }

    #[setter]
    pub fn set_uniprot(self_: PyRef<'_, Self>, py: Python, data: Option<Vec<String>>) {
        self_.as_ref().__getitem__(UNIPROT).set_lines(py, data);
    }

    #[getter]
    pub fn get_geo_cc(self_: PyRef<'_, Self>, py: Python) -> Option<Vec<String>> {
        self_.as_ref().__getitem__(GEO_CC).get_lines(py)
    }

    #[setter]
    pub fn set_geo_cc(self_: PyRef<'_, Self>, py: Python, data: Option<Vec<String>>) {
        self_.as_ref().__getitem__(GEO_CC).set_lines(py, data);
    }

    #[getter]
    pub fn get_geo_mf(self_: PyRef<'_, Self>, py: Python) -> Option<Vec<String>> {
        self_.as_ref().__getitem__(GEO_MF).get_lines(py)
    }

    #[setter]
    pub fn set_geo_mf(self_: PyRef<'_, Self>, py: Python, data: Option<Vec<String>>) {
        self_.as_ref().__getitem__(GEO_MF).set_lines(py, data);
    }

    #[getter]
    pub fn get_geo_bp(self_: PyRef<'_, Self>, py: Python) -> Option<Vec<String>> {
        self_.as_ref().__getitem__(GEO_BP).get_lines(py)
    }

    #[setter]
    pub fn set_geo_bp(self_: PyRef<'_, Self>, py: Python, data: Option<Vec<String>>) {
        self_.as_ref().__getitem__(GEO_BP).set_lines(py, data);
    }

    #[getter]
    pub fn get_ncbi(self_: PyRef<'_, Self>, py: Python) -> Option<Vec<String>> {
        self_.as_ref().__getitem__(NCBI).get_lines(py)
    }

    #[setter]
    pub fn set_ncbi(self_: PyRef<'_, Self>, py: Python, data: Option<Vec<String>>) {
        self_.as_ref().__getitem__(NCBI).set_lines(py, data);
    }
}

#[pymethods]
impl VariableLayoutAnnotation {
    #[getter]
    pub fn get_position(self_: PyRef<'_, Self>, py: Python) -> Option<(f64, f64)> {
        self_
            .as_ref()
            .__getitem__(POSITION)
            .get_value(py)
            .map(|str| {
                let mut parts = str.split(",");
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
    pub fn set_position(self_: PyRef<'_, Self>, py: Python, data: Option<(f64, f64)>) {
        let data = data.map(|(a, b)| format!("{},{}", a, b));
        self_.as_ref().__getitem__(POSITION).set_value(py, data);
    }
}
