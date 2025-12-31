use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
use pyo3::{FromPyObject, Py, PyErr, PyResult, Python, create_exception, exceptions::PyException};

use crate::{
    AsNative as _,
    bindings::{
        algorithms::{
            fixed_points::PyFixedPointsConfig, token_python::CancelTokenPython,
            trap_spaces::PyTrapSpacesConfig,
        },
        lib_param_bn::{
            boolean_network::BooleanNetwork,
            symbolic::{
                asynchronous_graph::AsynchronousGraph, symbolic_context::SymbolicContext,
                symbolic_space_context::SymbolicSpaceContext,
            },
        },
    },
    internal::algorithms::{
        configurable::{Config as _, Configurable as _},
        fixed_points::{FixedPoints, FixedPointsConfig},
        percolation::{PercolationConfig, PercolationError},
        trap_spaces::{TrapSpaces, TrapSpacesConfig, TrapSpacesError},
    },
};

#[derive(FromPyObject)]
pub enum PyAsynchronousGraphType {
    Graph(Py<AsynchronousGraph>),
    Network(Py<BooleanNetwork>),
}

impl PyAsynchronousGraphType {
    pub fn clone_native(&self, py: Python) -> SymbolicAsyncGraph {
        match self {
            PyAsynchronousGraphType::Graph(value) => value.get().as_native().clone(),
            PyAsynchronousGraphType::Network(value) => {
                SymbolicAsyncGraph::new(value.borrow(py).as_native()).unwrap()
            }
        }
    }

    pub fn clone_py_context(&self, py: Python) -> PyResult<Py<SymbolicContext>> {
        match &self {
            PyAsynchronousGraphType::Graph(graph) => Ok(graph.borrow(py).symbolic_context()),
            PyAsynchronousGraphType::Network(network) => {
                Py::new(py, SymbolicContext::new(py, network.clone(), None)?)
            }
        }
    }
}

impl TryFrom<PyAsynchronousGraphType> for PercolationConfig {
    type Error = PercolationError;

    /// Create a new "default" [PercolationConfig] from the given [PyAsynchronousGraphType].
    fn try_from(representation: PyAsynchronousGraphType) -> Result<Self, Self::Error> {
        match representation {
            PyAsynchronousGraphType::Graph(graph) => {
                Ok(PercolationConfig::from(graph.get().as_native().clone())
                    .with_cancellation(CancelTokenPython::default()))
            }
            PyAsynchronousGraphType::Network(network) => Python::attach(|py| {
                PercolationConfig::try_from(network.borrow(py).as_native())
                    .map(|config| config.with_cancellation(CancelTokenPython::default()))
            }),
        }
    }
}

impl TryFrom<PyAsynchronousGraphType> for PyFixedPointsConfig {
    type Error = PyErr;

    /// Create a new "default" [PyFixedPointsConfig] from the given [PyAsynchronousGraphType].
    fn try_from(representation: PyAsynchronousGraphType) -> Result<Self, Self::Error> {
        match representation {
            PyAsynchronousGraphType::Graph(graph) => {
                let config = FixedPointsConfig::from(graph.get().as_native().clone())
                    .with_cancellation(CancelTokenPython::default());

                Ok(PyFixedPointsConfig {
                    inner: FixedPoints::with_config(config),
                    ctx: graph.get().symbolic_context().clone(),
                })
            }
            PyAsynchronousGraphType::Network(network) => Python::attach(|py| {
                let stg = AsynchronousGraph::new(py, network, None, None)?;
                let config = FixedPointsConfig::from(stg.as_native().clone())
                    .with_cancellation(CancelTokenPython::default());

                Ok(PyFixedPointsConfig {
                    inner: FixedPoints::with_config(config),
                    ctx: stg.symbolic_context().clone(),
                })
            }),
        }
    }
}

impl TryFrom<PyAsynchronousGraphType> for PyTrapSpacesConfig {
    type Error = PyErr;

    /// Create a new "default" [PyTrapSpacesConfig] from the given [PyAsynchronousGraphType].
    fn try_from(representation: PyAsynchronousGraphType) -> Result<Self, Self::Error> {
        match representation {
            PyAsynchronousGraphType::Graph(_graph) => {
                Err(TrapSpacesError::CreationFailed(
                    "Currently, trap spaces cannot be created from just a graph. Use a boolean network or from_graph_with_context() instead."
                        .to_string()).into())
            },
            PyAsynchronousGraphType::Network(network) => Python::attach(|py| {
                let config = TrapSpacesConfig::try_from(network.borrow(py).as_native())?
                    .with_cancellation(
                        CancelTokenPython::default(),
                    );

                let ctx = Py::new(
                    py,
                    (
                        SymbolicSpaceContext::new(config.ctx.clone()),
                        SymbolicContext::new(py, network, None)?,
                    ),
                )?;

                Ok(PyTrapSpacesConfig {
                    inner: TrapSpaces::with_config(config),
                    ctx,
                })
            }),
        }
    }
}

create_exception!(graph_representation, CreationFailedError, PyException);
