use std::convert::TryFrom;

use biodivine_lib_param_bn::BooleanNetwork;
use dyn_clone::DynClone;

use crate::bindings::algorithms::cancellation::CancellationHandler;

// TODO: once we are able to create symbolic space context from a graph, add a trait bound
// for From<SymbolicSpaceContext>, also add to Configurable
pub trait Config: for<'a> TryFrom<&'a BooleanNetwork> {
    fn cancellation(&self) -> &dyn CancellationHandler;

    fn set_cancellation(&mut self, cancellation: Box<dyn CancellationHandler>);

    /// Update the `cancellation` property, automatically wrapping the [CancellationHandler]
    /// in a `Box`.
    fn with_cancellation<C>(mut self, cancellation: C) -> Self
    where
        C: CancellationHandler + 'static,
        Self: Sized,
    {
        self.set_cancellation(Box::new(cancellation));
        self
    }

    /// Update the `cancellation` property. For internal use only. Do not use for the Creation API.
    fn with_cancellation_nowrap(mut self, cancellation: Box<dyn CancellationHandler>) -> Self {
        self.set_cancellation(cancellation);
        self
    }
}

pub trait Configurable: for<'a> TryFrom<&'a BooleanNetwork> {
    type ConfigType: Config;

    /// Retrieve the internal configuration struct of this instance.
    fn config(&self) -> &Self::ConfigType;

    /// Retrieve the internal configuration struct of this instance by transferring ownership.
    fn into_config(self) -> Self::ConfigType;

    /// Create a new instance with the given configuration struct.
    fn with_config(config: Self::ConfigType) -> Self;
}

impl<T> CancellationHandler for T
where
    T: Configurable + Send + Sync + DynClone,
{
    fn is_cancelled(&self) -> bool {
        self.config().cancellation().is_cancelled()
    }

    fn start_timer(&self) {
        self.config().cancellation().start_timer()
    }
}
