use std::fmt::Debug;

use pyo3::{create_exception, exceptions::PyException};

/// Represents the fact that the operation has been cancelled before completion.
///
/// The struct can contain the partial results of the cancelled operation. If you don't intend
/// to use this feature, simply use `()` as the "partial result".
#[derive(Debug)]
pub struct CancellationError<T: Sized + Debug + 'static>(T);

impl<T: Sized + Debug + 'static> CancellationError<T> {
    pub fn with_partial_data(value: T) -> CancellationError<T> {
        CancellationError(value)
    }

    pub fn partial_data(&self) -> &T {
        &self.0
    }

    pub fn into_partial_data(self) -> T {
        self.0
    }
}

impl<T: Sized + Debug + 'static> From<T> for CancellationError<T> {
    fn from(value: T) -> Self {
        CancellationError(value)
    }
}

impl<T: Sized + Debug + 'static + Default> Default for CancellationError<T> {
    fn default() -> Self {
        CancellationError(Default::default())
    }
}

create_exception!(cancellation, CancelledError, PyException);
