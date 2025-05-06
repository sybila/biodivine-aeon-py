use std::time::Duration;

use biodivine_lib_param_bn::{symbolic_async_graph::SymbolicAsyncGraph, BooleanNetwork};
use macros::Config;
use pyo3::{pyclass, pymethods, PyResult};

use crate::bindings::algorithms::{
    cancellation::{
        tokens::{CancelTokenPython, CancelTokenTimer},
        CancellationHandler,
    },
    configurable::Config,
    graph_representation::PyGraphRepresentation,
    percolation::percolation_error::PercolationError,
};

#[pyclass(module = "biodivine_aeon", frozen)]
#[derive(Clone, Config)]
pub struct PercolationConfig {
    /// The symbolic graph whose variables will be used for subspace percolation.
    pub graph: SymbolicAsyncGraph,

    /// A `CancellationHandler` that can be used to stop the algorithm externally.
    ///
    /// Default: [CancelTokenNever].
    pub cancellation: Box<dyn CancellationHandler>,
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
    #[pyo3(signature = (graph_representation, time_limit_millis = None))]
    pub fn python_new(
        graph_representation: PyGraphRepresentation,
        time_limit_millis: Option<u64>,
    ) -> PyResult<Self> {
        let mut config = PercolationConfig::try_from(graph_representation)?;

        if let Some(millis) = time_limit_millis {
            config = config.with_cancellation(CancelTokenPython::with_inner(CancelTokenTimer::new(
                Duration::from_millis(millis),
            )))
        } else {
            config = config.with_cancellation(CancelTokenPython::default());
        }

        Ok(config)
    }

    #[staticmethod]
    #[pyo3(name = "create_from")]
    pub fn python_create_from(graph_representation: PyGraphRepresentation) -> PyResult<Self> {
        Ok(PercolationConfig::try_from(graph_representation)?)
    }

    // TODO: if we ever move away from abi3-py37, use Duration as an argument
    #[pyo3(name = "with_time_limit")]
    pub fn python_with_time_limit(&self, millis: u64) -> Self {
        self.clone()
            .with_cancellation(CancelTokenPython::with_inner(CancelTokenTimer::new(
                Duration::from_millis(millis),
            )))
    }
}
