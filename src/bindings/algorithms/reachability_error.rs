use std::fmt::{Debug, Formatter, Result};

use pyo3::{
    create_exception,
    exceptions::{
        asyncio::{CancelledError, InvalidStateError},
        PyException,
    },
    PyErr,
};
use thiserror::Error;

use crate::bindings::{
    algorithms::cancellation_error::CancellationError,
    lib_param_bn::symbolic::set_colored_vertex::ColoredVertexSet,
};

/// An error returned by a [Reachability] procedure.
#[derive(Error)]
pub enum ReachabilityError {
    // TODO: ohtenkay - chech if this is needed
    // #[error("operation cancelled")]
    // Cancelled(ColoredVertexSet),
    #[error("steps limit exceeded")]
    StepsLimitExceeded(ColoredVertexSet),
    #[error("BDD size limit exceeded")]
    BddSizeLimitExceeded(ColoredVertexSet),
    #[error("subgraph set not compatible with the given graph or initial states")]
    InvalidSubgraph,
}

/// The default implementation will print the whole BDD, which can be quite large.
impl Debug for ReachabilityError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            // ReachabilityError::Cancelled(x) => {
            //     write!(f, "Cancelled(partial_result={})", x.cardinality())
            // }
            ReachabilityError::StepsLimitExceeded(x) => {
                write!(f, "StepsLimitExceeded(partial_result={})", x.cardinality())
            }
            ReachabilityError::BddSizeLimitExceeded(x) => {
                write!(
                    f,
                    "BddSizeLimitExceeded(partial_result={})",
                    x.cardinality()
                )
            }
            ReachabilityError::InvalidSubgraph => {
                write!(f, "InvalidSubgraph")
            }
        }
    }
}

impl From<ReachabilityError> for PyErr {
    fn from(err: ReachabilityError) -> Self {
        match err {
            // ReachabilityError::Cancelled(x) => PyErr::new::<CancelledError, _>(format!(
            //     "Cancelled(partial_result={})",
            //     x.cardinality()
            // )),
            ReachabilityError::StepsLimitExceeded(x) => PyErr::new::<StepsLimitExceededError, _>(
                format!("StepsLimitExceeded(partial_result={})", x.cardinality()),
            ),
            ReachabilityError::BddSizeLimitExceeded(x) => {
                PyErr::new::<BddSizeLimitExceededError, _>(format!(
                    "BddSizeLimitExceeded(partial_result={})",
                    x.cardinality()
                ))
            }
            ReachabilityError::InvalidSubgraph => {
                PyErr::new::<InvalidStateError, _>("InvalidSubgraph")
            }
        }
    }
}

// TODO: ohtenkay - think about the module name, maybe make it more specific,
// TODO: ohtenkay - add fourth argument, documentation
create_exception!(bindings, BddSizeLimitExceededError, PyException);
create_exception!(bindings, StepsLimitExceededError, PyException);

impl From<CancellationError<ColoredVertexSet>> for PyErr {
    fn from(value: CancellationError<ColoredVertexSet>) -> Self {
        PyErr::new::<CancelledError, _>(format!(
            "Cancelled(partial_result={})",
            value.partial_data().cardinality()
        ))
    }
}
