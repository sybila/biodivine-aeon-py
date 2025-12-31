use crate::AsNative;
use crate::bindings::algorithms::graph_representation::PyAsynchronousGraphType;
use crate::bindings::algorithms::reachability::reachability_config::{
    PyReachabilityConfig, ReachabilityConfigOrGraph,
};
use crate::bindings::lib_param_bn::symbolic::asynchronous_graph::AsynchronousGraph;
use crate::bindings::lib_param_bn::symbolic::set_colored_vertex::ColoredVertexSet;
use biodivine_algo_bdd_scc::reachability::{
    BackwardReachability, ForwardReachability, ReachabilityState,
};
use biodivine_algo_bdd_scc::trapping::{BackwardTrap, ForwardTrap};
use computation_process::Algorithm;
use pyo3::{Py, PyResult, Python, pyclass, pymethods};

pub mod reachability_config;

/// An "algorithm object" that facilitates reachability procedures, i.e., iterative computation
/// of successors (or predecessors) of a particular symbolic set,
/// such that the successors/predecessors are then added to or removed from the set.
#[pyclass(module = "biodivine_aeon", frozen)]
pub struct Reachability {
    _dummy: (),
}

#[pymethods]
impl Reachability {
    /// Compute the greatest *superset* of vertices forward reachable (i.e., forward-closed)
    /// from the given `initial_set`.
    ///
    /// See `ReachabilityConfig` for more information regarding algorithm configuration.
    #[staticmethod]
    pub fn forward_superset(
        py: Python,
        config: ReachabilityConfigOrGraph,
        initial_set: &ColoredVertexSet,
    ) -> PyResult<ColoredVertexSet> {
        cancel_this::on_python(|| {
            let config = PyReachabilityConfig::from(config);
            let state = ReachabilityState::from(initial_set.as_native());
            let result = ForwardReachability::run(config.clone_native(py)?, state)?;
            let symbolic_context = config.graph.clone_py_context(py)?;
            Ok(ColoredVertexSet::mk_native(symbolic_context, result))
        })
    }

    /// Compute the greatest *superset* of vertices backward reachable (i.e., backward-closed)
    /// from the given `initial_set`.
    ///
    /// See `ReachabilityConfig` for more information regarding algorithm configuration.
    #[staticmethod]
    pub fn backward_superset(
        py: Python,
        config: ReachabilityConfigOrGraph,
        initial_set: &ColoredVertexSet,
    ) -> PyResult<ColoredVertexSet> {
        cancel_this::on_python(|| {
            let config = PyReachabilityConfig::from(config);
            let state = ReachabilityState::from(initial_set.as_native());
            let result = BackwardReachability::run(config.clone_native(py)?, state)?;
            let symbolic_context = config.graph.clone_py_context(py)?;
            Ok(ColoredVertexSet::mk_native(symbolic_context, result))
        })
    }

    /// Compute the greatest *subset* of the given `initial_set` that is forward-closed (i.e.,
    /// eliminate all states that can reach outside the set).
    ///
    /// **The result of this operation is also sometimes called the trap set.**
    ///
    /// See `ReachabilityConfig` for more information regarding algorithm configuration.
    #[staticmethod]
    pub fn forward_subset(
        py: Python,
        config: ReachabilityConfigOrGraph,
        initial_set: &ColoredVertexSet,
    ) -> PyResult<ColoredVertexSet> {
        cancel_this::on_python(|| {
            let config = PyReachabilityConfig::from(config);
            let state = ReachabilityState::from(initial_set.as_native());
            let result = ForwardTrap::run(config.clone_native(py)?, state)?;
            let symbolic_context = config.graph.clone_py_context(py)?;
            Ok(ColoredVertexSet::mk_native(symbolic_context, result))
        })
    }

    /// Compute the greatest *subset* of the given `initial_set` that is backward-closed (i.e.,
    /// eliminate all states that can be reached from outside the set).
    ///
    /// See `ReachabilityConfig` for more information regarding algorithm configuration.
    #[staticmethod]
    pub fn backward_subset(
        py: Python,
        config: ReachabilityConfigOrGraph,
        initial_set: &ColoredVertexSet,
    ) -> PyResult<ColoredVertexSet> {
        cancel_this::on_python(|| {
            let config = PyReachabilityConfig::from(config);
            let state = ReachabilityState::from(initial_set.as_native());
            let result = BackwardTrap::run(config.clone_native(py)?, state)?;
            let symbolic_context = config.graph.clone_py_context(py)?;
            Ok(ColoredVertexSet::mk_native(symbolic_context, result))
        })
    }

    /// **Deprecated**: Use `ReachabilityComp.forward_superset()` instead.
    ///
    /// Compute the (colored) set of vertices that are forward-reachable from the given
    /// initial set.
    ///
    /// Note that this method should be faster than just iteratively calling
    /// `AsynchronousGraph.post` as it is based on the saturation technique.
    #[staticmethod]
    pub fn reach_fwd(
        py: Python,
        graph: Py<AsynchronousGraph>,
        initial: &ColoredVertexSet,
    ) -> PyResult<ColoredVertexSet> {
        let config = ReachabilityConfigOrGraph::Graph(PyAsynchronousGraphType::Graph(graph));
        Self::forward_superset(py, config, initial)
    }

    /// **Deprecated**: Use `Reachability.backward_superset()` instead.
    ///
    /// Compute the (colored) set of vertices that are backward-reachable from the given
    /// initial set.
    ///
    /// Note that this method should be faster than just iteratively calling
    /// `AsynchronousGraph.pre` as it is based on the saturation technique.
    #[staticmethod]
    pub fn reach_bwd(
        py: Python,
        graph: Py<AsynchronousGraph>,
        initial: &ColoredVertexSet,
    ) -> PyResult<ColoredVertexSet> {
        let config = ReachabilityConfigOrGraph::Graph(PyAsynchronousGraphType::Graph(graph));
        Self::backward_superset(py, config, initial)
    }
}
