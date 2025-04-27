use pyo3::{pyclass, FromPyObject, Py, Python};

use crate::{
    bindings::{
        algorithms::{
            cancellation::tokens::CancelTokenPython,
            configurable::Config as _,
            reachability::{ReachabilityConfig, ReachabilityError},
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

// TODO: ohtenkay - implement this for the rest
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
