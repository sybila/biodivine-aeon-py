use std::fmt::{Debug, Formatter, Result};

use biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices;
use thiserror::Error;

use crate::bindings::algorithms::cancellation::CancellationError;

/// An error returned by a [FixedPoints] procedure.
#[derive(Error)]
pub enum FixedPointsError {
    #[error("operation cancelled")]
    CancelledEmpty,
    // #[error("operation cancelled")]
    // Cancelled(GraphColoredVertices),
    // TODO: oktenkay - Errors can only contain GraphColoredVertices, but algoritms can return also
    // only Colors or only Vertices, change this to work with bdds
    #[error("BDD size limit exceeded")]
    BddSizeLimitExceeded(GraphColoredVertices),
}

/// The default implementation will print the whole BDD, which can be quite large.
impl Debug for FixedPointsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            FixedPointsError::CancelledEmpty => {
                write!(f, "Cancelled")
            }
            FixedPointsError::BddSizeLimitExceeded(x) => {
                write!(
                    f,
                    "BddSizeLimitExceeded(partial_result={})",
                    x.exact_cardinality()
                )
            }
        }
    }
}

impl<T> From<CancellationError<T>> for FixedPointsError
where
    T: Sized + Debug + 'static,
{
    fn from(_: CancellationError<T>) -> Self {
        FixedPointsError::CancelledEmpty
    }
}
