use std::convert::TryFrom;

use biodivine_lib_param_bn::BooleanNetwork;
use dyn_clone::DynClone;
use pyo3::{create_exception, exceptions::PyException};

use crate::bindings::algorithms::cancellation::CancellationHandler;

// TODO: discuss - enforces try_from, but not from, due to needing SymbolicSpaceContext in TrapSpacesConfig
pub trait Config: for<'a> TryFrom<&'a BooleanNetwork> {
    fn cancellation(&self) -> &dyn CancellationHandler;

    fn set_cancellation(&mut self, cancellation: Box<dyn CancellationHandler>);

    /// Update the `cancellation` property, automatically wrapping the [CancellationHandler]
    /// in a `Box`.
    fn with_cancellation<C: CancellationHandler + 'static>(mut self, cancellation: C) -> Self
    where
        Self: Sized,
    {
        self.set_cancellation(Box::new(cancellation));
        self
    }

    /// Update the `cancellation` property, without wrapping the [CancellationHandler]
    /// in a `Box`.
    fn with_cancellation_nowrap(mut self, cancellation: Box<dyn CancellationHandler>) -> Self {
        self.set_cancellation(cancellation);
        self
    }
}

// TODO: discuss - also create a macro for this?
pub trait Configurable {
    type ConfigType: Config;

    fn config(&self) -> &Self::ConfigType;

    fn with_config(config: Self::ConfigType) -> Self;
}

impl<T: Configurable + Send + Sync + DynClone> CancellationHandler for T {
    fn is_cancelled(&self) -> bool {
        self.config().cancellation().is_cancelled()
    }

    fn start_timer(&self) {
        self.config().cancellation().start_timer()
    }
}

// TODO: docs - add fourth argument, documentation
create_exception!(configurable, CreationFailedError, PyException);
