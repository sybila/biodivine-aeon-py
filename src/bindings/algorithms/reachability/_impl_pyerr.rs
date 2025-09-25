use pyo3::{PyErr, create_exception, exceptions::PyException};

use crate::{
    bindings::algorithms::{
        graph_representation::CreationFailedError, token_python::CancelledError,
    },
    internal::algorithms::reachability::ReachabilityError,
};

impl From<ReachabilityError> for PyErr {
    fn from(err: ReachabilityError) -> Self {
        match err {
            ReachabilityError::CreationFailed(error) => {
                PyErr::new::<CreationFailedError, _>(format!("Config creation failed: {}", error))
            }
            ReachabilityError::Cancelled(gcv) => PyErr::new::<CancelledError, _>(format!(
                "Cancelled: partial_result={}",
                gcv.exact_cardinality()
            )),
            ReachabilityError::StepsLimitExceeded(gcv) => {
                PyErr::new::<StepsLimitExceededError, _>(format!(
                    "Steps limit exceeded: partial_result={}",
                    gcv.exact_cardinality()
                ))
            }
            ReachabilityError::BddSizeLimitExceeded(gcv) => {
                PyErr::new::<BddSizeLimitExceededError, _>(format!(
                    "BDD size limit exceeded: partial_result={}",
                    gcv.exact_cardinality()
                ))
            }
            ReachabilityError::InvalidSubgraph => {
                PyErr::new::<InvalidSubgraphError, _>("Invalid subgraph")
            }
        }
    }
}

create_exception!(reachability, StepsLimitExceededError, PyException);
create_exception!(reachability, BddSizeLimitExceededError, PyException);
create_exception!(reachability, InvalidSubgraphError, PyException);
