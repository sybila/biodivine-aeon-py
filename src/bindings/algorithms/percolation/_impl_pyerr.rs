use pyo3::PyErr;

use crate::bindings::algorithms::{
    cancellation::CancelledError, percolation::percolation_error::PercolationError,
};

impl From<PercolationError> for PyErr {
    fn from(err: PercolationError) -> Self {
        match err {
            PercolationError::Cancelled(x) => {
                PyErr::new::<CancelledError, _>(format!("Cancelled(partial_result={:#?})", x))
            }
        }
    }
}
