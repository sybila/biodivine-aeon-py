use std::collections::HashMap;
use crate::bindings::lib_bdd::PyBdd;
use crate::bindings::lib_param_bn::{PyGraphColoredVertices, PyGraphColors};
use crate::bindings::pbn_control::{PyPerturbationGraph, PyPhenotypeControlMap};
use crate::{AsNative, throw_runtime_error};
use biodivine_pbn_control::phenotype_control::PhenotypeControlMap;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

fn py_dict_to_rust_hashmap(py_dict: &PyDict) -> HashMap<String, bool> {
    let mut rust_hashmap = HashMap::new();

    for (key, value) in py_dict.iter() {
        if let (Ok(py_key), Ok(py_value)) = (key.extract::<String>(), value.extract::<bool>()) {
            rust_hashmap.insert(py_key, py_value);
        }
    }

    rust_hashmap
}

fn rust_hashmap_to_py_dict(py:Python, rust_hashmap: &HashMap<String, bool>) -> Py<PyDict> {
    let mut pyDict = PyDict::new(py);

    for (k, v) in rust_hashmap {
        pyDict.set_item(k, v).unwrap();
    }

    pyDict.into()
}

impl From<PhenotypeControlMap> for PyPhenotypeControlMap {
    fn from(value: PhenotypeControlMap) -> Self {
        PyPhenotypeControlMap(value)
    }
}

impl From<PyPhenotypeControlMap> for PhenotypeControlMap {
    fn from(value: PyPhenotypeControlMap) -> Self {
        value.0
    }
}

impl AsNative<PhenotypeControlMap> for PyPhenotypeControlMap {
    fn as_native(&self) -> &PhenotypeControlMap {
        &self.0
    }

    fn as_native_mut(&mut self) -> &mut PhenotypeControlMap {
        &mut self.0
    }
}

#[pymethods]
impl PyPhenotypeControlMap {
    #[new]
    pub fn new(colors: PyGraphColoredVertices, stg: PyPerturbationGraph) -> Self {
        PhenotypeControlMap::new(
            stg.as_native().clone(),
            colors.as_native().clone(),
        ).into()
    }


    /// Obtain a copy of the underlying `Bdd` representing this map.
    pub fn as_bdd(&self) -> PyBdd {
        self.as_native().as_bdd().clone().into()
    }

    /// Obtain a copy of this map as a `ColoredVertexSet`. This set is useful when considering
    /// the internal representation employed by `PerturbationGraph`.
    pub fn as_colored_vertices(&self) -> PyGraphColoredVertices {
        self.as_native().as_colored_vertices().clone().into()
    }

    // pub fn working_perturbations(&self, min_robustness: f64, verbose: bool) -> Vec<(HashMap<String, bool>, GraphColors)> {
    /// Obtain list of working perturbations
    pub fn working_perturbations(&self, py: Python, min_robustness: f64, verbose: bool) -> Vec<(Py<PyDict>, PyGraphColors)> {
        self.as_native().working_perturbations(min_robustness, verbose).iter().map(|i| (rust_hashmap_to_py_dict(py, &i.0), i.1.clone().into())).collect()
    }

    /// Obtain a set of colours for which the given perturbation works
    pub fn perturbation_working_colors(&self, perturbation: &PyDict) -> PyGraphColors {
        let p = py_dict_to_rust_hashmap(perturbation);

        self.as_native().perturbation_working_colors(&p).clone().into()
    }
}
