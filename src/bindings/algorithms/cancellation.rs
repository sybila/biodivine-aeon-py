use crate::bindings::algorithms::{
    cancellation_error::CancellationError, cancellation_handler::CancellationHandler,
};
use std::fmt::Debug;

#[macro_export]
macro_rules! is_cancelled {
    ($handler:expr) => {
        $crate::bindings::algorithms::cancellation::test_cancellation($handler)
    };
    ($handler:expr, $partial:expr) => {
        $crate::bindings::algorithms::cancellation::test_with_partial($handler, $partial)
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
/// be returned if the operation is cancelled. The value is returned back as `Ok` if
/// the operation is not cancelled (so there should be no need to clone it).
pub fn test_with_partial<C, T>(handler: &C, partial: T) -> Result<T, CancellationError<T>>
where
    C: CancellationHandler,
    T: Sized + Debug + 'static,
{
    if handler.is_cancelled() {
        Err(CancellationError::with_partial_data(partial))
    } else {
        Ok(partial)
    }
}
