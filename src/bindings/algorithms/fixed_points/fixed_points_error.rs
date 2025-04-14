use std::fmt::{Debug, Formatter, Result};

use biodivine_lib_bdd::Bdd;
use thiserror::Error;

use crate::bindings::algorithms::cancellation::CancellationError;

/// An error returned by a [FixedPoints] procedure.
#[derive(Error)]
pub enum FixedPointsError {
    #[error("operation cancelled")]
    CancelledEmpty,
    #[error("operation cancelled")]
    Cancelled(Bdd),
    #[error("BDD size limit exceeded")]
    BddSizeLimitExceeded(Bdd),
}

/// The default implementation will print the whole BDD, which can be quite large.
impl Debug for FixedPointsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            FixedPointsError::CancelledEmpty => {
                write!(f, "Cancelled")
            }
            FixedPointsError::Cancelled(bdd) => {
                write!(f, "Cancelled(partial_result={})", bdd.exact_cardinality())
            }
            FixedPointsError::BddSizeLimitExceeded(bdd) => {
                write!(
                    f,
                    "BddSizeLimitExceeded(partial_result={})",
                    bdd.exact_cardinality()
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
