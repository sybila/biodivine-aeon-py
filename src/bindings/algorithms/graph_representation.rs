use pyo3::{pyclass, FromPyObject, Py, Python};

use crate::{
    bindings::{
        algorithms::{
            cancellation::tokens::CancelTokenPython,
            configurable::Config as _,
            fixed_points::{
                fixed_points_config::FixedPointsConfig, fixed_points_error::FixedPointsError,
            },
            percolation::{
                percolation_config::PercolationConfig, percolation_error::PercolationError,
            },
            reachability::{ReachabilityConfig, ReachabilityError},
            trap_spaces::{
                trap_spaces_config::TrapSpacesConfig, trap_spaces_error::TrapSpacesError,
            },
        },
        lib_param_bn::{
            boolean_network::BooleanNetwork, symbolic::asynchronous_graph::AsynchronousGraph,
        },
    },
    AsNative as _,
};

#[pyclass(module = "biodivine_aeon", frozen)]
#[derive(FromPyObject)]
pub enum GraphRepresentation {
    Graph(Py<AsynchronousGraph>),
    Network(Py<BooleanNetwork>),
}

impl TryFrom<GraphRepresentation> for ReachabilityConfig {
    type Error = ReachabilityError;

    /// Create a new "default" [ReachabilityConfig] from the given [GraphRepresentation].
    fn try_from(representation: GraphRepresentation) -> Result<Self, Self::Error> {
        match representation {
            GraphRepresentation::Graph(graph) => {
                Ok(ReachabilityConfig::from(graph.get().as_native().clone())
                    .with_cancellation(CancelTokenPython::default()))
            }
            GraphRepresentation::Network(network) => Python::with_gil(|py| {
                ReachabilityConfig::try_from(network.borrow(py).as_native())
                    .map(|config| config.with_cancellation(CancelTokenPython::default()))
            }),
        }
    }
}

impl TryFrom<GraphRepresentation> for PercolationConfig {
    type Error = PercolationError;

    /// Create a new "default" [PercolationConfig] from the given [GraphRepresentation].
    fn try_from(representation: GraphRepresentation) -> Result<Self, Self::Error> {
        match representation {
            GraphRepresentation::Graph(graph) => {
                Ok(PercolationConfig::from(graph.get().as_native().clone())
                    .with_cancellation(CancelTokenPython::default()))
            }
            GraphRepresentation::Network(network) => Python::with_gil(|py| {
                PercolationConfig::try_from(network.borrow(py).as_native())
                    .map(|config| config.with_cancellation(CancelTokenPython::default()))
            }),
        }
    }
}

impl TryFrom<GraphRepresentation> for FixedPointsConfig {
    type Error = FixedPointsError;

    /// Create a new "default" [FixedPointsConfig] from the given [GraphRepresentation].
    fn try_from(representation: GraphRepresentation) -> Result<Self, Self::Error> {
        match representation {
            GraphRepresentation::Graph(graph) => {
                Ok(FixedPointsConfig::from(graph.get().as_native().clone())
                    .with_cancellation(CancelTokenPython::default()))
            }
            GraphRepresentation::Network(network) => Python::with_gil(|py| {
                FixedPointsConfig::try_from(network.borrow(py).as_native())
                    .map(|config| config.with_cancellation(CancelTokenPython::default()))
            }),
        }
    }
}

impl TryFrom<GraphRepresentation> for TrapSpacesConfig {
    type Error = TrapSpacesError;

    /// Create a new "default" [TrapSpacesConfig] from the given [GraphRepresentation].
    fn try_from(representation: GraphRepresentation) -> Result<Self, Self::Error> {
        match representation {
            GraphRepresentation::Graph(_graph) => {
                Err(TrapSpacesError::CreationFailed(
                    "Currently, trap spaces cannot be created from just a graph. Use a boolean network or from_graph_with_context() instead."
                        .to_string()))
            },
            GraphRepresentation::Network(network) => Python::with_gil(|py| {
                TrapSpacesConfig::try_from(network.borrow(py).as_native())
                    .map(|config| config.with_cancellation(CancelTokenPython::default()))
            }),
        }
    }
}
