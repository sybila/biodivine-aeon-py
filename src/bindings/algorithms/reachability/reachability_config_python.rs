use std::{collections::HashSet, time::Duration};

use pyo3::{PyResult, pymethods};

use crate::{
    AsNative as _,
    bindings::{
        algorithms::{
            graph_representation::PyGraphRepresentation, token_python::CancelTokenPython,
        },
        lib_param_bn::{symbolic::set_colored_vertex::ColoredVertexSet, variable_id::VariableId},
    },
    internal::algorithms::{
        cancellation::CancelTokenTimer, configurable::Config as _, reachability::ReachabilityConfig,
    },
};

/// These methods are Python facing wrappers of native methods and thus should not be used from
/// within Rust.
#[pymethods]
impl ReachabilityConfig {
    #[new]
    #[pyo3(signature = (graph_representation, subgraph = None, variables = None, time_limit_millis = None, bdd_size_limit = None, steps_limit = None))]
    pub fn python_new(
        graph_representation: PyGraphRepresentation,
        subgraph: Option<&ColoredVertexSet>,
        variables: Option<HashSet<VariableId>>,
        time_limit_millis: Option<u64>,
        bdd_size_limit: Option<usize>,
        steps_limit: Option<usize>,
    ) -> PyResult<Self> {
        let mut config = ReachabilityConfig::try_from(graph_representation)?;

        if let Some(subgraph) = subgraph {
            config = config.with_subgraph(subgraph.as_native().clone())
        }

        if let Some(variables) = variables {
            config = config.with_variables(variables.iter().map(|var| *var.as_native()).collect())
        }

        if let Some(millis) = time_limit_millis {
            config = config.with_cancellation(CancelTokenPython::with_inner(CancelTokenTimer::new(
                Duration::from_millis(millis),
            )))
        }

        if let Some(limit) = bdd_size_limit {
            config = config.with_bdd_size_limit(limit)
        }

        if let Some(limit) = steps_limit {
            config = config.with_steps_limit(limit)
        }

        Ok(config)
    }

    /// Create a new `ReachabilityConfig` from the given `AsynchronousGraph` or `BooleanNetwork`,
    /// with otherwise default configuration.
    #[staticmethod]
    #[pyo3(name = "create_from")]
    pub fn python_create_from(graph_representation: PyGraphRepresentation) -> PyResult<Self> {
        Ok(ReachabilityConfig::try_from(graph_representation)?)
    }

    /// Restricts the reachability operation to the given set of vertices. This also includes
    /// edges! For example, if a vertex `x` only has outgoing edges into vertices outside the
    /// `subgraph`, it would be considered a fixed-point.
    ///
    /// The initial set must be a subset of the subgraph vertices.
    ///
    /// Default: `None`.
    #[pyo3(name = "with_subgraph")]
    pub fn python_with_subgraph(&self, subgraph: &ColoredVertexSet) -> Self {
        self.clone().with_subgraph(subgraph.as_native().clone())
    }

    /// Specifies the set of variables that can be updated by the reachability process.
    /// Remaining variables stay constant, because they are never updated.
    ///
    /// This can be used to implement "reachability within a subspace" that is faster than
    /// providing a `subgraph`, since the variables that are constant in the subspace never
    /// need to be updated. Alternatively, this can be used for various "multi-stage"
    /// schemes, for example to start with only a small component of the whole network and
    /// then gradually expand to the whole variable set.
    ///
    /// Default: `graph.network_variables()`.
    #[pyo3(name = "with_variables")]
    pub fn python_with_variables(&self, variables: HashSet<VariableId>) -> Self {
        self.clone()
            .with_variables(variables.iter().map(|var| *var.as_native()).collect())
    }

    /// Sets a time limit for the reachability operation, in milliseconds.
    ///
    /// Default: no time limit.
    // TODO: if we ever move away from abi3-py37, use Duration as an argument
    #[pyo3(name = "with_time_limit")]
    pub fn python_with_time_limit(&self, duration_in_millis: u64) -> Self {
        self.clone()
            .with_cancellation(CancelTokenPython::with_inner(CancelTokenTimer::new(
                Duration::from_millis(duration_in_millis),
            )))
    }

    /// The maximum BDD size of the reachable set.
    ///
    /// Note that the algorithm can use other auxiliary BDDs that do not
    /// count towards this limit.
    ///
    /// Default: `usize::MAX`.
    #[pyo3(name = "with_bdd_size_limit")]
    pub fn python_with_bdd_size_limit(&self, bdd_size_limit: usize) -> Self {
        self.clone().with_bdd_size_limit(bdd_size_limit)
    }

    /// The maximum number of steps that the algorithm can take before terminating.
    ///
    /// A step is a single extension or reduction of the reachable set of vertices.
    ///
    /// Default: `usize::MAX`.
    #[pyo3(name = "with_steps_limit")]
    pub fn python_with_steps_limit(&self, steps_limit: usize) -> Self {
        self.clone().with_steps_limit(steps_limit)
    }
}
