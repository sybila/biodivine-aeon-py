use log::info;
use pyo3::Python;
use std::{
    fmt::Debug,
    sync::{
        atomic::{AtomicBool, Ordering::SeqCst},
        Arc,
    },
    time::Duration,
};

use crate::bindings::algorithms::cancellation::CancellationHandler;

// TODO: ohtenkay - nice to have - consider trying this with an enum

/* Never - Start */

/// An implementation of [CancellationHandler] that cannot be cancelled.
#[derive(Copy, Clone, Debug)]
pub struct CancelTokenNever;

impl CancellationHandler for CancelTokenNever {
    fn is_cancelled(&self) -> bool {
        false
    }
}

/* Never - End */

/* Atomic - Start */

/// A [CancellationHandler] implemented using atomics that can be shared between multiple threads.
#[derive(Clone, Debug, Default)]
pub struct CancelTokenAtomic(Arc<AtomicBool>);

impl CancellationHandler for CancelTokenAtomic {
    fn is_cancelled(&self) -> bool {
        self.0.load(SeqCst)
    }
}

impl CancelTokenAtomic {
    /// Signal cancellation through this token. Returns `true` if the token was successfully
    /// updated, and `false` if it was already cancelled by a previous action.
    pub fn cancel(&self) -> bool {
        !self.0.fetch_or(true, SeqCst)
    }
}

/* Atomic - End */

/* Timer - Start */

/// A [CancellationHandler] implemented using a timer that can be started for a specified duration.
#[derive(Clone, Debug, Default)]
pub struct CancelTokenTimer {
    cancelled: Arc<AtomicBool>,
    started: Arc<AtomicBool>,
}

impl CancellationHandler for CancelTokenTimer {
    fn is_cancelled(&self) -> bool {
        self.cancelled.load(SeqCst)
    }
}

impl CancelTokenTimer {
    /// Create a new timer for the specified `duration` that is immediately running.
    pub fn start(duration: Duration) -> CancelTokenTimer {
        let timer = Self::default();
        assert!(timer.cancel_after(duration));
        timer
    }

    /// Start the cancellation timer for the given `duration`.
    ///
    /// Returns `true` if the timer was started successfully, and `false` if the timer
    /// is either running, or the token is already cancelled.
    pub fn cancel_after(&self, duration: Duration) -> bool {
        let is_started = self.started.fetch_or(true, SeqCst);
        if !is_started {
            let thread_copy = self.clone();
            std::thread::spawn(move || {
                info!(target: "cancellation", "Timer for {}ms started.", duration.as_millis());
                std::thread::sleep(duration);
                thread_copy.cancelled.store(true, SeqCst);
                info!(target: "cancellation", "Timer for {}ms elapsed. Operation cancelled.", duration.as_millis());
            });
        }
        !is_started
    }
}

/* Timer - End */

/* Python - Start */

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
}

impl CancelTokenPython {
    pub fn with_inner<T: CancellationHandler + 'static>(handler: T) -> Self {
        CancelTokenPython(Box::new(handler))
    }

    pub fn set_inner<T: CancellationHandler + 'static>(&mut self, handler: T) {
        self.0 = Box::new(handler)
    }
}

/* Python - End */
