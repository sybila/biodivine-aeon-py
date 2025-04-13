use std::fmt::{Debug, Formatter, Result};

use thiserror::Error;

use crate::bindings::algorithms::cancellation::CancellationError;

/// An error returned by a [FixedPoints] procedure.
#[derive(Error)]
pub enum TrapSpacesError {
    #[error("operation cancelled")]
    CancelledEmpty,
    #[error("config creation failed")]
    CreationFailed(String),
    // #[error("operation cancelled")]
    // Cancelled(GraphColoredVertices),
    // TODO: ohtenkay - is there a partial result? yes, also bdd as in fixed points. similar for the
    // limit
    // #[error("BDD size limit exceeded")]
    // BddSizeLimitExceeded(GraphColoredVertices),
}

/// The default implementation will print the whole BDD, which can be quite large.
impl Debug for TrapSpacesError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            TrapSpacesError::CancelledEmpty => {
                write!(f, "Cancelled")
            }
            // FixedPointsError::BddSizeLimitExceeded(x) => {
            //     write!(
            //         f,
            //         "BddSizeLimitExceeded(partial_result={})",
            //         x.exact_cardinality()
            //     )
            // }
            TrapSpacesError::CreationFailed(x) => {
                write!(f, "CreationFailed({})", x)
            }
        }
    }
}

impl<T> From<CancellationError<T>> for TrapSpacesError
where
    T: Sized + Debug + 'static,
{
    fn from(_: CancellationError<T>) -> Self {
        TrapSpacesError::CancelledEmpty
    }
}
