use std::fmt::{Debug, Formatter, Result};

use biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices;
use thiserror::Error;

use crate::bindings::algorithms::cancellation::CancellationError;

/// An error returned by a [Reachability] procedure.
#[derive(Error)]
pub enum ReachabilityError {
    #[error("config creation failed")]
    CreationFailed(String),
    #[error("operation cancelled")]
    Cancelled(GraphColoredVertices),
    #[error("steps limit exceeded")]
    StepsLimitExceeded(GraphColoredVertices),
    #[error("BDD size limit exceeded")]
    BddSizeLimitExceeded(GraphColoredVertices),
    #[error("subgraph set not compatible with the given graph or initial states")]
    InvalidSubgraph,
}

/// The default implementation will print the whole BDD, which can be quite large.
impl Debug for ReachabilityError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ReachabilityError::CreationFailed(error) => {
                write!(f, "CreationFailed({})", error)
            }
            ReachabilityError::Cancelled(gcv) => {
                write!(f, "Cancelled(partial_result={})", gcv.exact_cardinality())
            }
            ReachabilityError::StepsLimitExceeded(gcv) => {
                write!(
                    f,
                    "StepsLimitExceeded(partial_result={})",
                    gcv.exact_cardinality()
                )
            }
            ReachabilityError::BddSizeLimitExceeded(gcv) => {
                write!(
                    f,
                    "BddSizeLimitExceeded(partial_result={})",
                    gcv.exact_cardinality()
                )
            }
            ReachabilityError::InvalidSubgraph => {
                write!(f, "InvalidSubgraph")
            }
        }
    }
}

impl From<CancellationError<GraphColoredVertices>> for ReachabilityError {
    fn from(error_value: CancellationError<GraphColoredVertices>) -> Self {
        ReachabilityError::Cancelled(error_value.into_partial_data())
    }
}
