use crate::bindings::lib_param_bn::boolean_network::BooleanNetwork;
use crate::AsNative;
use biodivine_lib_param_bn::FnUpdate;
use pyo3::prelude::*;
use std::sync::Arc;

#[pyclass(module = "biodivine_aeon")]
#[derive(Clone)]
pub struct UpdateFunction {
    ctx: Py<BooleanNetwork>,
    root: Arc<FnUpdate>,
    value: &'static FnUpdate,
}

#[pymethods]
impl UpdateFunction {
    fn __str__(&self, py: Python) -> String {
        self.value.to_string(self.ctx.borrow(py).as_native())
    }
}

impl UpdateFunction {
    pub fn new_raw(ctx: Py<BooleanNetwork>, native: FnUpdate) -> UpdateFunction {
        let root = Arc::new(native);
        let value: &'static FnUpdate =
            unsafe { (root.as_ref() as *const FnUpdate).as_ref().unwrap() };
        UpdateFunction { ctx, root, value }
    }

    pub fn as_native(&self) -> &FnUpdate {
        self.value
    }
}
