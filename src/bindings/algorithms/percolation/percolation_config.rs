use biodivine_lib_param_bn::{symbolic_async_graph::SymbolicAsyncGraph, VariableId};
use pyo3::{pyclass, pymethods};

use crate::bindings::algorithms::cancellation::CancellationHandler;

#[pyclass(module = "biodivine_aeon", frozen)]
#[derive(Clone)]
pub struct PercolationConfig {
    pub graph: SymbolicAsyncGraph,
    // TODO: ohtenkay - is there a default value for this? if not create an empty vector and check
    // for empty? no default value, this will be a parameter to the algorithm, remove from here
    pub subspace: Vec<(VariableId, bool)>,

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
            subspace: vec![],
            cancellation: Default::default(),
        }
    }

    /// Update the `subspace` property
    pub fn with_subspace(mut self, subspace: Vec<(VariableId, bool)>) -> Self {
        self.subspace = subspace;
        self
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
