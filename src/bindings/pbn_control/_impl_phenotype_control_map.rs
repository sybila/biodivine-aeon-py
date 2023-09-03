use std::collections::HashMap;
use crate::bindings::lib_bdd::PyBdd;
use crate::bindings::lib_param_bn::{PyGraphColoredVertices, PyGraphColors};
use crate::bindings::pbn_control::PyPhenotypeControlMap;
use crate::AsNative;
use biodivine_pbn_control::phenotype_control::PhenotypeControlMap;
use pyo3::prelude::*;
use pyo3::types::PyDict;

fn py_dict_to_rust_hashmap(py_dict: &PyDict) -> HashMap<String, bool> {
    let mut rust_hashmap = HashMap::new();

    for (key, value) in py_dict.iter() {
        if let (Ok(py_key), Ok(py_value)) = (key.extract::<String>(), value.extract::<bool>()) {
            rust_hashmap.insert(py_key, py_value);
        }
    }

    rust_hashmap
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
    /// Obtain a copy of the underlying `Bdd` representing this map.
    pub fn as_bdd(&self) -> PyBdd {
        self.as_native().as_bdd().clone().into()
    }

    /// Obtain a copy of this map as a `ColoredVertexSet`. This set is useful when considering
    /// the internal representation employed by `PerturbationGraph`.
    pub fn as_colored_vertices(&self) -> PyGraphColoredVertices {
        self.as_native().as_colored_vertices().clone().into()
    }


    /// Obtain a set of colours for which the given perturbation works
    pub fn perturbation_working_colors(&self, perturbation: &PyDict) -> PyGraphColors {
        let p = py_dict_to_rust_hashmap(perturbation);

        self.as_native().perturbation_working_colors(&p).clone().into()
    }
}
