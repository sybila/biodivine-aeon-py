use std::{collections::HashSet, time::Duration};

use pyo3::{pymethods, PyResult};

use crate::{
    bindings::{
        algorithms::{
            cancellation::tokens::{CancelTokenPython, CancelTokenTimer},
            configurable::Config as _,
            graph_representation::PyGraphRepresentation,
        },
        lib_param_bn::{symbolic::set_colored_vertex::ColoredVertexSet, variable_id::VariableId},
    },
    AsNative as _,
};

use super::ReachabilityConfig;

/// These methods are Python facing wrappers of native methods and thus should not be used from
/// within Rust. When working with [ReachabilityConfig] from Rust, use methods without the python_
/// prefix.
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

    /// Update the `subgraph` property.
    #[pyo3(name = "with_subgraph")]
    pub fn python_with_subgraph(&self, subgraph: &ColoredVertexSet) -> Self {
        self.clone().with_subgraph(subgraph.as_native().clone())
    }

    /// Update the `variables` property.
    #[pyo3(name = "with_variables")]
    pub fn python_with_variables(&self, variables: HashSet<VariableId>) -> Self {
        self.clone()
            .with_variables(variables.iter().map(|var| *var.as_native()).collect())
    }

    /// Update the `cancellation` property, setting a time limit in milliseconds.
    // TODO: if we ever move away from abi3-py37, use Duration as an argument
    #[pyo3(name = "with_time_limit")]
    pub fn python_with_time_limit(&self, duration_in_millis: u64) -> Self {
        self.clone()
            .with_cancellation(CancelTokenPython::with_inner(CancelTokenTimer::new(
                Duration::from_millis(duration_in_millis),
            )))
    }

    /// Update the `bdd_size_limit` property.
    #[pyo3(name = "with_bdd_size_limit")]
    pub fn python_with_bdd_size_limit(&self, bdd_size_limit: usize) -> Self {
        self.clone().with_bdd_size_limit(bdd_size_limit)
    }

    /// Update the `steps_limit` property.
    #[pyo3(name = "with_steps_limit")]
    pub fn python_with_steps_limit(&self, steps_limit: usize) -> Self {
        self.clone().with_steps_limit(steps_limit)
    }
}
