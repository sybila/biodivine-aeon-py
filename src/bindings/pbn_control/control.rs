use crate::bindings::lib_param_bn::symbolic::set_color::ColorSet;
use crate::bindings::lib_param_bn::symbolic::set_vertex::VertexSet;
use crate::bindings::pbn_control::{
    AsynchronousPerturbationGraph, ColoredPerturbationSet, extract_phenotype_type,
};
use crate::{AsNative, global_log_level, should_log, throw_runtime_error};
use biodivine_lib_param_bn::biodivine_std::bitvector::{ArrayBitVector, BitVector};
use biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices;
use biodivine_pbn_control::control::{ControlMap, PhenotypeOscillationType};
use pyo3::prelude::PyAnyMethods;
use pyo3::{Bound, Py, PyAny, PyResult, Python, pyclass, pymethods};

#[pyclass(module = "biodivine_aeon", frozen)]
pub struct Control {
    _dummy: (),
}

#[pymethods]
impl Control {
    /// Compute the color-perturbation pairs which guarantee that the network reaches
    /// a `target` attractor from the given `source` state, assuming the perturbation is applied
    /// for a single time step.
    ///
    /// Optionally, you can provide a subset of relevant `colors` that will be considered. If not
    /// given, the method considers all colors.
    #[staticmethod]
    #[pyo3(signature = (graph, source, target, colors = None))]
    pub fn attractor_one_step(
        py: Python,
        graph: Py<AsynchronousPerturbationGraph>,
        source: &Bound<'_, PyAny>,
        target: &Bound<'_, PyAny>,
        colors: Option<ColorSet>,
    ) -> PyResult<ColoredPerturbationSet> {
        let source = extract_state(py, &graph, source)?;
        let target = extract_state(py, &graph, target)?;

        let colors = match colors {
            Some(x) => x,
            None => AsynchronousPerturbationGraph::mk_perturbable_unit_colors(graph.clone(), py),
        };

        let verbose = should_log(global_log_level(py)?);

        let perturbations =
            graph
                .get()
                .as_native()
                .one_step_control(&source, &target, colors.as_native(), verbose);

        Ok(sanitize_control_map(graph, perturbations.as_bdd().clone()))
    }

    /// Compute the color-perturbation pairs which guarantee that the network reaches
    /// a `target` attractor from the given `source` state, assuming the perturbation is applied
    /// indefinitely, but the system eventually returns to its original dynamics.
    ///
    /// Optionally, you can provide a subset of relevant `colors` that will be considered. If not
    /// given, the method considers all colors.
    #[staticmethod]
    #[pyo3(signature = (graph, source, target, colors = None))]
    pub fn attractor_temporary(
        py: Python,
        graph: Py<AsynchronousPerturbationGraph>,
        source: &Bound<'_, PyAny>,
        target: &Bound<'_, PyAny>,
        colors: Option<ColorSet>,
    ) -> PyResult<ColoredPerturbationSet> {
        let source = extract_state(py, &graph, source)?;
        let target = extract_state(py, &graph, target)?;

        let colors = match colors {
            Some(x) => x,
            None => graph.borrow(py).as_ref().mk_unit_colors(),
        };

        let verbose = should_log(global_log_level(py)?);

        let perturbations = graph.get().as_native().temporary_control(
            &source,
            &target,
            colors.as_native(),
            verbose,
        );

        Ok(sanitize_control_map(graph, perturbations.as_bdd().clone()))
    }

    /// Compute the color-perturbation pairs which guarantee that the network reaches
    /// a `target` attractor from the given `source` state, assuming the perturbation is applied
    /// indefinitely.
    ///
    /// Optionally, you can provide a subset of relevant `colors` that will be considered. If not
    /// given, the method considers all colors.
    #[staticmethod]
    #[pyo3(signature = (graph, source, target, colors = None))]
    pub fn attractor_permanent(
        py: Python,
        graph: Py<AsynchronousPerturbationGraph>,
        source: &Bound<'_, PyAny>,
        target: &Bound<'_, PyAny>,
        colors: Option<ColorSet>,
    ) -> PyResult<ColoredPerturbationSet> {
        let source = extract_state(py, &graph, source)?;
        let target = extract_state(py, &graph, target)?;

        let colors = match colors {
            Some(x) => x,
            None => graph.borrow(py).as_ref().mk_unit_colors(),
        };

        let verbose = should_log(global_log_level(py)?);

        let perturbations = graph.get().as_native().permanent_control(
            &source,
            &target,
            colors.as_native(),
            verbose,
        );

        Ok(sanitize_control_map(graph, perturbations.as_bdd().clone()))
    }

