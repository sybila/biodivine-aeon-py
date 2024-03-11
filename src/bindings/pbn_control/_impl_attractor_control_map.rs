use std::collections::HashMap;
use biodivine_lib_bdd::Bdd;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, GraphColors};
use crate::bindings::lib_bdd::PyBdd;
use crate::bindings::lib_param_bn::{PyGraphColoredVertices, PyVariableId};
use crate::bindings::pbn_control::PyAttractorControlMap;
use crate::AsNative;
use biodivine_pbn_control::control::{AttractorControlMap, ControlMap,};
use biodivine_pbn_control::perturbation::PerturbationGraph;
use pyo3::prelude::*;

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

// #[pymethods]
// impl PyControlMap {
    /// Modify this `ControlMap` such that it only retains perturbations where the given variable
    /// (specified using a `VariableId`) is perturbed. Optionally, a Boolean constant value
    /// can be provided, in which case the perturbations also must lead to this value.
    // pub fn require_perturbation(&mut self, variable: PyVariableId, value: Option<bool>) {
    //     self.as_native_mut()
    //         .require_perturbation(variable.into(), value);
    // }

    /// Modify this `ControlMap` such that it only retains perturbations where the given variable
    /// (specified using a `VariableId`) is **not** perturbed. Optionally, a Boolean constant value
    /// can be provided, in which case the map retains perturbations where the variable is not
    /// perturbed to this value (e.g. it can still be perturbed to the opposite value).
    // pub fn exclude_perturbation(&mut self, variable: PyVariableId, value: Option<bool>) {
    //     self.as_native_mut()
    //         .exclude_perturbation(variable.into(), value);
    // }

    /// Obtain a copy of the underlying `Bdd` representing this map.
    // pub fn as_bdd(&self) -> PyBdd {
    //     self.as_native().as_bdd().clone().into()
    // }

    /// Obtain a copy of this map as a `ColoredVertexSet`. This set is useful when considering
    /// the internal representation employed by `PerturbationGraph`.
    // pub fn as_colored_vertices(&self) -> PyGraphColoredVertices {
    //     self.as_native().as_colored_vertices().clone().into()
    // }

    /// Obtain a `Bdd` that represents the set of colors (network parameter valuations) that are
    /// controllable by the perturbations in this map.
    // pub fn controllable_colors(&self) -> PyBdd {
    //     self.as_native().controllable_colors().into()
    // }

    /// Obtain the approx. number of colors (network parameter valuations) that are controllable
    /// by the perturbations in this map.
    // pub fn controllable_colors_cardinality(&self) -> f64 {
    //     self.0.controllable_colors_cardinality()
    // }

    /// Obtain the approx. number of vertices the source state can jump to in order to
    /// achieve the desired target assuming *some* perturbation.
    // pub fn jump_vertices(&self) -> f64 {
    //     self.as_native().jump_vertices()
    // }
// }


impl ControlMap for PyAttractorControlMap {
    fn new(context: PerturbationGraph, perturbation_set: GraphColoredVertices) -> Self {
        todo!()
    }

    fn as_bdd(&self) -> &Bdd {
        todo!()
    }

    fn as_colored_vertices(&self) -> &GraphColoredVertices {
        todo!()
    }

    fn working_perturbations(&self, min_robustness: f64, verbose: bool, return_all: bool) -> Vec<(HashMap<String, bool>, GraphColors)> {
        todo!()
    }

    fn perturbation_working_colors(&self, perturbation: &HashMap<String, bool>) -> GraphColors {
        todo!()
    }
}

