use std::fmt::{Debug, Formatter, Result};

use biodivine_lib_bdd::Bdd;
use thiserror::Error;

use crate::bindings::algorithms::cancellation::CancellationError;

/// An error returned by a [FixedPoints] procedure.
#[derive(Error)]
pub enum FixedPointsError {
    #[error("config creation failed")]
    CreationFailed(String),
    #[error("operation cancelled")]
    Cancelled(Bdd),
    #[error("BDD size limit exceeded")]
    BddSizeLimitExceeded(Bdd),
}

/// The default implementation will print the whole BDD, which can be quite large.
impl Debug for FixedPointsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            FixedPointsError::CreationFailed(error) => {
                write!(f, "CreationFailed({})", error)
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

impl From<CancellationError<Bdd>> for FixedPointsError {
    fn from(error_value: CancellationError<Bdd>) -> Self {
        FixedPointsError::Cancelled(error_value.into_partial_data())
    }
}
