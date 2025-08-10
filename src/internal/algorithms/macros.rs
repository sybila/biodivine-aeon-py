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
    (target: $target:expr, size: $size:expr, $($arg:tt)*) => {
        let level = if $size > 100_000 {
            $crate::internal::algorithms::macros::Level::Debug
        } else {
            $crate::internal::algorithms::macros::Level::Trace
        };

        if $crate::internal::algorithms::macros::log_enabled!(target: $target, level) {
            $crate::internal::algorithms::macros::log!(target: $target, level, $($arg)*);
        }
    };
}

/// A macro to conditionally add the `#[pyclass]` attribute with a custom name
/// to a struct based on the `algorithms-pyo3-bindings` feature.
///
/// # Example:
/// ```rust
/// maybe_pyclass!(
///     "PercolationComp",
///     /// Implements subspace percolation over a [SymbolicAsyncGraph].
///     #[derive(Clone, Configurable)]
///     pub struct Percolation {
///         ...
///     }
/// )
/// ```
#[macro_export]
macro_rules! maybe_pyclass {
    (
        $python_name:literal,
        $(#[$meta:meta])*
        $vis:vis struct $name:ident $($rest:tt)*
    ) => {
        #[cfg(feature = "algorithms-pyo3-bindings")]
        use pyo3::pyclass;

        #[cfg(feature = "algorithms-pyo3-bindings")]
        #[pyclass(name = $python_name, module = "biodivine_aeon", frozen)]
        $(#[$meta])*
        $vis struct $name $($rest)*

        #[cfg(not(feature = "algorithms-pyo3-bindings"))]
        $(#[$meta])*
        $vis struct $name $($rest)*
    };
}