    /// Compute the color-perturbation pairs which guarantee that the network reaches the
    /// specified target `phenotype` from any initial state, assuming the perturbation
    /// is applied indefinitely.
    ///
    /// Optionally, you can provide an `PhenotypeOscillation` type that specifies whether the attractors
    /// must fully reside in the `phenotype` set (`forbidden`), can only intersect the
    /// phenotype set but still be proper subsets (`allowed`), or must intersect the phenotype
    /// while not being subsets (`required`). Default behavior is `forbidden`,
    /// i.e., each attractor fully resides in the `phenotype` set.
    ///
    /// To reduce the search space (and speed up the computation), you can also specify an
    /// `size_limit` constraint (only perturbations that are smaller or equal will be considered).
    /// Furthermore, if `stop_when_found` is set, the method terminates early if a perturbation
    /// with robustness `1.0` is discovered (i.e., a perturbation is found that is effective for
    /// all network colors). When this option is active, other results that have been computed
    /// so far are still returned.
    ///
    /// Finally, you can specify an `initial_states` set. If specified, the resulting control
    /// strategies only work across the states reachable from this set.
    ///
    #[staticmethod]
    #[pyo3(signature = (graph, phenotype, oscillation_type = None, size_limit = None, stop_when_found = false, initial_states = None))]
    pub fn phenotype_permanent(
        py: Python,
        graph: Py<AsynchronousPerturbationGraph>,
        phenotype: &VertexSet,
        oscillation_type: Option<String>,
        size_limit: Option<usize>,
        stop_when_found: bool,
        initial_states: Option<&VertexSet>,
    ) -> PyResult<ColoredPerturbationSet> {
        let verbose = should_log(global_log_level(py)?);

        let p_type = if let Some(p_type) = oscillation_type {
            extract_phenotype_type(p_type.as_str())?
        } else {
            PhenotypeOscillationType::Forbidden
        };

        // If initial states is not set, we consider all networks states as potential initial states
        let initial_states_native = match initial_states {
            Some(x) => x.as_native().clone(),
            None => graph
                .get()
                .as_native()
                .mk_unit_colored_vertices()
                .vertices()
                .clone(),
        };

        // If the size limit is not set, we consider the largest possible size.
        let size_limit =
            size_limit.unwrap_or_else(|| graph.get().as_native().perturbable_variables().len());

        let perturbations = graph.get().as_native().ceiled_phenotype_permanent_control(
            phenotype.as_native().clone(),
            size_limit,
            p_type,
            initial_states_native,
            stop_when_found,
            verbose,
        );

        Ok(sanitize_control_map(graph, perturbations.as_bdd().clone()))
    }
}

fn extract_state(
    py: Python,
    graph: &Py<AsynchronousPerturbationGraph>,
    state: &Bound<'_, PyAny>,
) -> PyResult<ArrayBitVector> {
    if let Ok(set) = state.extract::<VertexSet>() {
        return if !set.is_singleton() {
            throw_runtime_error("The state set must be a singleton.")
        } else {
            Ok(set.as_native().iter().next().unwrap())
        };
    }
    let graph_ref = graph.borrow(py);
    let state = graph_ref.as_ref().resolve_subspace_valuation(state)?;
    let mut state_vector = ArrayBitVector::empty(graph_ref.as_ref().as_native().num_vars());

    for (k, v) in state {
        state_vector.set(k.to_index(), v);
    }

    Ok(state_vector)
}

/// Tries to "normalize" a control map into a state in which it can be safely handled by
/// `ColoredPerturbationSet`. In particular, this means that any state variable must be
/// unconstrained in cases where it is unperturbed.
pub fn sanitize_control_map(
    graph: Py<AsynchronousPerturbationGraph>,
    mut bdd: biodivine_lib_bdd::Bdd,
) -> ColoredPerturbationSet {
    let native_graph = graph.get().as_native();
    let mapping = native_graph.get_perturbation_bdd_mapping(native_graph.perturbable_variables());
    for (k, v) in mapping {
        let s_var = native_graph.as_symbolic_context().get_state_variable(k);
        let is_perturbed = bdd.var_select(v, true);
        let not_perturbed = bdd.var_select(v, false);
        let not_perturbed = not_perturbed.var_exists(s_var);
        bdd = is_perturbed.or(&not_perturbed);
    }

    ColoredPerturbationSet::mk_native(
        graph.clone(),
        GraphColoredVertices::new(bdd, native_graph.as_symbolic_context()),
    )
}
