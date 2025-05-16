mod cancellation_error;
mod cancellation_functions;
mod cancellation_handler;
mod tokens;

pub use cancellation_error::CancellationError;
pub use cancellation_functions::{test_cancellation, test_with_partial};
pub use cancellation_handler::CancellationHandler;
pub use tokens::*;

#[cfg(feature = "algorithms-pyo3-bindings")]
mod token_python;
#[cfg(feature = "algorithms-pyo3-bindings")]
pub use token_python::{CancelTokenPython, CancelledError};
