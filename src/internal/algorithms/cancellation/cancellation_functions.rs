use std::fmt::Debug;

use crate::internal::algorithms::cancellation::{CancellationError, CancellationHandler};

#[macro_export]
macro_rules! is_cancelled {
    ($handler:expr) => {
        $crate::internal::algorithms::cancellation::test_cancellation($handler)
    };
    ($handler:expr, $partial:expr) => {
        $crate::internal::algorithms::cancellation::test_with_partial($handler, $partial)
    };
}

/// Test if the operation is cancelled and return an error if it is. This is useful
/// in functions that already return `Result<X, E>`, since we can use the `?` operator to
/// test for cancellation, as long as `CancellationError` can
/// be automatically converted into `E`.
pub fn test_cancellation<C>(handler: &C) -> Result<(), CancellationError<()>>
where
    C: CancellationHandler,
{
    if handler.is_cancelled() {
        Err(CancellationError::default())
    } else {
        Ok(())
    }
}

/// The same as [CancellationHandler::test], but you can provide a partial result that will
/// be returned if the operation is cancelled.
pub fn test_with_partial<C, T, Comp>(handler: &C, partial: Comp) -> Result<(), CancellationError<T>>
where
    C: CancellationHandler,
    T: Sized + Debug + 'static,
    Comp: FnOnce() -> T,
{
    if handler.is_cancelled() {
        Err(CancellationError::with_partial_data(partial()))
    } else {
        Ok(())
    }
}
