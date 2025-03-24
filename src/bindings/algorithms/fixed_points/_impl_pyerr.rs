use pyo3::{create_exception, exceptions::PyException, PyErr};

use crate::bindings::algorithms::{
    cancellation::CancelledError, fixed_points::fixed_points_error::FixedPointsError,
};

impl From<FixedPointsError> for PyErr {
    fn from(err: FixedPointsError) -> Self {
        match err {
            FixedPointsError::CancelledEmpty => PyErr::new::<CancelledError, _>("Cancelled"),
        }
    }
}

// TODO: docs - add fourth argument, documentation
create_exception!(fixed_points, BddSizeLimitExceededError, PyException);
