use crate::internal::scc::{Behaviour, Classifier};
use biodivine_bdd::{Bdd, BddVariable, BddVariableSet, BddVariableSetBuilder, BooleanExpression};
use biodivine_boolean_networks::{
    BooleanNetwork, ColorSet, ColoredVertexSet, ParameterId, RegulatoryGraph, SymbolicAsyncGraph,
    VariableId, VertexSet,
};
use biodivine_lib_param_bn::biodivine_std::bitvector::ArrayBitVector;
use biodivine_lib_param_bn::biodivine_std::bitvector::BitVector;
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

#[pyclass]
pub struct ControlMap(biodivine_pbn_control::control::ControlMap);

impl From<ControlMap> for biodivine_pbn_control::control::ControlMap {
    fn from(value: ControlMap) -> Self {
        value.0
    }
}

impl From<biodivine_pbn_control::control::ControlMap> for ControlMap {
    fn from(value: biodivine_pbn_control::control::ControlMap) -> Self {
        ControlMap(value)
    }
}

#[pymethods]
impl ControlMap {
    /// Remove from this control map any results that *do not* perturb `variable`.
    /// If `value` is given, only keep perturbations which result in this value.
    pub fn require_perturbation(&mut self, variable: VariableId, value: Option<bool>) {
        self.0.require_perturbation(variable.into(), value);
    }

    /// Remove from this control map any results that perturb `variable`. If `value` is given,
    /// only remove perturbations which result in this value.
    pub fn exclude_perturbation(&mut self, variable: VariableId, value: Option<bool>) {
        self.0.exclude_perturbation(variable.into(), value);
    }

    pub fn as_bdd(&self) -> Bdd {
        self.0.as_bdd().clone().into()
    }

    pub fn as_colored_vertices(&self) -> ColoredVertexSet {
        self.0.as_colored_vertices().clone().into()
    }

    pub fn controllable_colors(&self) -> Bdd {
        self.0.controllable_colors().into()
    }

    pub fn controllable_colors_cardinality(&self) -> f64 {
        self.0.controllable_colors_cardinality()
    }

    pub fn jump_vertices(&self) -> f64 {
        self.0.jump_vertices()
    }
}

#[pyclass]
pub struct PerturbationGraph(biodivine_pbn_control::perturbation::PerturbationGraph);

impl From<PerturbationGraph> for biodivine_pbn_control::perturbation::PerturbationGraph {
    fn from(value: PerturbationGraph) -> Self {
        value.0
    }
}

impl From<biodivine_pbn_control::perturbation::PerturbationGraph> for PerturbationGraph {
    fn from(value: biodivine_pbn_control::perturbation::PerturbationGraph) -> Self {
        PerturbationGraph(value)
    }
}

#[pymethods]
impl PerturbationGraph {
    /// Create a new Boolean network with no functions using a regulatory graph.
    #[new]
    pub fn new(network: BooleanNetwork) -> PerturbationGraph {
        let network_ = network.into();
        biodivine_pbn_control::perturbation::PerturbationGraph::new(&network_).into()
    }

    /// Create a new perturbation graph for a given Boolean network.
    #[staticmethod]
    pub fn with_restricted_variables(
        network: BooleanNetwork,
        perturb: Vec<VariableId>,
    ) -> PerturbationGraph {
        let network_ = network.into();
        let perturb_ = &perturb
            .into_iter()
            .map(|i| i.into())
            .collect::<Vec<biodivine_lib_param_bn::VariableId>>()[..];
        biodivine_pbn_control::perturbation::PerturbationGraph::with_restricted_variables(
            &network_, perturb_,
        )
        .into()
    }

    pub fn as_original(&self) -> SymbolicAsyncGraph {
        self.0.as_original().clone().into()
    }

    pub fn as_perturbed(&self) -> SymbolicAsyncGraph {
        self.0.as_perturbed().clone().into()
    }

    // Not (easily) exportable now
    // pub fn as_symbolic_context(&self) -> &SymbolicContext {
    //     self.0.as_symbolic_context().clone().into()
    // }

    pub fn variables(&self) -> Vec<VariableId> {
        self.0.variables().map(|i| i.into()).collect()
    }

    pub fn get_perturbation_parameter(&self, variable: VariableId) -> PyResult<ParameterId> {
        if let Some(result) = self.0.get_perturbation_parameter(variable.into()) {
            Ok(result.into())
        } else {
            Err(PyTypeError::new_err(format!(
                "Variable {:?} not found",
                variable
            )))
        }
    }

    /*
        WARNING: The unit color set in the perturbed graph is not correct! It enforces
        observability and for a regulation to be observable, the variable cannot be perturbed.
        So the unit set only contains one non-perturbed parametrisation.
        Consequently, we use the original graph where possible.
    */

