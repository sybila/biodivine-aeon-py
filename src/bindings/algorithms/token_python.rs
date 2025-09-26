use pyo3::{Python, create_exception, exceptions::PyException};

use crate::internal::algorithms::cancellation::CancellationHandler;

/// A [CancellationHandler] that wraps any other [CancellationHandler] and also checks for Python
/// interrupts.
#[derive(Clone, Debug, Default)]
pub struct CancelTokenPython(Box<dyn CancellationHandler>);

impl CancellationHandler for CancelTokenPython {
    fn is_cancelled(&self) -> bool {
        if self.0.is_cancelled() {
            return true;
        }

        Python::attach(|py| py.check_signals()).is_err()
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

create_exception!(biodivine_aeon, CancelledError, PyException);
