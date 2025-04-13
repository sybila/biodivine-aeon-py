use std::time::Duration;

use biodivine_lib_param_bn::{symbolic_async_graph::SymbolicAsyncGraph, BooleanNetwork};
use pyo3::{pyclass, pymethods, PyResult};

use crate::{
    bindings::{
        algorithms::{
            cancellation::{
                tokens::{CancelTokenPython, CancelTokenTimer},
                CancellationHandler,
            },
            configurable::Config,
            percolation::percolation_error::PercolationError,
        },
        lib_param_bn::{
            boolean_network::BooleanNetwork as BooleanNetworkBinding,
            symbolic::asynchronous_graph::AsynchronousGraph,
        },
    },
    AsNative,
};

#[pyclass(module = "biodivine_aeon", frozen)]
#[derive(Clone)]
pub struct PercolationConfig {
    /// The symbolic graph whose variables will be used for subspace percolation.
    pub graph: SymbolicAsyncGraph,

    /// A `CancellationHandler` that can be used to stop the algorithm externally.
    ///
    /// Default: [CancelTokenNever].
    pub cancellation: Box<dyn CancellationHandler>,
}

impl Config for PercolationConfig {
    fn cancellation(&self) -> &dyn CancellationHandler {
        self.cancellation.as_ref()
    }

    fn set_cancellation(&mut self, cancellation: Box<dyn CancellationHandler>) {
        self.cancellation = cancellation;
    }
}

impl From<SymbolicAsyncGraph> for PercolationConfig {
    /// Create a new "default" [PercolationConfig] from the given [SymbolicAsyncGraph].
    fn from(graph: SymbolicAsyncGraph) -> Self {
        PercolationConfig {
            graph,
            cancellation: Default::default(),
        }
    }
}

impl TryFrom<&BooleanNetwork> for PercolationConfig {
    type Error = PercolationError;

    /// Create a new "default" [PercolationConfig] from the given [BooleanNetwork].
    fn try_from(boolean_network: &BooleanNetwork) -> Result<Self, Self::Error> {
        let graph =
            SymbolicAsyncGraph::new(boolean_network).map_err(PercolationError::CreationFailed)?;

        Ok(PercolationConfig::from(graph))
    }
}

#[pymethods]
impl PercolationConfig {
    #[new]
    #[pyo3(signature = (graph, time_limit_millis = None))]
    pub fn python_new(graph: &AsynchronousGraph, time_limit_millis: Option<u64>) -> Self {
        let mut config = PercolationConfig::from(graph.as_native().clone());

        if let Some(millis) = time_limit_millis {
            config = config.with_cancellation(CancelTokenPython::with_inner(CancelTokenTimer::new(
                Duration::from_millis(millis),
            )))
        }

        config
    }

    #[staticmethod]
    #[pyo3(name = "from_boolean_network")]
    pub fn python_from_boolean_network(boolean_network: &BooleanNetworkBinding) -> PyResult<Self> {
        Ok(PercolationConfig::try_from(boolean_network.as_native())?)
    }

    #[staticmethod]
    #[pyo3(name = "from_graph")]
    pub fn python_from_graph(graph: &AsynchronousGraph) -> Self {
        PercolationConfig::from(graph.as_native().clone())
    }

    #[pyo3(name = "with_time_limit")]
    pub fn python_with_time_limit(&self, millis: u64) -> Self {
        self.clone()
            .with_cancellation(CancelTokenPython::with_inner(CancelTokenTimer::new(
                Duration::from_millis(millis),
            )))
    }
}
