use std::fmt::{Debug, Formatter, Result};

use biodivine_lib_bdd::Bdd;
use thiserror::Error;

use crate::internal::algorithms::{
    cancellation::CancellationError, fixed_points::FixedPointsError,
};

/// An error returned by a [TrapSpaces] procedure.
#[derive(Error)]
pub enum TrapSpacesError {
    #[error("config creation failed: {0}")]
    CreationFailed(String),
    #[error("operation cancelled")]
    Cancelled(Bdd),
    #[error("BDD size limit exceeded")]
    BddSizeLimitExceeded(Bdd),
}

/// The default implementation will print the whole BDD, which can be quite large.
impl Debug for TrapSpacesError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            TrapSpacesError::CreationFailed(error) => {
                write!(f, "CreationFailed({})", error)
            }
            TrapSpacesError::Cancelled(bdd) => {
                write!(f, "Cancelled(partial_result={})", bdd.exact_cardinality())
            }
            TrapSpacesError::BddSizeLimitExceeded(bdd) => {
                write!(
                    f,
                    "BddSizeLimitExceeded(partial_result={})",
                    bdd.exact_cardinality()
                )
            }
        }
    }
}

impl From<CancellationError<Bdd>> for TrapSpacesError {
    fn from(error_value: CancellationError<Bdd>) -> Self {
        TrapSpacesError::Cancelled(error_value.into_partial_data())
    }
}

impl From<FixedPointsError> for TrapSpacesError {
    fn from(error_value: FixedPointsError) -> Self {
        match error_value {
            FixedPointsError::CreationFailed(msg) => TrapSpacesError::CreationFailed(msg),
            FixedPointsError::Cancelled(bdd) => TrapSpacesError::Cancelled(bdd),
            FixedPointsError::BddSizeLimitExceeded(bdd) => {
                TrapSpacesError::BddSizeLimitExceeded(bdd)
            }
        }
    }
}
