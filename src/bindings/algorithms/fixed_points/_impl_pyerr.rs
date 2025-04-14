use pyo3::{create_exception, exceptions::PyException, PyErr};

use crate::bindings::algorithms::{
    cancellation::CancelledError, fixed_points::fixed_points_error::FixedPointsError,
};

impl From<FixedPointsError> for PyErr {
    fn from(err: FixedPointsError) -> Self {
        match err {
            FixedPointsError::Cancelled(bdd) => PyErr::new::<CancelledError, _>(format!(
                "Cancelled: partial_result={}",
                bdd.exact_cardinality()
            )),
            FixedPointsError::BddSizeLimitExceeded(bdd) => {
                PyErr::new::<BddSizeLimitExceededError, _>(format!(
                    "BDD size limit exceeded: {}",
                    bdd.exact_cardinality()
                ))
            }
        }
    }
}

// TODO: docs - add fourth argument, documentation
create_exception!(fixed_points, BddSizeLimitExceededError, PyException);
