use std::fmt::Debug;

use thiserror::Error;

use crate::bindings::algorithms::cancellation::CancellationError;

// TODO: ohtenkay - impl Debug for FixedPointsError
/// An error returned by a [FixedPoints] procedure.
#[derive(Error, Debug)]
pub enum FixedPointsError {
    #[error("no fixed points found")]
    NoFixedPointsFound,
    #[error("operation cancelled")]
    CancelledEmpty,
    // #[error("operation cancelled")]
    // Cancelled(GraphColoredVertices),
    // #[error("steps limit exceeded")]
    // StepsLimitExceeded(GraphColoredVertices),
    // #[error("BDD size limit exceeded")]
    // BddSizeLimitExceeded(GraphColoredVertices),
    // #[error("subgraph set not compatible with the given graph or initial states")]
    // InvalidSubgraph,
}

impl<T> From<CancellationError<T>> for FixedPointsError
where
    T: Sized + Debug + 'static,
{
    fn from(_: CancellationError<T>) -> Self {
        FixedPointsError::CancelledEmpty
    }
}
