use std::collections::HashMap;
use biodivine_lib_bdd::Bdd;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, GraphColors};
use crate::bindings::lib_bdd::PyBdd;
use crate::bindings::lib_param_bn::{PyGraphColoredVertices, PyVariableId, PyGraphColors};
use crate::bindings::pbn_control::{PyPerturbationGraph, PyAttractorControlMap};
use crate::AsNative;
use biodivine_pbn_control::control::{AttractorControlMap, ControlMap,};
use pyo3::prelude::*;
use crate::bindings::pbn_control::py_dict_to_rust_hashmap;
use crate::bindings::pbn_control::rust_hashmap_to_py_dict;
use pyo3::types::PyDict;


impl From<AttractorControlMap> for PyAttractorControlMap {
    fn from(value: AttractorControlMap) -> Self {
        PyAttractorControlMap(value)
    }
}

impl From<PyAttractorControlMap> for AttractorControlMap {
    fn from(value: PyAttractorControlMap) -> Self {
        value.0
    }
}

impl AsNative<AttractorControlMap> for PyAttractorControlMap {
    fn as_native(&self) -> &AttractorControlMap {
        &self.0
    }

    fn as_native_mut(&mut self) -> &mut AttractorControlMap {
        &mut self.0
    }
}

#[pymethods]
impl PyAttractorControlMap {
    #[new]
    pub fn new(colors: PyGraphColoredVertices, stg: PyPerturbationGraph) -> Self {
        AttractorControlMap::new(stg.as_native().clone(), colors.as_native().clone()).into()
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
        return_only_smallest: bool
    ) -> Vec<(Py<PyDict>, PyGraphColors)> {
        self.as_native()
            .working_perturbations(min_robustness, verbose, return_only_smallest)
            .iter()
            .map(|i| (rust_hashmap_to_py_dict(py, &i.0), i.1.clone().into()))
            .collect()
    }

    /// Obtain a set of colours for which the given perturbation works
    pub fn perturbation_working_colors(&self, perturbation: &PyDict) -> PyGraphColors {
        let p = py_dict_to_rust_hashmap(perturbation);

        self.as_native()
            .perturbation_working_colors(&p)
            .clone()
            .into()
    }
}
//
// #[pymethods]
// impl ControlMap for PyAttractorControlMap {
//     fn new(context: PerturbationGraph, perturbation_set: GraphColoredVertices) -> Self {
//         todo!()
//     }
//
//     fn as_bdd(&self) -> &Bdd {
//         todo!()
//     }
//
//     fn as_colored_vertices(&self) -> &GraphColoredVertices {
//         todo!()
//     }
//
//     fn working_perturbations(&self, min_robustness: f64, verbose: bool, return_all: bool) -> Vec<(HashMap<String, bool>, GraphColors)> {
//         todo!()
//     }
//
//     fn perturbation_working_colors(&self, perturbation: &HashMap<String, bool>) -> GraphColors {
//         todo!()
//     }
// }

