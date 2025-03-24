use std::fmt::Debug;

use thiserror::Error;

use crate::bindings::algorithms::cancellation::CancellationError;

// TODO: ohtenkay - impl Debug for FixedPointsError
/// An error returned by a [FixedPoints] procedure.
// TODO: ohtenkay - by default GraphColoredVertices is restriction, in the future, consider making it more precise
#[derive(Error, Debug)]
pub enum FixedPointsError {
    #[error("operation cancelled")]
    CancelledEmpty,
    // #[error("operation cancelled")]
    // Cancelled(GraphColoredVertices),
    // #[error("BDD size limit exceeded")]
    // BddSizeLimitExceeded(GraphColoredVertices),
}

impl<T> From<CancellationError<T>> for FixedPointsError
where
    T: Sized + Debug + 'static,
{
    fn from(_: CancellationError<T>) -> Self {
        FixedPointsError::CancelledEmpty
    }
}
