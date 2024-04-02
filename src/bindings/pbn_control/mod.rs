use std::collections::HashMap;
use biodivine_pbn_control::control::{AttractorControlMap, PhenotypeControlMap};
use biodivine_pbn_control::perturbation::PerturbationGraph;
use pyo3::prelude::*;
use pyo3::types::PyDict;

mod _impl_attractor_control_map;
mod _impl_perturbation_graph;
mod _impl_phenotype_control_map;

pub(crate) fn register(module: &PyModule) -> PyResult<()> {
    module.add_class::<PyPerturbationGraph>()?;
    module.add_class::<PyPhenotypeControlMap>()?;
    module.add_class::<PyAttractorControlMap>()?;
    Ok(())
}

#[pyclass(name = "AttractorControlMap")]
#[derive(Clone)]
pub struct PyAttractorControlMap(AttractorControlMap);

/// A symbolically represented state-transition graph that supports perturbations in all
/// admissible BN variables.
#[pyclass(name = "PerturbationGraph")]
#[derive(Clone)]
pub struct PyPerturbationGraph(PerturbationGraph);

#[pyclass(name = "PhenotypeControlMap")]
#[derive(Clone)]
pub struct PyPhenotypeControlMap(PhenotypeControlMap);


fn py_dict_to_rust_hashmap(py_dict: &PyDict) -> HashMap<String, bool> {
    let mut rust_hashmap = HashMap::new();

    for (key, value) in py_dict.iter() {
        if let (Ok(py_key), Ok(py_value)) = (key.extract::<String>(), value.extract::<bool>()) {
            rust_hashmap.insert(py_key, py_value);
        }
    }

    rust_hashmap
}

fn rust_hashmap_to_py_dict(py: Python, rust_hashmap: &HashMap<String, bool>) -> Py<PyDict> {
    let py_dict = PyDict::new(py);

    for (k, v) in rust_hashmap {
        py_dict.set_item(k, v).unwrap();
    }

    py_dict.into()
}
