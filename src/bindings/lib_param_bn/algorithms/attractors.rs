use crate::bindings::lib_param_bn::symbolic::asynchronous_graph::AsynchronousGraph;
use crate::bindings::lib_param_bn::symbolic::set_colored_vertex::ColoredVertexSet;
use crate::bindings::lib_param_bn::NetworkVariableContext;
use crate::internal::scc::algo_interleaved_transition_guided_reduction::interleaved_transition_guided_reduction;
use crate::{global_log_level, AsNative};
use pyo3::prelude::*;
use pyo3::types::PyList;

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
        to_reduce: Option<&Bound<'_, PyList>>,
        py: Python,
    ) -> PyResult<ColoredVertexSet> {
        let mut to_reduce_native = Vec::new();
        if let Some(to_reduce) = to_reduce {
            for x in to_reduce {
                to_reduce_native.push(graph.resolve_network_variable(&x)?);
            }
        } else {
            to_reduce_native.extend(graph.as_native().variables());
        }

        let restriction_native = if let Some(r) = restriction {
            r.as_native().clone()
        } else {
            graph.as_native().mk_unit_colored_vertices()
        };

        let (states, _) = interleaved_transition_guided_reduction(
            graph.as_native(),
            restriction_native,
            &to_reduce_native,
            global_log_level(py)?,
        )?;

        Ok(ColoredVertexSet::mk_native(
            graph.symbolic_context(),
            states,
        ))
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
        py: Python,
    ) -> PyResult<Vec<ColoredVertexSet>> {
        let restriction_native = if let Some(r) = restriction {
            r.as_native().clone()
        } else {
            graph.as_native().mk_unit_colored_vertices()
        };

        let transitions = graph.as_native().variables().collect::<Vec<_>>();
        let result = crate::internal::scc::algo_xie_beerel::xie_beerel_attractors(
            graph.as_native(),
            &restriction_native,
            &transitions,
            global_log_level(py)?,
        )?;
        Ok(result
            .into_iter()
            .map(|it| ColoredVertexSet::mk_native(graph.symbolic_context(), it))
            .collect())
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
        to_reduce: Option<&Bound<'_, PyList>>,
        py: Python,
    ) -> PyResult<Vec<ColoredVertexSet>> {
        let reduced = Self::transition_guided_reduction(graph, restriction, to_reduce, py)?;
        Self::xie_beerel(graph, Some(&reduced), py)
    }
}
