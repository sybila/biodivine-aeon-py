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
    #[staticmethod]
    pub fn attractors(
        graph: &AsynchronousGraph,
        restrictions: &ColoredVertexSet,
    ) -> Vec<ColoredVertexSet> {
        let (states, transitions) = interleaved_transition_guided_reduction(
            graph.as_native(),
            restrictions.as_native().clone(),
        );
        let result = crate::internal::scc::algo_xie_beerel::xie_beerel_attractors(
            graph.as_native(),
            &states,
            &transitions,
        );
        result
            .into_iter()
            .map(|it| ColoredVertexSet::mk_native(graph.symbolic_context(), it))
            .collect()
    }
}
