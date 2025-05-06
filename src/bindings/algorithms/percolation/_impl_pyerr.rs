use pyo3::PyErr;

use crate::bindings::algorithms::{
    cancellation::CancelledError, configurable::CreationFailedError,
};

use super::PercolationError;

impl From<PercolationError> for PyErr {
    fn from(err: PercolationError) -> Self {
        match err {
            PercolationError::CreationFailed(x) => {
                PyErr::new::<CreationFailedError, _>(format!("Config creation failed: {}", x))
            }
            PercolationError::Cancelled(x) => {
                PyErr::new::<CancelledError, _>(format!("Cancelled: partial_result={:#?}", x))
            }
        }
    }
}
