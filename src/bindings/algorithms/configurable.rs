use dyn_clone::DynClone;
use pyo3::{create_exception, exceptions::PyException};

use crate::bindings::algorithms::cancellation::CancellationHandler;

pub trait Config {
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
}

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
