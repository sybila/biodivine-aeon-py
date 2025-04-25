use std::time::Duration;

use biodivine_lib_param_bn::{
    symbolic_async_graph::SymbolicAsyncGraph,
    trap_spaces::{NetworkColoredSpaces, SymbolicSpaceContext},
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
            configurable::{Config, Configurable},
            trap_spaces::{trap_spaces_error::TrapSpacesError, trap_spaces_impl::TrapSpaces},
        },
        lib_param_bn::{
            boolean_network::BooleanNetwork as BooleanNetworkBinding,
            symbolic::{
                asynchronous_graph::AsynchronousGraph, set_colored_space::ColoredSpaceSet,
                symbolic_context::SymbolicContext,
                symbolic_space_context::SymbolicSpaceContext as SymbolicSpaceContextBinding,
            },
        },
    },
    AsNative as _,
};

/// A configuration struct for the [TrapSpaces] algorithms.
#[derive(Clone, Config)]
pub struct TrapSpacesConfig {
    pub graph: SymbolicAsyncGraph,
    pub ctx: SymbolicSpaceContext,
    pub restriction: NetworkColoredSpaces,

    /// A `CancellationHandler` that can be used to stop the algorithm externally.
    ///
    /// Default: [CancelTokenNever].
    pub cancellation: Box<dyn CancellationHandler>,
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

        Ok(TrapSpacesConfig {
            restriction: ctx.mk_unit_colored_spaces(&graph),
            cancellation: Default::default(),
            bdd_size_limit: usize::MAX,
            graph,
            ctx,
        })
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

#[pyclass(module = "biodivine_aeon", frozen)]
#[pyo3(name = "TrapSpacesConfig")]
#[derive(Clone)]
pub struct PyTrapSpacesConfig {
    inner: TrapSpaces,
    ctx: Py<SymbolicSpaceContextBinding>,
}

impl PyTrapSpacesConfig {
    pub fn inner(&self) -> &TrapSpaces {
        &self.inner
    }

    pub fn symbolic_space_context(&self) -> Py<SymbolicSpaceContextBinding> {
        self.ctx.clone()
    }
}

#[pymethods]
impl PyTrapSpacesConfig {
    #[new]
    #[pyo3(signature = (graph, ctx, restriction = None, time_limit_millis = None))]
    pub fn python_new(
        graph: &AsynchronousGraph,
        ctx: Py<SymbolicSpaceContextBinding>,
        restriction: Option<&ColoredSpaceSet>,
        time_limit_millis: Option<u64>,
    ) -> PyResult<Self> {
        let ctx_native = ctx.get().as_native();
        let mut config = TrapSpacesConfig::from((graph.as_native().clone(), ctx_native.clone()));

        if let Some(restriction) = restriction {
            config = config.with_restriction(restriction.as_native().clone())
        } else {
            config = config.with_restriction(ctx_native.mk_unit_colored_spaces(graph.as_native()));
        };

        if let Some(millis) = time_limit_millis {
            config = config.with_cancellation(CancelTokenPython::with_inner(CancelTokenTimer::new(
                Duration::from_millis(millis),
            )))
        } else {
            config = config.with_cancellation(CancelTokenPython::default());
        }

        Ok(PyTrapSpacesConfig {
            inner: TrapSpaces::with_config(config),
            ctx,
        })
    }

    #[staticmethod]
    pub fn from_boolean_network(py: Python, bn: Py<BooleanNetworkBinding>) -> PyResult<Self> {
        let config = TrapSpacesConfig::try_from(bn.borrow(py).as_native())?;

        let ctx = Py::new(
            py,
            (
                SymbolicSpaceContextBinding::new(config.ctx.clone()),
                SymbolicContext::new(py, bn, None)?,
            ),
        )?;

        Ok(PyTrapSpacesConfig {
            inner: TrapSpaces::with_config(config),
            ctx,
        })
    }

    // TODO: discuss - is this OK?
    #[staticmethod]
    pub fn from_graph_with_context(
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
