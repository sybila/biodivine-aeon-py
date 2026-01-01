use std::time::Duration;

use pyo3::{Py, PyResult, pyclass, pymethods};

use crate::{
    AsNative as _,
    bindings::{
        algorithms::{
            graph_representation::PyAsynchronousGraphType, token_python::CancelTokenPython,
        },
        lib_param_bn::symbolic::{
            set_colored_vertex::ColoredVertexSet, symbolic_context::SymbolicContext,
        },
    },
    internal::algorithms::{
        cancellation::CancelTokenTimer,
        configurable::{Config as _, Configurable as _},
        fixed_points::{FixedPoints, FixedPointsConfig},
    },
};

/// A configuration class for the `FixedPointsComp` class. It allows you to specify various
/// parameters for the fixed points computation, such as the underlying `AsynchronousGraph`,
/// a restriction set for the vertices, a time limit, and a BDD size limit. The configuration
/// can be created using a Python constructor or the `create_from` method, and you can modify it using the
/// `with_*` methods.
/// The configuration is immutable, meaning that each `with_*` method
/// returns a new instance of `FixedPointsConfig` with the specified modifications.
/// This API design means the method calls can be chained together.
///
/// **This feature should eventually replace `FixedPoints` and is currently in "preview mode",
/// so please expect the API of this object to change. For guaranteed stable API, please rely
/// on `FixedPoints`.**
#[pyclass(name = "FixedPointsConfig", module = "biodivine_aeon", frozen)]
#[derive(Clone)]
pub struct PyFixedPointsConfig {
    pub inner: FixedPoints,
    pub ctx: Py<SymbolicContext>,
}

impl PyFixedPointsConfig {
    fn extract_inner(self) -> (FixedPointsConfig, Py<SymbolicContext>) {
        (self.inner.into_config(), self.ctx)
    }
}

/// These methods are Python facing wrappers of native methods and thus should not be used from
/// within Rust.
#[pymethods]
impl PyFixedPointsConfig {
    #[new]
    #[pyo3(signature = (graph_representation, restriction = None, time_limit_millis = None, bdd_size_limit = None))]
    pub fn new_py(
        graph_representation: PyAsynchronousGraphType,
        restriction: Option<&ColoredVertexSet>,
        time_limit_millis: Option<u64>,
        bdd_size_limit: Option<usize>,
    ) -> PyResult<Self> {
        let (mut config, ctx) =
            PyFixedPointsConfig::try_from(graph_representation)?.extract_inner();

        if let Some(restriction) = restriction {
            config = config.with_restriction(restriction.as_native().clone())
        }

        if let Some(millis) = time_limit_millis {
            config = config.with_cancellation(CancelTokenPython::with_inner(CancelTokenTimer::new(
                Duration::from_millis(millis),
            )))
        }

        if let Some(size_limit) = bdd_size_limit {
            config = config.with_bdd_size_limit(size_limit)
        }

        Ok(PyFixedPointsConfig {
            inner: FixedPoints::with_config(config),
            ctx,
        })
    }

    /// Create a new `FixedPointsConfig` from the given `AsynchronousGraph` or `BooleanNetwork`,
    /// with otherwise default configuration.
    #[staticmethod]
    pub fn create_from(graph_representation: PyAsynchronousGraphType) -> PyResult<Self> {
        PyFixedPointsConfig::try_from(graph_representation)
    }

    /// Restricts result to the given set of vertices.
    ///
    /// Default: `graph.unit_colored_vertices()`.
    pub fn with_restriction(&self, restriction: &ColoredVertexSet) -> Self {
        let config = self
            .inner
            .config()
            .clone()
            .with_restriction(restriction.as_native().clone());

        PyFixedPointsConfig {
            inner: FixedPoints::with_config(config),
            ctx: self.ctx.clone(),
        }
    }

    /// Sets a time limit for the fixed points computation, in milliseconds.
    ///
    /// Default: no time limit.
    // TODO: if we ever move away from abi3-py37, use Duration as an argument
    pub fn with_time_limit(&self, duration_in_millis: u64) -> Self {
        let config = self
            .inner
            .config()
            .clone()
            .with_cancellation(CancelTokenPython::with_inner(CancelTokenTimer::new(
                Duration::from_millis(duration_in_millis),
            )));

        PyFixedPointsConfig {
            inner: FixedPoints::with_config(config),
            ctx: self.ctx.clone(),
        }
    }

    /// The maximum size of the BDD used in the merging process.
    ///
    /// Note that the algorithms can use other auxiliary BDDs that do not
    /// count towards this limit.
    ///
    /// Default: `usize::MAX`.
    pub fn with_bdd_size_limit(&self, bdd_size_limit: usize) -> Self {
        let config = self
            .inner
            .config()
            .clone()
            .with_bdd_size_limit(bdd_size_limit);

        PyFixedPointsConfig {
            inner: FixedPoints::with_config(config),
            ctx: self.ctx.clone(),
        }
    }
}
