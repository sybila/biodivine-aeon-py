use std::fmt::Debug;

use thiserror::Error;

use crate::bindings::algorithms::cancellation::CancellationError;

/// An error returned by a [Percolation] procedure.
#[derive(Error, Debug)]
pub enum PercolationError {
    #[error("config creation failed")]
    CreationFailed(String),
    #[error("operation cancelled")]
    Cancelled(Vec<Option<bool>>),
}

impl From<CancellationError<Vec<Option<bool>>>> for PercolationError {
    fn from(error_value: CancellationError<Vec<Option<bool>>>) -> Self {
        PercolationError::Cancelled(error_value.into_partial_data())
    }
}
