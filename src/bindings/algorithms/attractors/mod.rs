use crate::AsNative;
use crate::bindings::algorithms::attractors::attractor_config::{
    AttractorConfigOrGraph, PyAttractorConfig,
};
use crate::bindings::lib_param_bn::argument_types::variable_id_type::VariableIdType;
use crate::bindings::lib_param_bn::symbolic::set_colored_vertex::ColoredVertexSet;
use crate::bindings::lib_param_bn::variable_id::VariableIdResolvable;
use biodivine_algo_bdd_scc::attractor::{
    InterleavedTransitionGuidedReduction, ItgrState, XieBeerelAttractors, XieBeerelState,
};
use computation_process::{Algorithm, Stateful};
use pyo3::prelude::*;

pub mod attractor_config;

#[pyclass(module = "biodivine_aeon", frozen)]
pub struct Attractors {
    _dummy: (),
}

#[pymethods]
impl Attractors {
    /// Compute a subset of the given `initial_set` set that is guaranteed to be a superset
    /// of all the attractors within the `initial_set` set.
    ///
    /// Note that the attractor property is evaluated globally, not with respect to
    /// the `initial_set`. The `initial_set` set only specifies that the result must be
    /// a subset of the input. If no `restriction` is given, the whole state space is used.
    ///
    /// Furthermore, note that the exact characterisation of the set
    /// that is retained is a bit complicated. You shouldn't assume that the result is
    /// a trap set or that it is forward/backward closed. Just that any attractor
    /// that is a subset of `restriction` is still a subset of the result.
    ///
    /// You can limit which variables are reduced using the `to_reduce` list. For example, for
    /// some larger models, it may be sufficient to select a subset of variables.
    #[staticmethod]
    #[pyo3(signature = (config, initial_set = None, to_reduce = None))]
    pub fn transition_guided_reduction(
        config: AttractorConfigOrGraph,
        initial_set: Option<&ColoredVertexSet>,
        to_reduce: Option<Vec<VariableIdType>>,
        py: Python,
    ) -> PyResult<ColoredVertexSet> {
        let py_config = PyAttractorConfig::from(config);
        let config = py_config.clone_native(py)?;

        let to_reduce = if let Some(to_reduce) = to_reduce {
            VariableIdType::resolve_collection(to_reduce, &config.graph)?
        } else {
            config.graph.variables().collect::<Vec<_>>()
        };

        let initial_set = if let Some(r) = initial_set {
            r.as_native().clone()
        } else {
            config.graph.mk_unit_colored_vertices()
        };

        cancel_this::on_python(|| {
            let state = ItgrState::new_with_variables(&config.graph, &initial_set, &to_reduce);
            let result = InterleavedTransitionGuidedReduction::run(config, state)?;

            Ok(ColoredVertexSet::mk_native(
                py_config.graph.clone_py_context(py)?,
                result,
            ))
        })
    }

    /// Perform attractor detection on the given symbolic set.
    ///
    /// This is similar to `Attractors.attractors`, but it does not perform
    /// `Attractors.transition_guided_reduction`. Instead, it directly runs the attractor
    /// detection algorithm on the `initial_set` set without any preprocessing.
    ///
    /// Note that the result is a collection of sets, such that for each set and each color
    /// holds that if the color is present in the set, the vertices of this color in the set
    /// together form an attractor. It is not guaranteed that this "decomposition" into colored
    /// sets is in some sense canonical (but the method should be deterministic). If you only
    /// care about attractor states and not individual attractors, you can simply merge all the
    /// sets together.
    #[staticmethod]
    #[pyo3(signature = (config, initial_set = None))]
    pub fn xie_beerel(
        config: AttractorConfigOrGraph,
        initial_set: Option<&ColoredVertexSet>,
        py: Python,
    ) -> PyResult<Vec<ColoredVertexSet>> {
        let py_config = PyAttractorConfig::from(config);
        let py_ctx = py_config.graph.clone_py_context(py)?;
        let config = py_config.clone_native(py)?;

        // Convert `Option<ColoredVertexSet>` to `GraphColoredVertices`
        let initial_set = if let Some(r) = initial_set {
            r.as_native().clone()
        } else {
            config.graph.mk_unit_colored_vertices()
        };

        cancel_this::on_python(|| {
            let state = XieBeerelState::from(&initial_set);
            let mut result = Vec::new();
            for attr in XieBeerelAttractors::configure(config, state).take(py_config.solution_count)
            {
                let attr = ColoredVertexSet::mk_native(py_ctx.clone(), attr?);
                result.push(attr);
            }

            Ok(result)
        })
    }

    /// Compute the (colored) attractor set of the given `AsynchronousGraph`.
    ///
    /// See `Attractors.xie_beerel`, `Attractors.transition_guided_reduction`, and
    /// `AttractorConfig` for relevant documentation.
    #[staticmethod]
    #[pyo3(signature = (config, initial_set = None, to_reduce = None))]
    pub fn attractors(
        config: AttractorConfigOrGraph,
        initial_set: Option<&ColoredVertexSet>,
        to_reduce: Option<Vec<VariableIdType>>,
        py: Python,
    ) -> PyResult<Vec<ColoredVertexSet>> {
        let py_config = PyAttractorConfig::from(config);
        let py_ctx = py_config.graph.clone_py_context(py)?;
        let config = py_config.clone_native(py)?;

        let to_reduce = if let Some(to_reduce) = to_reduce {
            VariableIdType::resolve_collection(to_reduce, &config.graph)?
        } else {
            config.graph.variables().collect::<Vec<_>>()
        };

        let initial_set = if let Some(r) = initial_set {
            r.as_native().clone()
        } else {
            config.graph.mk_unit_colored_vertices()
        };

        cancel_this::on_python(|| {
            let state = ItgrState::new_with_variables(&config.graph, &initial_set, &to_reduce);
            let result = InterleavedTransitionGuidedReduction::run(config.clone(), state)?;

            let state = XieBeerelState::from(&result);
            let mut result = Vec::new();
            for attr in XieBeerelAttractors::configure(config, state).take(py_config.solution_count)
            {
                let attr = ColoredVertexSet::mk_native(py_ctx.clone(), attr?);
                result.push(attr);
            }

            Ok(result)
        })
    }
}
