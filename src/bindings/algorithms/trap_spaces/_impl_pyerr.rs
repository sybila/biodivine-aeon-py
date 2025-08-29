use pyo3::{PyErr, create_exception, exceptions::PyException};

use crate::{
    bindings::algorithms::{
        graph_representation::CreationFailedError, token_python::CancelledError,
    },
    internal::algorithms::trap_spaces::TrapSpacesError,
};

impl From<TrapSpacesError> for PyErr {
    fn from(err: TrapSpacesError) -> Self {
        match err {
            TrapSpacesError::CreationFailed(error) => {
                PyErr::new::<CreationFailedError, _>(format!("Config creation failed: {}", error))
            }
            TrapSpacesError::Cancelled(bdd) => {
                PyErr::new::<CancelledError, _>(format!("Cancelled: {}", bdd.exact_cardinality()))
            }
            TrapSpacesError::BddSizeLimitExceeded(bdd) => {
                PyErr::new::<BddSizeLimitExceededError, _>(format!(
                    "BDD size limit exceeded: {}",
                    bdd.exact_cardinality()
                ))
            }
        }
    }
}

create_exception!(trap_spaces, BddSizeLimitExceededError, PyException);
