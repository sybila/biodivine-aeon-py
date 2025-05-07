use std::time::Duration;

use biodivine_lib_param_bn::{
    symbolic_async_graph::SymbolicAsyncGraph,
    trap_spaces::{NetworkColoredSpaces, SymbolicSpaceContext},
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
            configurable::{Config, Configurable},
            graph_representation::PyGraphRepresentation,
            trap_spaces::{trap_spaces_error::TrapSpacesError, trap_spaces_impl::TrapSpaces},
        },
        lib_param_bn::symbolic::{
            asynchronous_graph::AsynchronousGraph, set_colored_space::ColoredSpaceSet,
            symbolic_space_context::SymbolicSpaceContext as SymbolicSpaceContextBinding,
        },
    },
    AsNative as _,
};

/// A configuration struct for the [TrapSpaces] algorithms.
#[derive(Clone, Config)]
pub struct TrapSpacesConfig {
    /// The symbolic graph that will be used to compute the trap spaces.
    pub graph: SymbolicAsyncGraph,

    /// The symbolic space context that will be used to compute the trap spaces.
    pub ctx: SymbolicSpaceContext,

    /// Restricts result to the given set of spaces.
    ///
    /// Default: `ctx.mk_unit_colored_spaces(&graph)`.
    pub restriction: NetworkColoredSpaces,

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

// TODO: the current API does not allow creation straight from SymbolicAsyncGraph, this is a
// temporary workaround
impl From<(SymbolicAsyncGraph, SymbolicSpaceContext)> for TrapSpacesConfig {
    /// Create a new "default" [TrapSpacesConfig] from the given [SymbolicAsyncGraph] and
    /// [SymbolicSpaceContext].
    fn from((graph, ctx): (SymbolicAsyncGraph, SymbolicSpaceContext)) -> Self {
        assert_eq!(
            graph.symbolic_context().bdd_variable_set().variable_names(),
            ctx.bdd_variable_set().variable_names()
        );
        TrapSpacesConfig {
            restriction: ctx.mk_unit_colored_spaces(&graph),
            cancellation: Default::default(),
            bdd_size_limit: usize::MAX,
            graph,
            ctx,
        }
    }
}

impl TryFrom<&BooleanNetwork> for TrapSpacesConfig {
    type Error = TrapSpacesError;

    fn try_from(bn: &BooleanNetwork) -> Result<Self, Self::Error> {
        let graph = SymbolicAsyncGraph::new(bn).map_err(TrapSpacesError::CreationFailed)?;
        let ctx = SymbolicSpaceContext::new(bn);

        Ok(Self::from((graph, ctx)))
    }
}

impl TrapSpacesConfig {
    /// Update the `restriction` property
    pub fn with_restriction(mut self, restriction: NetworkColoredSpaces) -> Self {
        self.restriction = restriction;
        self
    }

    /// Update the `bdd_size_limit` property.
    pub fn with_bdd_size_limit(mut self, bdd_size_limit: usize) -> Self {
        self.bdd_size_limit = bdd_size_limit;
        self
    }
}

/// A configuration class for the `TrapSpaces` class. It allows you to specify various
/// parameters for the trap spaces computation, such as the underlying `AsynchronousGraph`,
/// a restriction set for the spaces, a time limit, and a BDD size limit. The configuration
/// can be created using a Python constructor or the `create_from` method, and you can modify it using the
/// `with_*` methods.
/// Currently, the only supported graph representation is `BooleanNetwork`. For creation from
/// `AsynchronousGraph`, use `create_from_graph_with_context`.
/// The configuration is immutable, meaning that each `with_*` method
/// returns a new instance of `TrapSpacesConfig` with the specified modifications.
/// This API design means the method calls can be chained together.
#[pyclass(module = "biodivine_aeon", frozen)]
#[pyo3(name = "TrapSpacesConfig")]
#[derive(Clone)]
pub struct PyTrapSpacesConfig {
    pub inner: TrapSpaces,
    pub ctx: Py<SymbolicSpaceContextBinding>,
}

impl PyTrapSpacesConfig {
    pub fn extract_inner(self) -> (TrapSpacesConfig, Py<SymbolicSpaceContextBinding>) {
        (self.inner.into_config(), self.ctx)
    }
}

/// These methods are Python facing wrappers of native methods and thus should not be used from
/// within Rust.
#[pymethods]
impl PyTrapSpacesConfig {
    #[new]
    #[pyo3(signature = (graph_representation, restriction = None, time_limit_millis = None, bdd_size_limit = None))]
    pub fn python_new(
        graph_representation: PyGraphRepresentation,
        restriction: Option<&ColoredSpaceSet>,
        time_limit_millis: Option<u64>,
        bdd_size_limit: Option<usize>,
    ) -> PyResult<Self> {
        let (mut config, ctx) = PyTrapSpacesConfig::try_from(graph_representation)?.extract_inner();

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

        Ok(PyTrapSpacesConfig {
            inner: TrapSpaces::with_config(config),
            ctx,
        })
    }

    /// Create a new `TrapSpacesConfig` from the given `BooleanNetwork`,
    /// with otherwise default configuration.
    /// `AsynchronousGraph` is currently not supported, use `from_graph_with_context` instead.
    #[staticmethod]
    pub fn create_from(graph_representation: PyGraphRepresentation) -> PyResult<Self> {
        PyTrapSpacesConfig::try_from(graph_representation)
    }

    /// Create a new `TrapSpacesConfig` from the given `AsynchronousGraph` and
    /// `SymbolicSpaceContext`, with otherwise default configuration.
    #[staticmethod]
    pub fn create_from_graph_with_context(
        graph: &AsynchronousGraph,
        ctx: Py<SymbolicSpaceContextBinding>,
    ) -> Self {
        let config =
            TrapSpacesConfig::from((graph.as_native().clone(), ctx.get().as_native().clone()));

        PyTrapSpacesConfig {
            inner: TrapSpaces::with_config(config),
            ctx,
        }
    }

    /// Restricts result to the given set of spaces.
    ///
    /// Default: `ctx.mk_unit_colored_spaces(&graph)`.
    pub fn with_restriction(&self, restriction: &ColoredSpaceSet) -> Self {
        let config = self
            .inner
            .config()
            .clone()
            .with_restriction(restriction.as_native().clone());

        PyTrapSpacesConfig {
            inner: TrapSpaces::with_config(config),
            ctx: self.ctx.clone(),
        }
    }

    /// Sets a time limit for the trap spaces computation, in milliseconds.
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

        PyTrapSpacesConfig {
            inner: TrapSpaces::with_config(config),
            ctx: self.ctx.clone(),
        }
    }

    /// Sets a limit on the size of the BDD used in the merging process.
    ///
    /// Note that the algorithm can use other auxiliary BDDs that do not
    /// count towards this limit.
    ///
    /// Default: `usize::MAX`.
    pub fn with_bdd_size_limit(&self, bdd_size_limit: usize) -> Self {
        let config = self
            .inner
            .config()
            .clone()
            .with_bdd_size_limit(bdd_size_limit);

        PyTrapSpacesConfig {
            inner: TrapSpaces::with_config(config),
            ctx: self.ctx.clone(),
        }
    }
}
