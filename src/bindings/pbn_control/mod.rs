use std::collections::HashMap;
use biodivine_lib_bdd::Bdd;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, GraphColors};
use biodivine_pbn_control::control::{AttractorControlMap, PhenotypeControlMap};
use biodivine_pbn_control::perturbation::PerturbationGraph;
use pyo3::prelude::*;

mod _impl_attractor_control_map;
mod _impl_perturbation_graph;
mod _impl_phenotype_control_map;

pub(crate) fn register(module: &PyModule) -> PyResult<()> {
    module.add_class::<PyPerturbationGraph>()?;
    module.add_class::<PyPhenotypeControlMap>()?;
    module.add_class::<PyAttractorControlMap>()?;
    Ok(())
}

// /// A symbolic representation of possible perturbation strategies associated with
// /// Boolean network parameter valuations.
// #[pyclass(name = "ControlMap")]
// #[derive(Clone)]
// pub struct PyControlMap(dyn ControlMap);

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
