use pyo3::{create_exception, exceptions::PyException, PyErr};

use crate::bindings::algorithms::{cancellation::CancelledError, reachability::ReachabilityError};

impl From<ReachabilityError> for PyErr {
    fn from(err: ReachabilityError) -> Self {
        match err {
            ReachabilityError::Cancelled(x) => PyErr::new::<CancelledError, _>(format!(
                "Cancelled(partial_result={})",
                x.exact_cardinality()
            )),
            ReachabilityError::StepsLimitExceeded(x) => {
                PyErr::new::<StepsLimitExceededError, _>(format!(
                    "StepsLimitExceeded(partial_result={})",
                    x.exact_cardinality()
                ))
            }
            ReachabilityError::BddSizeLimitExceeded(x) => {
                PyErr::new::<BddSizeLimitExceededError, _>(format!(
                    "BddSizeLimitExceeded(partial_result={})",
                    x.exact_cardinality()
                ))
            }
            ReachabilityError::InvalidSubgraph => {
                PyErr::new::<InvalidSubgraphError, _>("InvalidSubgraph")
            }
        }
    }
}

// TODO: ohtenkay - add fourth argument, documentation
create_exception!(reachability, StepsLimitExceededError, PyException);
create_exception!(reachability, BddSizeLimitExceededError, PyException);
create_exception!(reachability, InvalidSubgraphError, PyException);
