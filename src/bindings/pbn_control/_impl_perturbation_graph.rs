use crate::bindings::lib_bdd::PyBddVariableSet;
use crate::bindings::lib_param_bn::{
    PyBooleanNetwork, PyGraphColoredVertices, PyGraphColors, PyGraphVertices, PyParameterId,
    PySymbolicAsyncGraph, PyVariableId,
};
use crate::bindings::pbn_control::{PyAttractorControlMap, PyPhenotypeControlMap, PyPerturbationGraph};
use crate::{throw_runtime_error, AsNative};
use biodivine_lib_param_bn::biodivine_std::bitvector::{ArrayBitVector, BitVector};
use biodivine_lib_param_bn::VariableId;
use biodivine_pbn_control::perturbation::PerturbationGraph;
use biodivine_pbn_control::control::PhenotypeOscillationType;
use pyo3::prelude::*;
use pyo3::types::PyList;

impl From<PyPerturbationGraph> for PerturbationGraph {
    fn from(value: PyPerturbationGraph) -> Self {
        value.0
    }
}

impl From<PerturbationGraph> for PyPerturbationGraph {
    fn from(value: PerturbationGraph) -> Self {
        PyPerturbationGraph(value)
    }
}

impl AsNative<PerturbationGraph> for PyPerturbationGraph {
    fn as_native(&self) -> &PerturbationGraph {
        &self.0
    }

    fn as_native_mut(&mut self) -> &mut PerturbationGraph {
        &mut self.0
    }
}

pub fn convert_str_to_oscillation_type(osc: &str) -> PhenotypeOscillationType {
    match osc {
        "Forbidden" => PhenotypeOscillationType::Forbidden,
        "Allowed" => PhenotypeOscillationType::Allowed,
        "Required" => PhenotypeOscillationType::Required,
        _ => panic!("Invalid variant"),
    }
}

#[pymethods]
impl PyPerturbationGraph {
    /// Create a new `PerturbationGraph` based on a `BooleanNetwork`.
    #[new]
    pub fn new(network: &PyBooleanNetwork) -> Self {
        PerturbationGraph::new(network.as_native()).into()
    }

    /// Create a new `PerturbationGraph` based on a `BooleanNetwork` such that
    /// only the provided list of variables can be perturbed.
    ///
    /// The list can specify these variables either using names or `VariableId` objects.
    #[staticmethod]
    pub fn with_restricted_variables(
        network: PyRef<'_, PyBooleanNetwork>,
        perturb: &PyList,
    ) -> PyResult<PyPerturbationGraph> {
        let mut perturb_vars = Vec::new();
        for var in perturb.iter() {
            perturb_vars.push(network.as_ref().find_variable(var)?.unwrap().into());
        }

        Ok(
            PerturbationGraph::with_restricted_variables(network.as_native(), perturb_vars.clone())
                .into(),
        )
    }

    /// Get a `SymbolicAsyncGraph` that represents the original (unperturbed) behaviour
    /// within this graph.
    pub fn as_original(&self) -> PySymbolicAsyncGraph {
        self.as_native().as_original().clone().into()
    }

    /// Get a `SymbolicAsyncGraph` that represented the perturbed behaviour within this graph.
    pub fn as_perturbed(&self) -> PySymbolicAsyncGraph {
        self.as_native().as_perturbed().clone().into()
    }

    /// Get the underlying `BddVariableSet` that is used to encode the elements of this graph.
    pub fn bdd_variables(&self) -> PyBddVariableSet {
        self.as_native()
            .as_symbolic_context()
            .bdd_variable_set()
            .clone()
            .into()
    }

    /// Get the list of `VariableId` objects representing the variables of the underling
    /// Boolean network.
    pub fn variables(&self) -> Vec<PyVariableId> {
        self.as_native().variables().map(|i| i.into()).collect()
    }

    /// Get the list of `VariableId` objects which can be perturbed
    pub fn perturbable_variables(&self) -> Vec<PyVariableId> {
        self.as_native()
            .perturbable_variables()
            .iter()
            .map(|i| (*i).into())
            .collect()
    }

    /// Get the `ParameterId` of a parameter that is associated with the perturbation
    /// of a specific network variable (given as `VariableId` or string name).
    pub fn get_perturbation_parameter(&self, variable: &PyAny) -> PyResult<PyParameterId> {
        let variable: VariableId = self.find_variable(variable)?.into();
        if let Some(id) = self.as_native().get_perturbation_parameter(variable) {
            Ok(id.into())
        } else {
            throw_runtime_error(format!("Variable {variable:?} not found"))
        }
    }

