use pyo3::{FromPyObject, Py, PyErr, Python};

use crate::{
    bindings::{
        algorithms::{
            cancellation::tokens::CancelTokenPython,
            configurable::{Config as _, Configurable as _},
            fixed_points::{
                fixed_points_config::{FixedPointsConfig, PyFixedPointsConfig},
                fixed_points_impl::FixedPoints,
            },
            percolation::{
                percolation_config::PercolationConfig, percolation_error::PercolationError,
            },
            reachability::{ReachabilityConfig, ReachabilityError},
            trap_spaces::{
                trap_spaces_config::{PyTrapSpacesConfig, TrapSpacesConfig},
                trap_spaces_error::TrapSpacesError,
                trap_spaces_impl::TrapSpaces,
            },
        },
        lib_param_bn::{
            boolean_network::BooleanNetwork,
            symbolic::{
                asynchronous_graph::AsynchronousGraph, symbolic_context::SymbolicContext,
                symbolic_space_context::SymbolicSpaceContext,
            },
        },
    },
    AsNative as _,
};

#[derive(FromPyObject)]
pub enum PyGraphRepresentation {
    Graph(Py<AsynchronousGraph>),
    Network(Py<BooleanNetwork>),
}

impl TryFrom<PyGraphRepresentation> for ReachabilityConfig {
    type Error = ReachabilityError;

    /// Create a new "default" [ReachabilityConfig] from the given [PyGraphRepresentation].
    fn try_from(representation: PyGraphRepresentation) -> Result<Self, Self::Error> {
        match representation {
            PyGraphRepresentation::Graph(graph) => {
                Ok(ReachabilityConfig::from(graph.get().as_native().clone())
                    .with_cancellation(CancelTokenPython::default()))
            }
            PyGraphRepresentation::Network(network) => Python::with_gil(|py| {
                ReachabilityConfig::try_from(network.borrow(py).as_native())
                    .map(|config| config.with_cancellation(CancelTokenPython::default()))
            }),
        }
    }
}

impl TryFrom<PyGraphRepresentation> for PercolationConfig {
    type Error = PercolationError;

    /// Create a new "default" [PercolationConfig] from the given [PyGraphRepresentation].
    fn try_from(representation: PyGraphRepresentation) -> Result<Self, Self::Error> {
        match representation {
            PyGraphRepresentation::Graph(graph) => {
                Ok(PercolationConfig::from(graph.get().as_native().clone())
                    .with_cancellation(CancelTokenPython::default()))
            }
            PyGraphRepresentation::Network(network) => Python::with_gil(|py| {
                PercolationConfig::try_from(network.borrow(py).as_native())
                    .map(|config| config.with_cancellation(CancelTokenPython::default()))
            }),
        }
    }
}

impl TryFrom<PyGraphRepresentation> for PyFixedPointsConfig {
    type Error = PyErr;

    /// Create a new "default" [PyFixedPointsConfig] from the given [PyGraphRepresentation].
    fn try_from(representation: PyGraphRepresentation) -> Result<Self, Self::Error> {
        match representation {
            PyGraphRepresentation::Graph(graph) => {
                let config = FixedPointsConfig::from(graph.get().as_native().clone())
                    .with_cancellation(CancelTokenPython::default());
                let ctx = graph.get().symbolic_context().clone();

                Ok(PyFixedPointsConfig::new(
                    FixedPoints::with_config(config),
                    ctx,
                ))
            }
            PyGraphRepresentation::Network(network) => Python::with_gil(|py| {
                let stg = AsynchronousGraph::new(py, network, None, None)?;
                let config = FixedPointsConfig::from(stg.as_native().clone())
                    .with_cancellation(CancelTokenPython::default());

                Ok(PyFixedPointsConfig::new(
                    FixedPoints::with_config(config),
                    stg.symbolic_context().clone(),
                ))
            }),
        }
    }
}

impl TryFrom<PyGraphRepresentation> for PyTrapSpacesConfig {
    type Error = PyErr;

    /// Create a new "default" [PyTrapSpacesConfig] from the given [PyGraphRepresentation].
    fn try_from(representation: PyGraphRepresentation) -> Result<Self, Self::Error> {
        match representation {
            PyGraphRepresentation::Graph(_graph) => {
                Err(TrapSpacesError::CreationFailed(
                    "Currently, trap spaces cannot be created from just a graph. Use a boolean network or from_graph_with_context() instead."
                        .to_string()).into())
            },
            PyGraphRepresentation::Network(network) => Python::with_gil(|py| {
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

                Ok(PyTrapSpacesConfig::new(
                    TrapSpaces::with_config(config),
                    ctx,
                ))
            }),
        }
    }
}
