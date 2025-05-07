use std::time::Duration;

use biodivine_lib_param_bn::{
    symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph},
    BooleanNetwork,
};
use macros::Config;
use pyo3::{pyclass, pymethods, Py, PyResult};

use crate::{
    bindings::{
        algorithms::{
            cancellation::{
                tokens::{CancelTokenPython, CancelTokenTimer},
                CancellationHandler,
            },
            configurable::{Config, Configurable as _},
            fixed_points::{fixed_points_error::FixedPointsError, fixed_points_impl::FixedPoints},
            graph_representation::PyGraphRepresentation,
        },
        lib_param_bn::symbolic::{
            set_colored_vertex::ColoredVertexSet, symbolic_context::SymbolicContext,
        },
    },
    AsNative as _,
};

/// A configuration struct for the [FixedPoints] algorithms.
#[derive(Clone, Config)]
pub struct FixedPointsConfig {
    /// The symbolic graph that will be used to compute the fixed points.
    pub graph: SymbolicAsyncGraph,

    /// Restricts result to the given set of vertices.
    ///
    /// Default: `graph.unit_colored_vertices()`.
    pub restriction: GraphColoredVertices,

    /// A `CancellationHandler` that can be used to stop the algorithm externally.
    ///
    /// Default: [CancelTokenNever].
    pub cancellation: Box<dyn CancellationHandler>,

    /// The maximum size of the BDD used in the merging process.
    ///
    /// Note that the algorithm can use other auxiliary BDDs that do not
    /// count towards this limit.
    ///
    /// Default: `usize::MAX`.
    pub bdd_size_limit: usize,
}

impl From<SymbolicAsyncGraph> for FixedPointsConfig {
    /// Create a new "default" [FixedPointsConfig] from the given [SymbolicAsyncGraph].
    fn from(graph: SymbolicAsyncGraph) -> Self {
        FixedPointsConfig {
            restriction: graph.mk_unit_colored_vertices(),
            cancellation: Default::default(),
            bdd_size_limit: usize::MAX,
            graph,
        }
    }
}

impl TryFrom<&BooleanNetwork> for FixedPointsConfig {
    type Error = FixedPointsError;

    /// Create a new "default" [FixedPointsConfig] from the given [BooleanNetwork].
    fn try_from(boolean_network: &BooleanNetwork) -> Result<Self, Self::Error> {
        let graph =
            SymbolicAsyncGraph::new(boolean_network).map_err(FixedPointsError::CreationFailed)?;

        Ok(Self::from(graph))
    }
}

impl FixedPointsConfig {
    /// Update the `restriction` property
    pub fn with_restriction(mut self, restriction: GraphColoredVertices) -> Self {
        self.restriction = restriction;
        self
    }

    /// Update the `bdd_size_limit` property.
    pub fn with_bdd_size_limit(mut self, bdd_size_limit: usize) -> Self {
        self.bdd_size_limit = bdd_size_limit;
        self
    }
}

/// A configuration class for the `FixedPoints` class. It allows you to specify various
/// parameters for the fixed points computation, such as the underlying `AsynchronousGraph`,
/// a restriction set for the vertices, a time limit, and a BDD size limit. The configuration
/// can be created using a Python constructor or the `create_from` method, and you can modify it using the
/// `with_*` methods.
/// The configuration is immutable, meaning that each `with_*` method
/// returns a new instance of `FixedPointsConfig` with the specified modifications.
/// This API design means the method calls can be chained together.
#[pyclass(module = "biodivine_aeon", frozen)]
#[pyo3(name = "FixedPointsConfig")]
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
        graph_representation: PyGraphRepresentation,
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
    pub fn create_from(graph_representation: PyGraphRepresentation) -> PyResult<Self> {
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
