use dyn_clone::DynClone;
use pyo3::{create_exception, exceptions::PyException};

use crate::bindings::algorithms::cancellation::CancellationHandler;

pub trait Config {
    fn cancellation(&self) -> &dyn CancellationHandler;
}

pub trait Configurable {
    type ConfigType: Config;

    fn config(&self) -> &Self::ConfigType;
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
