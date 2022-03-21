use crate::bindings::lib_param_bn::PySymbolicAsyncGraph;
use crate::AsNative;
use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;

impl AsNative<SymbolicAsyncGraph> for PySymbolicAsyncGraph {
    fn as_native(&self) -> &SymbolicAsyncGraph {
        &self.0
    }

    fn as_native_mut(&mut self) -> &mut SymbolicAsyncGraph {
        &mut self.0
    }
}
