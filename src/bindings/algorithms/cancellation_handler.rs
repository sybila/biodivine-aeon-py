use dyn_clone::{clone_trait_object, DynClone};
use log::info;
use std::{
    fmt::Debug,
    sync::{
        atomic::{AtomicBool, Ordering::SeqCst},
        Arc,
    },
    time::Duration,
};

/// An implementation of [CancellationHandler] that cannot be cancelled.
#[derive(Copy, Clone, Debug)]
pub struct CancelTokenNever;

/// A [CancellationHandler] implemented using atomics that can be shared between multiple threads.
#[derive(Clone, Debug, Default)]
pub struct CancelTokenAtomic(Arc<AtomicBool>);

/// A [CancellationHandler] implemented using a timer that can be started for a specified duration.
#[derive(Clone, Debug, Default)]
pub struct CancelTokenTimer {
    cancelled: Arc<AtomicBool>,
    started: Arc<AtomicBool>,
}

pub trait CancellationHandler: Send + Sync + DynClone {
    /// Returns `true` if the computation associated with this handler is cancelled.
    ///
    /// (This usually checks some shared, thread-safe variable, like an atomic or mutex
    /// that is set by the thread responsible for user interactions)
    fn is_cancelled(&self) -> bool;
}

clone_trait_object!(CancellationHandler);

impl CancellationHandler for CancelTokenNever {
    fn is_cancelled(&self) -> bool {
        false
    }
}

impl CancellationHandler for CancelTokenAtomic {
    fn is_cancelled(&self) -> bool {
        self.0.load(SeqCst)
    }
}

impl CancellationHandler for CancelTokenTimer {
    fn is_cancelled(&self) -> bool {
        self.cancelled.load(SeqCst)
    }
}

impl CancelTokenAtomic {
    /// Signal cancellation through this token. Returns `true` if the token was successfully
    /// updated, and `false` if it was already cancelled by a previous action.
    pub fn cancel(&self) -> bool {
        !self.0.fetch_or(true, SeqCst)
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
