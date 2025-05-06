use biodivine_lib_param_bn::{symbolic_async_graph::SymbolicAsyncGraph, BooleanNetwork};
use macros::Config;
use pyo3::pyclass;

use crate::bindings::algorithms::{cancellation::CancellationHandler, configurable::Config};

use super::PercolationError;

/// A configuration struct for the [Percolation] algorithm.
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

        Ok(Self::from(graph))
    }
}
