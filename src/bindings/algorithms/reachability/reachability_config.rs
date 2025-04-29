use std::{collections::HashSet, time::Duration};

use biodivine_lib_param_bn::{
    symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph},
    BooleanNetwork, VariableId,
};
use macros::Config;
use pyo3::{pyclass, pymethods, PyResult};

use crate::{
    bindings::{
        algorithms::{
            cancellation::{
                tokens::{CancelTokenPython, CancelTokenTimer},
                CancellationHandler,
            },
            configurable::Config,
            graph_representation::PyGraphRepresentation,
            reachability::ReachabilityError,
        },
        lib_param_bn::{
            symbolic::set_colored_vertex::ColoredVertexSet,
            variable_id::VariableId as VariableIdBinding,
        },
    },
    AsNative as _,
};

/// A configuration struct for the [Reachability] algorithms.
#[pyclass(module = "biodivine_aeon", frozen)]
#[derive(Clone, Config)]
pub struct ReachabilityConfig {
    /// The symbolic graph that will be used to compute the successors and predecessors of
    /// individual states.
    pub graph: SymbolicAsyncGraph,

    /// Restricts the reachability operation to the given set of vertices. This also includes
    /// edges! For example, if a vertex `x` only has outgoing edges into vertices outside the
    /// `subgraph`, it would be considered a fixed-point.
    ///
    /// The initial set must be a subset of the subgraph vertices.
    ///
    /// Default: `None`.
    pub subgraph: Option<GraphColoredVertices>,

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
    pub variables: HashSet<VariableId>,

    /// A `CancellationHandler` that can be used to stop the algorithm externally.
    ///
    /// Default: [CancelTokenNever].
    pub cancellation: Box<dyn CancellationHandler>,

    /// The maximum BDD size of the reachable set.
    ///
    /// Note that the algorithm can use other auxiliary BDDs that do not
    /// count towards this limit.
    ///
    /// Default: `usize::MAX`.
    pub bdd_size_limit: usize,

    /// The maximum number of steps that the algorithm can take before terminating.
    ///
    /// A step is a single extension or reduction of the reachable set of vertices.
    ///
    /// Default: `usize::MAX`.
    pub steps_limit: usize,
}

impl From<SymbolicAsyncGraph> for ReachabilityConfig {
    /// Create a new "default" [ReachabilityConfig] from the given [SymbolicAsyncGraph].
    fn from(graph: SymbolicAsyncGraph) -> Self {
        ReachabilityConfig {
            variables: HashSet::from_iter(graph.variables()),
            subgraph: None,
            cancellation: Default::default(),
            bdd_size_limit: usize::MAX,
            steps_limit: usize::MAX,
            graph,
        }
    }
}

impl TryFrom<&BooleanNetwork> for ReachabilityConfig {
    type Error = ReachabilityError;

    /// Create a new "default" [ReachabilityConfig] from the given [BooleanNetwork].
    fn try_from(boolean_network: &BooleanNetwork) -> Result<Self, Self::Error> {
        let graph =
            SymbolicAsyncGraph::new(boolean_network).map_err(ReachabilityError::CreationFailed)?;

        Ok(Self::from(graph))
    }
}

impl ReachabilityConfig {
    /// Update the `subgraph` property, automatically wrapping the [GraphColoredVertices] in
    /// `Some`.
    pub fn with_subgraph(mut self, subgraph: GraphColoredVertices) -> Self {
        self.subgraph = Some(subgraph);
        self
    }

    /// Update the `variables` property.
    pub fn with_variables(mut self, variables: HashSet<VariableId>) -> Self {
        self.variables = variables;
        self
    }

    /// Update the `bdd_size_limit` property.
    pub fn with_bdd_size_limit(mut self, bdd_size_limit: usize) -> Self {
        self.bdd_size_limit = bdd_size_limit;
        self
    }

    /// Update the `steps_limit` property.
    pub fn with_steps_limit(mut self, steps_limit: usize) -> Self {
        self.steps_limit = steps_limit;
        self
    }
}

impl ReachabilityConfig {
    /// Return the variables sorted in ascending order.
    pub fn sorted_variables(&self) -> Vec<VariableId> {
        let mut variables = Vec::from_iter(self.variables.clone());
        variables.sort();
        variables
    }
}

// TODO: finalize - make this optional with a feature flag
/// These methods are Python facing wrappers of native mehtods and thus should not be used from
/// within Rust. When working with [ReachibilityConfig] from Rust, use methods without the python_
/// prefix.
#[pymethods]
impl ReachabilityConfig {
    #[new]
    #[pyo3(signature = (graph_representation, subgraph = None, variables = None, time_limit_millis = None, bdd_size_limit = None, steps_limit = None))]
    pub fn python_new(
        graph_representation: PyGraphRepresentation,
        subgraph: Option<&ColoredVertexSet>,
        variables: Option<HashSet<VariableIdBinding>>,
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

    #[staticmethod]
    #[pyo3(name = "from")]
    pub fn python_from(graph_representation: PyGraphRepresentation) -> PyResult<Self> {
        Ok(ReachabilityConfig::try_from(graph_representation)?)
    }

    #[pyo3(name = "with_subgraph")]
    pub fn python_with_subgraph(&self, subgraph: &ColoredVertexSet) -> Self {
        self.clone().with_subgraph(subgraph.as_native().clone())
    }

    #[pyo3(name = "with_variables")]
    pub fn python_with_variables(&self, variables: HashSet<VariableIdBinding>) -> Self {
        self.clone()
            .with_variables(variables.iter().map(|var| *var.as_native()).collect())
    }

    // TODO: if we ever move away from abi3-py37, use Duration as an argument
    #[pyo3(name = "with_time_limit")]
    pub fn python_with_time_limit(&self, duration_in_millis: u64) -> Self {
        self.clone()
            .with_cancellation(CancelTokenPython::with_inner(CancelTokenTimer::new(
                Duration::from_millis(duration_in_millis),
            )))
    }

    #[pyo3(name = "with_bdd_size_limit")]
    pub fn python_with_bdd_size_limit(&self, bdd_size_limit: usize) -> Self {
        self.clone().with_bdd_size_limit(bdd_size_limit)
    }

    #[pyo3(name = "with_steps_limit")]
    pub fn python_with_steps_limit(&self, steps_limit: usize) -> Self {
        self.clone().with_steps_limit(steps_limit)
    }
}
