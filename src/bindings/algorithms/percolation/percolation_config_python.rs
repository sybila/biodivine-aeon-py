use std::time::Duration;

use pyo3::{PyResult, pymethods};

use crate::{
    bindings::algorithms::{
        graph_representation::PyGraphRepresentation, token_python::CancelTokenPython,
    },
    internal::algorithms::{
        cancellation::CancelTokenTimer, configurable::Config as _, percolation::PercolationConfig,
    },
};

/// These methods are Python facing wrappers of native methods and thus should not be used from
/// within Rust.
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
        }

        Ok(config)
    }

    /// Create a new `PercolationConfig` from the given `AsynchronousGraph` or `BooleanNetwork`,
    /// with otherwise default configuration.
    #[staticmethod]
    #[pyo3(name = "create_from")]
    pub fn python_create_from(graph_representation: PyGraphRepresentation) -> PyResult<Self> {
        Ok(PercolationConfig::try_from(graph_representation)?)
    }

    /// Sets a time limit for the subspace percolation algorithm, in milliseconds.
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
}
