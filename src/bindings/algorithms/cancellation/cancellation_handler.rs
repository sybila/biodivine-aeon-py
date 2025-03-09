use dyn_clone::{clone_trait_object, DynClone};
use std::fmt::{Debug, Formatter, Result};

use crate::bindings::algorithms::cancellation::tokens::CancelTokenNever;

pub trait CancellationHandler: Send + Sync + DynClone {
    /// Returns `true` if the computation associated with this handler is cancelled.
    ///
    /// (This usually checks some shared, thread-safe variable, like an atomic or mutex
    /// that is set by the thread responsible for user interactions)
    fn is_cancelled(&self) -> bool;

    // TODO: ohtenkay - Consider using enum to encapsulate tokens
    // fn set_inner<T: CancellationHandler>(&mut self, handler: T) {}
}

clone_trait_object!(CancellationHandler);

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
