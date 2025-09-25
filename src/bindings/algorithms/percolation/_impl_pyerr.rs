use pyo3::PyErr;

use crate::{
    bindings::algorithms::{
        graph_representation::CreationFailedError, token_python::CancelledError,
    },
    internal::algorithms::percolation::PercolationError,
};

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
