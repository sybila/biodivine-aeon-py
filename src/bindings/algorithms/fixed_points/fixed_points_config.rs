use std::time::Duration;

use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use pyo3::{pyclass, pymethods, Py};

use crate::{
    bindings::{
        algorithms::cancellation::{
            tokens::{CancelTokenPython, CancelTokenTimer},
            CancellationHandler,
        },
        lib_param_bn::symbolic::{
            asynchronous_graph::AsynchronousGraph, set_colored_vertex::ColoredVertexSet,
            symbolic_context::SymbolicContext,
        },
    },
    AsNative,
};

#[derive(Clone)]
pub struct FixedPointsConfig {
    pub graph: SymbolicAsyncGraph,
    pub restriction: GraphColoredVertices,
    pub cancellation: Box<dyn CancellationHandler>,
    pub bdd_size_limit: usize,
}

impl FixedPointsConfig {
    /// Create a new "default" [FixedPointsCongfig] for the given [SymbolicAsyncGraph].
    pub fn with_graph(graph: SymbolicAsyncGraph) -> Self {
        FixedPointsConfig {
            restriction: graph.unit_colored_vertices().clone(),
            cancellation: Default::default(),
            bdd_size_limit: usize::MAX,
            graph,
        }
    }

    /// Update the `restriction` property
    pub fn with_restriction(mut self, restriction: GraphColoredVertices) -> Self {
        self.restriction = restriction;
        self
    }

    /// Update the `cancellation` property, automatically wrapping the [CancellationHandler]
    /// in a `Box`.
    pub fn with_cancellation<C: CancellationHandler + 'static>(mut self, cancellation: C) -> Self {
        self.cancellation = Box::new(cancellation);
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
pub struct FixedPointsConfigPython {
    inner: FixedPointsConfig,
    symbolic_context: Py<SymbolicContext>,
}

impl FixedPointsConfigPython {
    pub fn inner(&self) -> FixedPointsConfig {
        self.inner.clone()
    }

    pub fn symbolic_context(&self) -> Py<SymbolicContext> {
        self.symbolic_context.clone()
    }
}

#[pymethods]
impl FixedPointsConfigPython {
    #[new]
    #[pyo3(signature = (graph, restriction = None, time_limit_millis = None, bdd_size_limit = None))]
    pub fn new_py(
        graph: Py<AsynchronousGraph>,
        restriction: Option<Py<ColoredVertexSet>>,
        time_limit_millis: Option<u64>,
        bdd_size_limit: Option<usize>,
    ) -> Self {
        let mut config = FixedPointsConfig::with_graph(graph.get().as_native().clone());

        if let Some(restriction) = restriction {
            config = config.with_restriction(restriction.get().as_native().clone())
        }

        if let Some(millis) = time_limit_millis {
            config = config.with_cancellation(CancelTokenPython::with_inner(CancelTokenTimer::new(
                Duration::from_millis(millis),
            )))
        }

        if let Some(size_limit) = bdd_size_limit {
            config = config.with_bdd_size_limit(size_limit)
        }

        FixedPointsConfigPython {
            inner: config,
            symbolic_context: graph.get().symbolic_context().clone(),
        }
    }

    #[staticmethod]
    pub fn with_graph(graph: Py<AsynchronousGraph>) -> Self {
        let config = FixedPointsConfig::with_graph(graph.get().as_native().clone())
            .with_cancellation(CancelTokenPython::default());

        FixedPointsConfigPython {
            inner: config,
            symbolic_context: graph.get().symbolic_context().clone(),
        }
    }

    pub fn with_restriction(&self, restriction: Py<ColoredVertexSet>) -> Self {
        let config = self
            .inner
            .clone()
            .with_restriction(restriction.get().as_native().clone());

        FixedPointsConfigPython {
            inner: config,
            symbolic_context: self.symbolic_context(),
        }
    }

    // TODO: if we ever move away from abi3-py37, use Duration as an argument
    pub fn with_time_limit(&self, duration_in_millis: u64) -> Self {
        let config = self
            .inner
            .clone()
            .with_cancellation(CancelTokenPython::with_inner(CancelTokenTimer::new(
                Duration::from_millis(duration_in_millis),
            )));

        FixedPointsConfigPython {
            inner: config,
            symbolic_context: self.symbolic_context(),
        }
    }

    pub fn with_bdd_size_limit(&self, bdd_size_limit: usize) -> Self {
        let config = self.inner.clone().with_bdd_size_limit(bdd_size_limit);

        FixedPointsConfigPython {
            inner: config,
            symbolic_context: self.symbolic_context(),
        }
    }
}
