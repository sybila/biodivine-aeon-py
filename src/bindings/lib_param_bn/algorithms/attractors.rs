use crate::AsNative;
use crate::bindings::lib_param_bn::argument_types::variable_id_type::VariableIdType;
use crate::bindings::lib_param_bn::symbolic::asynchronous_graph::AsynchronousGraph;
use crate::bindings::lib_param_bn::symbolic::set_colored_vertex::ColoredVertexSet;
use crate::bindings::lib_param_bn::variable_id::VariableIdResolvable;
use biodivine_algo_bdd_scc::attractor::{
    AttractorConfig, InterleavedTransitionGuidedReduction, ItgrState, XieBeerelAttractors,
    XieBeerelState,
};
use computation_process::{Algorithm, Stateful};
use pyo3::prelude::*;

#[pyclass(module = "biodivine_aeon", frozen)]
pub struct Attractors {
    _dummy: (),
}

#[pymethods]
impl Attractors {
    /// Compute a subset of the given `restriction` set that is guaranteed to be a superset
    /// of all the attractors within the `restriction` set.
    ///
    /// Note that the attractor property is evaluated globally, not with respect to
    /// the `restriction` set. The `restriction` set only specifies that the result must be
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
    #[pyo3(signature = (graph, restriction = None, to_reduce = None))]
    pub fn transition_guided_reduction(
        graph: &AsynchronousGraph,
        restriction: Option<&ColoredVertexSet>,
        to_reduce: Option<Vec<VariableIdType>>,
    ) -> PyResult<ColoredVertexSet> {
        // Convert `Option<PyList<?>>` to `Vec<VariableId>`
        let to_reduce = if let Some(to_reduce) = to_reduce {
            VariableIdType::resolve_collection(to_reduce, graph.as_native())?
        } else {
            graph.as_native().variables().collect::<Vec<_>>()
        };

        // Convert `Option<ColoredVertexSet>` to `GraphColoredVertices`
        let restriction_native = if let Some(r) = restriction {
            r.as_native().clone()
        } else {
            graph.as_native().mk_unit_colored_vertices()
        };

        cancel_this::on_python(|| {
            let config = AttractorConfig::new(graph.as_native().clone());
            let state =
                ItgrState::new_with_variables(graph.as_native(), &restriction_native, &to_reduce);
            let result = InterleavedTransitionGuidedReduction::run(config, state)?;

            Ok(ColoredVertexSet::mk_native(
                graph.symbolic_context(),
                result,
            ))
        })
    }

    /// Perform attractor detection on the given symbolic set.
    ///
    /// This is similar to `Attractors.attractors`, but it does not perform
    /// `Attractors.transition_guilded_reduction`. Instead, it directly runs the attractor
    /// detection algorithm on the `restriction` set without any preprocessing.
    ///
    /// Note that the result is a collection of sets, such that for each set and each color
    /// holds that if the color is present in the set, the vertices of this color in the set
    /// together form an attractor. It is not guaranteed that this "decomposition" into colored
    /// sets is in some sense canonical (but the method should be deterministic). If you only
    /// care about attractor states and not individual attractors, you can simply merge all the
    /// sets together.
    #[staticmethod]
    #[pyo3(signature = (graph, restriction = None))]
    pub fn xie_beerel(
        graph: &AsynchronousGraph,
        restriction: Option<&ColoredVertexSet>,
    ) -> PyResult<Vec<ColoredVertexSet>> {
        // Convert `Option<ColoredVertexSet>` to `GraphColoredVertices`
        let restriction_native = if let Some(r) = restriction {
            r.as_native().clone()
        } else {
            graph.as_native().mk_unit_colored_vertices()
        };

        cancel_this::on_python(|| {
            let config = AttractorConfig::new(graph.as_native().clone());
            let state = XieBeerelState::from(&restriction_native);
            let mut result = Vec::new();
            for attr in XieBeerelAttractors::configure(config, state) {
                let attr = ColoredVertexSet::mk_native(graph.symbolic_context(), attr?);
                result.push(attr);
            }

            Ok(result)
        })
    }

    /// Compute the (colored) attractor set of the given `AsynchronousGraph`.
    ///
    /// See `Attractors.xie_beerel` and `Attractors.transition_guided_reduction` for relevant
    /// documentation.
    #[staticmethod]
    #[pyo3(signature = (graph, restriction = None, to_reduce = None))]
    pub fn attractors(
        graph: &AsynchronousGraph,
        restriction: Option<&ColoredVertexSet>,
        to_reduce: Option<Vec<VariableIdType>>,
    ) -> PyResult<Vec<ColoredVertexSet>> {
        let reduced = Self::transition_guided_reduction(graph, restriction, to_reduce)?;
        Self::xie_beerel(graph, Some(&reduced))
    }
}
