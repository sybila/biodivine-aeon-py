use std::{
    fmt::Debug,
    sync::{
        atomic::{AtomicBool, Ordering::SeqCst},
        Arc,
    },
    time::Duration,
};

use log::info;

use crate::bindings::algorithms::cancellation::CancellationHandler;

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
    time_limit: Duration,
}

impl CancellationHandler for CancelTokenTimer {
    fn is_cancelled(&self) -> bool {
        assert!(self.started.load(SeqCst));

        self.cancelled.load(SeqCst)
    }

    fn start_timer(&self) {
        if self.cancel_after(self.time_limit) {
            info!(target: "cancellation", "Timer for {}ms started.", self.time_limit.as_millis());
        } else {
            info!(target: "cancellation", "Timer for {}ms already started.", self.time_limit.as_millis());
        }
    }
}

impl CancelTokenTimer {
    /// Create a new timer with the specified `duration` that is not running.
    pub fn new(duration: Duration) -> CancelTokenTimer {
        CancelTokenTimer {
            cancelled: Default::default(),
            started: Default::default(),
            time_limit: duration,
        }
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
                std::thread::sleep(duration);
                thread_copy.cancelled.store(true, SeqCst);
                info!(target: "cancellation", "Timer for {}ms elapsed. Operation cancelled.", duration.as_millis());
            });
        }

        !is_started
    }
}

/* Timer - End */