    /*
        WARNING: The unit color set in the perturbed graph is not correct! It enforces
        observability and for a regulation to be observable, the variable cannot be perturbed.
        So the unit set only contains one non-perturbed parametrisation.
        Consequently, we use the original graph where possible.
    */

    /// Obtain an empty `ColorSet` valid in both perturbed and original asynchronous graph.
    pub fn empty_colors(&self) -> PyGraphColors {
        self.as_native().mk_empty_colors().into()
    }

    /// Obtain an empty `ColoredVertexSet` valid in both perturbed and original asynchronous graph.
    pub fn empty_colored_vertices(&self) -> PyGraphColoredVertices {
        self.as_native().mk_empty_colored_vertices().into()
    }

    /// Obtain a complete `ColorSet` that includes all admissible parameter valuations
    /// and all admissible perturbations.
    pub fn unit_colors(&self) -> PyGraphColors {
        self.as_native().mk_unit_colors().into()
    }

    /// Obtain a complete `ColoredVertexSet` of all colors and vertices (including all
    /// possible perturbations).
    pub fn unit_colored_vertices(&self) -> PyGraphColoredVertices {
        self.as_native().mk_unit_colored_vertices().into()
    }

    /// Obtain a `ColoredVertexSet` that represents a complete set of colors (parameter valuations
    /// and perturbations) associated with a single network vertex. The vertex is provided
    /// as a list of Boolean values.
    pub fn vertex(&self, state: Vec<bool>) -> PyGraphColoredVertices {
        self.as_native()
            .vertex(&ArrayBitVector::from_bool_vector(state))
            .into()
    }

    /// Obtain a `ColoredVertexSet` that represents a subset of all possible color-vertex
    /// pairs where the value of the given variable is fixed to the specified Boolean constant.
    ///
    /// Variable can be given as a name, or as `VariableId`.
    pub fn fix_variable(&self, variable: &PyAny, value: bool) -> PyResult<PyGraphColoredVertices> {
        let variable = self.find_variable(variable)?;
        Ok(self.as_native().fix_variable(variable.into(), value).into())
    }

    /// Compute a `ColoredVertexSet` representing a strong basin of the given target state
    /// (list of Boolean values) within the original graph (without perturbations).
    pub fn strong_basin(&self, target: Vec<bool>) -> PyGraphColoredVertices {
        self.as_native()
            .strong_basin(&ArrayBitVector::from_bool_vector(target))
            .into()
    }

    /// Return a `ColoredVertexSet` where the given variable is perturbed. Optionally,
    /// a constant value can be provided to further restrict the set to cases where the variable
    /// is perturbed to the given value.
    ///
    /// Variable can be given either as a name or as `VariableId`.
    ///
    /// If the value cannot be perturbed, returns an empty set.
    pub fn fix_perturbation(
        &self,
        variable: &PyAny,
        value: Option<&bool>,
    ) -> PyResult<PyGraphColoredVertices> {
        let variable = self.find_variable(variable)?;
        Ok(self
            .as_native()
            .fix_perturbation(variable.into(), value)
            .into())
    }

    /// Return a `ColorSet` representing the cases where the given variable is not perturbed.
    ///
    /// Variable can be given either as a name or as `VariableId`.
    pub fn not_perturbed(&self, variable: &PyAny) -> PyResult<PyGraphColors> {
        let variable = self.find_variable(variable)?;
        Ok(self.as_native().not_perturbed(variable.into()).into())
    }

    /// Compute the subset of `target` to which a jump from `source` is possible using a perturbation.
    ///
    /// Here, `target` is a `ColoredVertexSet` and `source` is a single state (a list of Boolean
    /// values).
    pub fn post_perturbation(
        &self,
        source: Vec<bool>,
        target: PyGraphColoredVertices,
    ) -> PyGraphColoredVertices {
        self.as_native()
            .post_perturbation(
                &ArrayBitVector::from_bool_vector(source),
                target.as_native(),
            )
            .into()
    }

