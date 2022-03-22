use biodivine_pbn_control::control::ControlMap;
use biodivine_pbn_control::perturbation::PerturbationGraph;
use pyo3::prelude::*;

mod _impl_control_map;
mod _impl_perturbation_graph;

pub(crate) fn register(module: &PyModule) -> PyResult<()> {
    module.add_class::<PyControlMap>()?;
    module.add_class::<PyPerturbationGraph>()?;
    Ok(())
}

/// A symbolic representation of possible perturbation strategies associated with
/// Boolean network parameter valuations.
#[pyclass(name = "ControlMap")]
#[derive(Clone)]
pub struct PyControlMap(ControlMap);

/// A symbolically represented state-transition graph that supports perturbations in all
/// admissible BN variables.
#[pyclass(name = "PerturbationGraph")]
#[derive(Clone)]
pub struct PyPerturbationGraph(PerturbationGraph);
