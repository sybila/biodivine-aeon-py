mod _impl_symbolic_space_context;
mod trap_spaces_config;
mod trap_spaces_error;
mod trap_spaces_impl;

pub use _impl_symbolic_space_context::SymbolicSpaceContextExt;
pub use trap_spaces_config::TrapSpacesConfig;
pub use trap_spaces_error::TrapSpacesError;
pub use trap_spaces_impl::TrapSpaces;

#[cfg(feature = "algorithms-pyo3-bindings")]
mod mod_python;
#[cfg(feature = "algorithms-pyo3-bindings")]
pub use mod_python::*;
