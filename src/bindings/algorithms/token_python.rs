use pyo3::{create_exception, exceptions::PyException, Python};

use crate::internal::algorithms::cancellation::CancellationHandler;

/// A [CancellationHandler] that wraps any ohter [CancellationHandler] and also checks for Python
/// interrupts.
#[derive(Clone, Debug, Default)]
pub struct CancelTokenPython(Box<dyn CancellationHandler>);

impl CancellationHandler for CancelTokenPython {
    fn is_cancelled(&self) -> bool {
        if self.0.is_cancelled() {
            return true;
        }

        Python::with_gil(|py| py.check_signals()).is_err()
    }

    fn start_timer(&self) {
        self.0.start_timer()
    }
}

impl CancelTokenPython {
    pub fn with_inner<T: CancellationHandler + 'static>(handler: T) -> Self {
        CancelTokenPython(Box::new(handler))
    }
}

create_exception!(cancellation, CancelledError, PyException);
