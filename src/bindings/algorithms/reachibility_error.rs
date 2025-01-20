use std::fmt::{Debug, Formatter};

use thiserror::Error;

use crate::bindings::lib_param_bn::symbolic::set_colored_vertex::ColoredVertexSet;

/// An error returned by a [Reachability] procedure.
#[derive(Error)]
pub enum ReachabilityError {
    #[error("operation cancelled")]
    Cancelled(ColoredVertexSet),
    #[error("steps limit exceeded")]
    StepsLimitExceeded(ColoredVertexSet),
    #[error("BDD size limit exceeded")]
    BddSizeLimitExceeded(ColoredVertexSet),
    #[error("subgraph set not compatible with the given graph or initial states")]
    InvalidSubgraph,
}

/// The default implementation will print the whole BDD, which can be quite large.
impl Debug for ReachabilityError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ReachabilityError::Cancelled(x) => {
                write!(f, "Cancelled(partial_result={})", x.cardinality())
            }
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
