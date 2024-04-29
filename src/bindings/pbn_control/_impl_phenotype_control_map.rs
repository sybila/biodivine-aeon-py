use crate::bindings::lib_bdd::PyBdd;
use crate::bindings::lib_param_bn::{PyGraphColoredVertices, PyGraphColors};
use crate::bindings::pbn_control::{PyPerturbationGraph, PyPhenotypeControlMap};
use crate::AsNative;
use biodivine_pbn_control::control::PhenotypeControlMap;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use biodivine_pbn_control::control::ControlMap;
use crate::bindings::pbn_control::py_dict_to_rust_hashmap;
use crate::bindings::pbn_control::rust_hashmap_to_py_dict;
use biodivine_lib_param_bn::symbolic_async_graph::GraphColors;
use std::collections::HashMap;


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
        PhenotypeControlMap::new(stg.as_native().clone(), colors.as_native().clone()).into()
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
    pub fn working_perturbations(
        &self,
        py: Python,
        min_robustness: f64,
        verbose: bool,
        return_all: bool
    ) -> Vec<(Py<PyDict>, PyGraphColors)> {
        self.as_native()
            .working_perturbations(min_robustness, verbose, return_all)
            .iter()
            .map(|i| (rust_hashmap_to_py_dict(py, &i.0), i.1.clone().into()))
            .collect()
    }

    /// Obtain a set of colours for which the given perturbation works
    pub fn perturbation_working_colors(&self, perturbation: &PyDict) -> PyGraphColors {
        // TEMPORARY WORKAROUND - The translation does not work properly yet in RUST -> go trough full result
        let p = py_dict_to_rust_hashmap(perturbation);
        let all = self.as_native().working_perturbations(0.01, false, true);
        let results: Vec<&(HashMap<String, bool>, GraphColors)> = all.iter().filter(|m| (m.0 == p && p == m.0)).collect();

        if results.len() > 1 {
            panic!("Multiple results matching to perturbation {:?} were found in the full result {:?}", perturbation, all)
        } else if results.len() == 1 {
            return  results[0].1.clone().into()
        } else {
            return self.as_colored_vertices().minus(&self.as_colored_vertices()).colors().into()
        }

        // let p = py_dict_to_rust_hashmap(perturbation);
        //
        // self.as_native()
        //     .perturbation_working_colors(&p)
        //     .clone()
        //     .into()
    }
}
