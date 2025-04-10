use std::fmt::Debug;

use thiserror::Error;

use crate::bindings::algorithms::cancellation::CancellationError;

/// An error returned by a [FixedPoints] procedure.
#[derive(Error, Debug)]
pub enum PercolationError {
    #[error("operation cancelled")]
    CancelledEmpty,
    // #[error("operation cancelled")]
    // Cancelled(GraphColoredVertices),
}

impl<T> From<CancellationError<T>> for PercolationError
where
    T: Sized + Debug + 'static,
{
    fn from(_: CancellationError<T>) -> Self {
        PercolationError::CancelledEmpty
    }
}
