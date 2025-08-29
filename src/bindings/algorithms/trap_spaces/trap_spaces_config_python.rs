use std::time::Duration;

use pyo3::{Py, PyResult, pyclass, pymethods};

use crate::{
    AsNative as _,
    bindings::{
        algorithms::{
            graph_representation::PyGraphRepresentation, token_python::CancelTokenPython,
        },
        lib_param_bn::symbolic::{
            asynchronous_graph::AsynchronousGraph, set_colored_space::ColoredSpaceSet,
            symbolic_space_context::SymbolicSpaceContext,
        },
    },
    internal::algorithms::{
        cancellation::CancelTokenTimer,
        configurable::{Config as _, Configurable as _},
        trap_spaces::{TrapSpaces, TrapSpacesConfig},
    },
};

/// A configuration class for the `TrapSpacesComp` class. It allows you to specify various
/// parameters for the trap spaces computation, such as the underlying `AsynchronousGraph`,
/// a restriction set for the spaces, a time limit, and a BDD size limit. The configuration
/// can be created using a Python constructor or the `create_from` method, and you can modify it using the
/// `with_*` methods.
/// Currently, the only supported graph representation is `BooleanNetwork`. For creation from
/// `AsynchronousGraph`, use `create_from_graph_with_context`.
/// The configuration is immutable, meaning that each `with_*` method
/// returns a new instance of `TrapSpacesConfig` with the specified modifications.
/// This API design means the method calls can be chained together.
#[pyclass(name = "TrapSpacesConfig", module = "biodivine_aeon", frozen)]
#[derive(Clone)]
pub struct PyTrapSpacesConfig {
    pub inner: TrapSpaces,
    pub ctx: Py<SymbolicSpaceContext>,
}

impl PyTrapSpacesConfig {
    pub fn extract_inner(self) -> (TrapSpacesConfig, Py<SymbolicSpaceContext>) {
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
    /// `AsynchronousGraph` is currently not supported, use `create_from_graph_with_context` instead.
    #[staticmethod]
    pub fn create_from(graph_representation: PyGraphRepresentation) -> PyResult<Self> {
        PyTrapSpacesConfig::try_from(graph_representation)
    }

    /// Create a new `TrapSpacesConfig` from the given `AsynchronousGraph` and
    /// `SymbolicSpaceContext`, with otherwise default configuration.
    #[staticmethod]
    pub fn create_from_graph_with_context(
        graph: &AsynchronousGraph,
        ctx: Py<SymbolicSpaceContext>,
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
