use std::fmt::{Debug, Formatter, Result};

use dyn_clone::{DynClone, clone_trait_object};

use crate::internal::algorithms::cancellation::tokens::CancelTokenNever;

pub trait CancellationHandler: Send + Sync + DynClone {
    /// Returns `true` if the computation associated with this handler is cancelled.
    ///
    /// (This usually checks some shared, thread-safe variable, like an atomic or mutex
    /// that is set by the thread responsible for user interactions)
    fn is_cancelled(&self) -> bool;

    /// This is a no-op by default, but if cancellation is implemented using a timer,
    /// this function starts the timer.
    fn start_timer(&self) {}
}

impl Default for Box<dyn CancellationHandler> {
    fn default() -> Self {
        Box::new(CancelTokenNever)
    }
}

impl Debug for Box<dyn CancellationHandler> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("Box<dyn CancellationHandler>")
            .field("is_cancelled", &self.is_cancelled())
            .finish()
    }
}

clone_trait_object!(CancellationHandler);