    /// Compute the `ControlMap` that encodes all the possible perturbations that leads to
    /// a successful control using the *one-step* perturbation scheme.
    ///
    /// First two arguments are `source` and `target`, which represent the corresponding network
    /// states. Optionally, a third argument can be a `ColorSet` that restricts the computation
    /// to the given subset.
    pub fn one_step_control(
        &self,
        source: Vec<bool>,
        target: Vec<bool>,
        compute_params: Option<&PyGraphColors>,
        verbose: bool,
    ) -> PyAttractorControlMap {
        let compute_params = compute_params
            .map(|it| it.as_native())
            .unwrap_or_else(|| self.as_native().unit_colors());
        self.as_native()
            .one_step_control(
                &ArrayBitVector::from_bool_vector(source),
                &ArrayBitVector::from_bool_vector(target),
                compute_params,
                verbose
            )
            .into()
    }

    /// Compute the `ControlMap` that encodes all the possible perturbations that leads to
    /// a successful control using the *temporary* perturbation scheme.
    ///
    /// First two arguments are `source` and `target`, which represent the corresponding network
    /// states. Optionally, a third argument can be a `ColorSet` that restricts the computation
    /// to the given subset.
    pub fn temporary_control(
        &self,
        source: Vec<bool>,
        target: Vec<bool>,
        compute_params: Option<&PyGraphColors>,
        verbose: bool,
    ) -> PyAttractorControlMap {
        let compute_params = compute_params
            .map(|it| it.as_native())
            .unwrap_or_else(|| self.as_native().unit_colors());
        self.as_native()
            .temporary_control(
                &ArrayBitVector::from_bool_vector(source),
                &ArrayBitVector::from_bool_vector(target),
                compute_params,
                verbose
            )
            .into()
    }

    /// Compute the `ControlMap` that encodes all the possible perturbations that leads to
    /// a successful control using the *permanent* perturbation scheme.
    ///
    /// First two arguments are `source` and `target`, which represent the corresponding network
    /// states. Optionally, a third argument can be a `ColorSet` that restricts the computation
    /// to the given subset.
    pub fn permanent_control(
        &self,
        source: Vec<bool>,
        target: Vec<bool>,
        compute_params: Option<&PyGraphColors>,
        verbose: bool,
    ) -> PyAttractorControlMap {
        let compute_params = compute_params
            .map(|it| it.as_native())
            .unwrap_or_else(|| self.as_native().unit_colors());
        self.as_native()
            .permanent_control(
                &ArrayBitVector::from_bool_vector(source),
                &ArrayBitVector::from_bool_vector(target),
                compute_params,
                verbose
            )
            .into()
    }

    // pub fn phenotype_permanent_control(
    //     &self,
    //     phenotype: GraphVertices,
    //     admissible_colors_perturbations: GraphColors,
    //     oscillation: PhenotypeOscillationType,
    //     verbose: bool
    // )

    pub fn phenotype_permanent_control(
        &self,
        phenotype: PyGraphVertices,
        oscillation: &str,
        verbose: bool,
    ) -> PyPhenotypeControlMap {
        let converted_phenotype = phenotype.as_native().clone();
        let converted_oscillation = convert_str_to_oscillation_type(oscillation);
        let result = self.as_native().phenotype_permanent_control(
            converted_phenotype,
            converted_oscillation,
            verbose,
        );
        result.into()
    }

    pub fn ceiled_phenotype_permanent_control(
        &self,
        phenotype: PyGraphVertices,
        size_bound: usize,
        oscillation: &str,
        stop_early: bool,
        verbose: bool,
    ) -> PyPhenotypeControlMap {
        let converted_phenotype = phenotype.as_native().clone();
        let converted_oscillation = convert_str_to_oscillation_type(oscillation);
        let result = self.as_native().ceiled_phenotype_permanent_control(
            converted_phenotype,
            size_bound,
            converted_oscillation,
            stop_early,
            verbose,
        );
        result.into()
    }

    /// Resolves a string or `VariableId` to `VariableId`.
    pub fn find_variable(&self, variable: &PyAny) -> PyResult<PyVariableId> {
        if let Ok(name) = variable.extract::<String>() {
            let ctx = self.as_native().as_original().symbolic_context();
            if let Some(var) = ctx.find_network_variable(name.as_str()) {
                Ok(var.into())
            } else {
                throw_runtime_error(format!("Variable {name} not found."))
            }
        } else {
            variable.extract::<PyVariableId>()
        }
    }
}
