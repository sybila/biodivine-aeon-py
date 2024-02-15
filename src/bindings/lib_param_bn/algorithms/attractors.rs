use crate::bindings::lib_param_bn::symbolic::asynchronous_graph::AsynchronousGraph;
use crate::bindings::lib_param_bn::symbolic::set_colored_vertex::ColoredVertexSet;
use crate::internal::scc::algo_interleaved_transition_guided_reduction::interleaved_transition_guided_reduction;
use crate::AsNative;
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
    /// the `restriction` set. Furthermore, note that the exact characterisation of the set
    /// that are retained is a bit complicated. You shouldn't assume that the result is
    /// a trap set or that it is forward/backward closed. Just that any attractor
    /// is still a subset of the result.
    #[staticmethod]
    pub fn transition_guided_reduction(
        graph: &AsynchronousGraph,
        restriction: &ColoredVertexSet,
    ) -> PyResult<ColoredVertexSet> {
        let (states, _) = interleaved_transition_guided_reduction(
            graph.as_native(),
            restriction.as_native().clone(),
        )?;
        Ok(ColoredVertexSet::mk_native(
            graph.symbolic_context(),
            states,
        ))
    }

    /// Perform attractor detection on the given symbolic set.
    ///
    /// This is similar to `Attractors.attractors`, but it does not perform
    /// `Attractors.transition_guilded_reduction`, it directly runs the attractor detection
    /// algorithm without any preprocessing.
    #[staticmethod]
    pub fn xie_beerel(
        graph: &AsynchronousGraph,
        restriction: &ColoredVertexSet,
    ) -> PyResult<Vec<ColoredVertexSet>> {
        let transitions = graph.as_native().variables().collect::<Vec<_>>();
        let result = crate::internal::scc::algo_xie_beerel::xie_beerel_attractors(
            graph.as_native(),
            restriction.as_native(),
            &transitions,
        )?;
        Ok(result
            .into_iter()
            .map(|it| ColoredVertexSet::mk_native(graph.symbolic_context(), it))
            .collect())
    }

    /// Compute the (colored) attractor set of the given `AsynchronousGraph`.
    ///
    /// When the network is fully specified, the result is a collection of vertex sets
    /// where each set is one attractor. When the network is partially unknown, the resulting
    /// sets are colored, such that for every color, the vertices in the set form an attractor.
    /// However, technically multiple attractors can be returned in one such set (up to one for
    /// each graph color). Naturally, when the number of attractors is not the same for all
    /// colors, some of these sets will only contain a subset of colors.
    #[staticmethod]
    pub fn attractors(
        graph: &AsynchronousGraph,
        restrictions: &ColoredVertexSet,
    ) -> PyResult<Vec<ColoredVertexSet>> {
        let (states, transitions) = interleaved_transition_guided_reduction(
            graph.as_native(),
            restrictions.as_native().clone(),
        )?;
        let result = crate::internal::scc::algo_xie_beerel::xie_beerel_attractors(
            graph.as_native(),
            &states,
            &transitions,
        )?;
        Ok(result
            .into_iter()
            .map(|it| ColoredVertexSet::mk_native(graph.symbolic_context(), it))
            .collect())
    }
}
