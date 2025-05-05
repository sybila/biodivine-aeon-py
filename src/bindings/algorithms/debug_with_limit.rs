#[doc(hidden)]
pub use log::{log, log_enabled, Level};

/// A macro for logging messages with level determined by the size of data structure.
///
/// If you set the log level to Debug, only large data structures (size > 100,000), will be logged.
/// If you wish to see ALL logs for smaller data structures, set the log level to Trace.
///
/// # Example:
/// ```
/// debug_with_limit!(target: "MyModule", size: data.len(), "Processing data: {}", data);
/// ```
#[macro_export]
macro_rules! debug_with_limit {
    (target: $target:expr, size: $size:expr, $($arg:tt)*) => {{
        let level = if $size > 100_000 {
            $crate::bindings::algorithms::debug_with_limit::Level::Debug
        } else {
            $crate::bindings::algorithms::debug_with_limit::Level::Trace
        };

        if $crate::bindings::algorithms::debug_with_limit::log_enabled!(target: $target, level) {
            $crate::bindings::algorithms::debug_with_limit::log!(target: $target, level, $($arg)*);
        }
    }};
}
