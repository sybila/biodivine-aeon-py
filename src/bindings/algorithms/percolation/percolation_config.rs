use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
use pyo3::{pyclass, pymethods};

use crate::bindings::algorithms::cancellation::CancellationHandler;

#[pyclass(module = "biodivine_aeon", frozen)]
#[derive(Clone)]
pub struct PercolationConfig {
    pub graph: SymbolicAsyncGraph,

    /// A `CancellationHandler` that can be used to stop the algorithm externally.
    ///
    /// Default: [CancelTokenNever].
    pub cancellation: Box<dyn CancellationHandler>,
}

impl PercolationConfig {
    /// Create a new "default" [PercolationConfig] for the given [SymbolicAsyncGraph].
    pub fn with_graph(graph: SymbolicAsyncGraph) -> Self {
        PercolationConfig {
            graph,
            cancellation: Default::default(),
        }
    }

    /// Update the `cancellation` property
    pub fn with_cancellation(mut self, cancellation: Box<dyn CancellationHandler>) -> Self {
        self.cancellation = cancellation;
        self
    }
}

#[pymethods]
impl PercolationConfig {
    // TODO: ohtenkay
}
