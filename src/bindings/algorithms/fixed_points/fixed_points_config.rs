use std::time::Duration;

use biodivine_lib_param_bn::{
    symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph},
    BooleanNetwork,
};
use macros::Config;
use pyo3::{pyclass, pymethods, Py, PyResult, Python};

use crate::{
    bindings::{
        algorithms::{
            cancellation::{
                tokens::{CancelTokenPython, CancelTokenTimer},
                CancellationHandler,
            },
            configurable::{Config, Configurable as _},
            fixed_points::{fixed_points_error::FixedPointsError, fixed_points_impl::FixedPoints},
        },
        lib_param_bn::{
            boolean_network::BooleanNetwork as BooleanNetworkBinding,
            symbolic::{
                asynchronous_graph::AsynchronousGraph, set_colored_vertex::ColoredVertexSet,
                symbolic_context::SymbolicContext as SymbolicContextBinding,
            },
        },
    },
    AsNative as _,
};

#[derive(Clone, Config)]
pub struct FixedPointsConfig {
    pub graph: SymbolicAsyncGraph,
    pub restriction: GraphColoredVertices,
    pub cancellation: Box<dyn CancellationHandler>,
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

#[pyclass(module = "biodivine_aeon", frozen)]
#[pyo3(name = "FixedPointsConfig")]
#[derive(Clone)]
pub struct PyFixedPointsConfig {
    inner: FixedPoints,
    ctx: Py<SymbolicContextBinding>,
}

impl PyFixedPointsConfig {
    pub fn inner(&self) -> &FixedPoints {
        &self.inner
    }

    pub fn symbolic_context(&self) -> Py<SymbolicContextBinding> {
        self.ctx.clone()
    }
}

#[pymethods]
impl PyFixedPointsConfig {
    #[new]
    #[pyo3(signature = (graph, restriction = None, time_limit_millis = None, bdd_size_limit = None))]
    pub fn new_py(
        graph: &AsynchronousGraph,
        restriction: Option<&ColoredVertexSet>,
        time_limit_millis: Option<u64>,
        bdd_size_limit: Option<usize>,
    ) -> Self {
        let mut config = FixedPointsConfig::from(graph.as_native().clone());

        if let Some(restriction) = restriction {
            config = config.with_restriction(restriction.as_native().clone())
        }

        if let Some(millis) = time_limit_millis {
            config = config.with_cancellation(CancelTokenPython::with_inner(CancelTokenTimer::new(
                Duration::from_millis(millis),
            )))
        } else {
            config = config.with_cancellation(CancelTokenPython::default());
        }

        if let Some(size_limit) = bdd_size_limit {
            config = config.with_bdd_size_limit(size_limit)
        }

        PyFixedPointsConfig {
            inner: FixedPoints::with_config(config),
            ctx: graph.symbolic_context().clone(),
        }
    }

    #[staticmethod]
    pub fn from_boolean_network(
        py: Python,
        boolean_network: Py<BooleanNetworkBinding>,
    ) -> PyResult<Self> {
        let stg = AsynchronousGraph::new(py, boolean_network, None, None)?;
        let config = FixedPointsConfig::from(stg.as_native().clone())
            .with_cancellation(CancelTokenPython::default());

        Ok(PyFixedPointsConfig {
            inner: FixedPoints::with_config(config),
            ctx: stg.symbolic_context().clone(),
        })
    }

    #[staticmethod]
    pub fn from_graph(graph: &AsynchronousGraph) -> Self {
        let config = FixedPointsConfig::from(graph.as_native().clone())
            .with_cancellation(CancelTokenPython::default());

        PyFixedPointsConfig {
            inner: FixedPoints::with_config(config),
            ctx: graph.symbolic_context().clone(),
        }
    }

    pub fn with_restriction(&self, restriction: &ColoredVertexSet) -> Self {
        let config = self
            .inner
            .config()
            .clone()
            .with_restriction(restriction.as_native().clone());

        PyFixedPointsConfig {
            inner: FixedPoints::with_config(config),
            ctx: self.symbolic_context(),
        }
    }

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
            ctx: self.symbolic_context(),
        }
    }

    pub fn with_bdd_size_limit(&self, bdd_size_limit: usize) -> Self {
        let config = self
            .inner
            .config()
            .clone()
            .with_bdd_size_limit(bdd_size_limit);

        PyFixedPointsConfig {
            inner: FixedPoints::with_config(config),
            ctx: self.symbolic_context(),
        }
    }
}
