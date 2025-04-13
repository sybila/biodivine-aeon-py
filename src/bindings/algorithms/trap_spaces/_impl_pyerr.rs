use pyo3::PyErr;

use crate::bindings::algorithms::{
    cancellation::CancelledError, configurable::CreationFailedError,
    trap_spaces::trap_spaces_error::TrapSpacesError,
};

impl From<TrapSpacesError> for PyErr {
    fn from(err: TrapSpacesError) -> Self {
        match err {
            TrapSpacesError::CancelledEmpty => PyErr::new::<CancelledError, _>("Cancelled"),
            // TrapSpacesError::BddSizeLimitExceeded(x) => {
            //     PyErr::new::<BddSizeLimitExceededError, _>(format!(
            //         "BDD size limit exceeded: {}",
            //         x.exact_cardinality()
            //     ))
            // }
            TrapSpacesError::CreationFailed(x) => {
                PyErr::new::<CreationFailedError, _>(format!("Config creation failed: {}", x))
            }
        }
    }
}

// TODO: docs - add fourth argument, documentation
// create_exception!(trap_spaces, BddSizeLimitExceededError, PyException);