    pub fn mk_empty_colors(&self) -> ColorSet {
        self.0.mk_empty_colors().into()
    }

    pub fn mk_empty_colored_vertices(&self) -> ColoredVertexSet {
        self.0.mk_empty_colored_vertices().into()
    }

    pub fn mk_unit_colors(&self) -> ColorSet {
        self.0.mk_unit_colors().into()
    }

    pub fn mk_unit_colored_vertices(&self) -> ColoredVertexSet {
        self.0.mk_unit_colored_vertices().into()
    }

    pub fn vertex(&self, state: Vec<bool>) -> ColoredVertexSet {
        let state_ = ArrayBitVector::from_bool_vector(state);
        self.0.vertex(&state_).into()
    }

    pub fn fix_variable(&self, variable: VariableId, value: bool) -> ColoredVertexSet {
        self.0.fix_variable(variable.into(), value).into()
    }

    pub fn strong_basin(&self, target: Vec<bool>) -> ColoredVertexSet {
        let target = ArrayBitVector::from_bool_vector(target);
        self.0.strong_basin(&target).into()
    }

    /// Return a subset of vertices and colors where the variable is perturbed to the given value.
    ///
    /// If no value is given, return vertices and colors where the variable is perturbed.
    ///
    /// If the value cannot be perturbed, return empty set.
    pub fn fix_perturbation(&self, variable: VariableId, value: Option<bool>) -> ColoredVertexSet {
        return self.0.fix_perturbation(variable.into(), value).into();
    }

    /// Return a subset of colors for which the given `variable` is not perturbed.
    pub fn not_perturbed(&self, variable: VariableId) -> ColorSet {
        self.0.not_perturbed(variable.into()).into()
    }

    /// Compute the subset of `target` to which a jump from `source` is possible using a perturbation.
    pub fn post_perturbation(
        &self,
        source: Vec<bool>,
        target: ColoredVertexSet,
    ) -> ColoredVertexSet {
        let source_ = ArrayBitVector::from_bool_vector(source);
        self.0
            .post_perturbation(&source_, &target.into())
            .clone()
            .into()
    }

    pub fn one_step_control(
        &self,
        source: Vec<bool>,
        target: Vec<bool>,
        compute_params: ColorSet,
    ) -> ControlMap {
        let source_ = ArrayBitVector::from_bool_vector(source);
        let target_ = ArrayBitVector::from_bool_vector(target);
        let compute_params_ = &compute_params.into();

        self.0
            .one_step_control(&source_, &target_, &compute_params_)
            .into()
    }

    pub fn temporary_control(
        &self,
        source: Vec<bool>,
        target: Vec<bool>,
        compute_params: ColorSet,
    ) -> ControlMap {
        let source_ = ArrayBitVector::from_bool_vector(source);
        let target_ = ArrayBitVector::from_bool_vector(target);
        let compute_params_ = &compute_params.into();

        self.0
            .temporary_control(&source_, &target_, &compute_params_)
            .into()
    }

    pub fn permanent_control(
        &self,
        source: Vec<bool>,
        target: Vec<bool>,
        compute_params: ColorSet,
    ) -> ControlMap {
        let source_ = ArrayBitVector::from_bool_vector(source);
        let target_ = ArrayBitVector::from_bool_vector(target);
        let compute_params_ = &compute_params.into();

        self.0
            .permanent_control(&source_, &target_, &compute_params_)
            .into()
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn biodivine_aeon(_py: Python, module: &PyModule) -> PyResult<()> {
    module.add_class::<VariableId>()?;
    module.add_class::<ParameterId>()?;
    module.add_class::<RegulatoryGraph>()?;
    module.add_class::<BooleanNetwork>()?;
    module.add_class::<SymbolicAsyncGraph>()?;
    module.add_class::<ColorSet>()?;
    module.add_class::<VertexSet>()?;
    module.add_class::<ColoredVertexSet>()?;
    module.add_class::<ControlMap>()?;
    module.add_class::<PerturbationGraph>()?;
    // Re-export everything here as well, because the types are incompatible in Python :/
    module.add_class::<Bdd>()?;
    module.add_class::<BddVariable>()?;
    module.add_class::<BddVariableSet>()?;
    module.add_class::<BddVariableSetBuilder>()?;
    module.add_class::<BooleanExpression>()?;
    module.add_function(wrap_pyfunction!(classify_attractor, module)?)?;
    module.add_function(wrap_pyfunction!(find_attractors, module)?)?;
    module.add_function(wrap_pyfunction!(reach_bwd, module)?)?;
    module.add_function(wrap_pyfunction!(reach_fwd, module)?)?;
    module.add_function(wrap_pyfunction!(xie_beerel_attractors, module)?)?;
    module.add_function(wrap_pyfunction!(transition_guided_reduction, module)?)?;
    Ok(())
}
