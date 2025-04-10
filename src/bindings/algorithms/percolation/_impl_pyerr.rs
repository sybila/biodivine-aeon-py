use pyo3::PyErr;

use crate::bindings::algorithms::{
    cancellation::CancelledError, percolation::percolation_error::PercolationError,
};

impl From<PercolationError> for PyErr {
    fn from(err: PercolationError) -> Self {
        match err {
            PercolationError::CancelledEmpty => PyErr::new::<CancelledError, _>("Cancelled"),
        }
    }
}
