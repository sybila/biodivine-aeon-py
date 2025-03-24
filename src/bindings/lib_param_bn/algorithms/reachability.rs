use crate::bindings::lib_param_bn::symbolic::asynchronous_graph::AsynchronousGraph;
use crate::bindings::lib_param_bn::symbolic::set_colored_vertex::ColoredVertexSet;
use crate::{global_log_level, AsNative};
use pyo3::prelude::*;

// TODO: finalize - delete this file

/// An "algorithm object" that facilitates reachability procedures, i.e. iterative computation
/// of successors (or predecessors) of a particular symbolic set.
#[pyclass(module = "biodivine_aeon", frozen)]
pub struct Reachability {
    _dummy: (),
}

#[pymethods]
impl Reachability {
    /// Compute the (colored) set of vertices that are forward-reachable from the given
    /// initial set.
    ///
    /// Note that this method should be faster than just iteratively calling
    /// `AsynchronousGraph.post` as it is based on the saturation technique.
    #[staticmethod]
    pub fn reach_fwd(
        py: Python,
        graph: &AsynchronousGraph,
        initial: &ColoredVertexSet,
    ) -> PyResult<ColoredVertexSet> {
        biodivine_lib_param_bn::symbolic_async_graph::reachability::Reachability::_reach_basic_saturation(
            graph.as_native(),
            initial.as_native(),
            |g, s, v| g.var_post_out(v, s),
            global_log_level(py)?,
            &|| py.check_signals(),
        )
        .map(|it| ColoredVertexSet::mk_native(graph.symbolic_context(), it))
    }

    /// Compute the (colored) set of vertices that are backward-reachable from the given
    /// initial set.
    ///
    /// Note that this method should be faster than just iteratively calling
    /// `AsynchronousGraph.pre` as it is based on the saturation technique.
    #[staticmethod]
    pub fn reach_bwd(
        py: Python,
        graph: &AsynchronousGraph,
        initial: &ColoredVertexSet,
    ) -> PyResult<ColoredVertexSet> {
        biodivine_lib_param_bn::symbolic_async_graph::reachability::Reachability::_reach_basic_saturation(
            graph.as_native(),
            initial.as_native(),
            |g, s, v| g.var_pre_out(v, s),
            global_log_level(py)?,
            &|| py.check_signals(),
        )
        .map(|it| ColoredVertexSet::mk_native(graph.symbolic_context(), it))
    }
}
