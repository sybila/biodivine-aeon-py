use biodivine_lib_bdd::BddVariable as BddVariableNative;
use biodivine_lib_param_bn::VariableId as VariableIdNative;
use macros::Wrapper;
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::types::PyTuple;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// A numeric identifier of a single *network variable* used within a `BooleanNetwork`
/// or a `RegulatoryGraph`.
///
/// It essentially behaves like a type-safe integer value:
/// ```python
/// a = VariableId(0)
/// b = VariableId(1)
/// assert a == eval(repr(a))
/// assert a != b
/// assert a < b
/// assert a <= a
/// assert str(a) == "v_0"
/// assert int(a) == 0
/// d = {a: True, b: False}
/// assert d[a] != d[b]
/// ```
///
/// The value of `VariableId` is frozen (i.e. immutable).
///
/// See also `VariableIdType`: In most cases where the ID can be "inferred from context",
/// a name can be also used to identify a network variable.
///  
#[pyclass(module = "biodivine_aeon", frozen)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Wrapper)]
pub struct VariableId(VariableIdNative);

#[pymethods]
impl VariableId {
    #[new]
    #[pyo3(signature = (value = 0))]
    pub fn new(value: usize) -> Self {
        Self(VariableIdNative::from_index(value))
    }

    pub fn __richcmp__(&self, other: &Self, op: CompareOp) -> bool {
        op.matches(self.cmp(other))
    }

    pub fn __str__(&self) -> String {
        format!("v_{}", self.0.to_index())
    }

    pub fn __repr__(&self) -> String {
        format!("VariableId({})", self.0.to_index())
    }

    pub fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    pub fn __index__(&self) -> usize {
        self.0.to_index()
    }

    pub fn __getnewargs__<'a>(&self, py: Python<'a>) -> PyResult<Bound<'a, PyTuple>> {
        PyTuple::new(py, [self.0.to_index()])
    }
}

/// A trait used together with [`VariableIdType`] to safely resolve a name, symbolic variable, or
/// a native ID into a [`VariableIdNative`]. It is possible to implement
/// a resolver that cannot resolve all of these types, in which case it should simply
/// return `None`.
pub trait VariableIdResolver {
    fn resolve_name(&self, name: &str) -> Option<VariableIdNative>;
    fn resolve_id(&self, id: usize) -> Option<VariableIdNative>;
    fn resolve_symbolic(&self, var: BddVariableNative) -> Option<VariableIdNative>;
    fn get_name(&self, id: VariableIdNative) -> String;
}

/// Implemented by types that can be resolved into [`VariableIdNative`] by [`VariableIdResolver`].
///
/// Mostly useful because we can also use it to provide blanket implementation for collection
/// resolution and similar tasks.
pub trait VariableIdResolvable {
    fn resolve<R: VariableIdResolver>(&self, resolver: &R) -> PyResult<VariableIdNative>;

    fn resolve_collection<
        I: IntoIterator<Item = Self>,
        O: FromIterator<VariableIdNative>,
        R: VariableIdResolver,
    >(
        collection: I,
        resolver: &R,
    ) -> PyResult<O>
    where
        Self: Sized,
    {
        collection
            .into_iter()
            .map(|it: Self| it.resolve(resolver))
            .collect()
    }
}

impl VariableIdResolver for biodivine_lib_param_bn::BooleanNetwork {
    fn resolve_name(&self, name: &str) -> Option<VariableIdNative> {
        self.as_graph().resolve_name(name)
    }

    fn resolve_id(&self, id: usize) -> Option<VariableIdNative> {
        self.as_graph().resolve_id(id)
    }

    fn resolve_symbolic(&self, var: BddVariableNative) -> Option<VariableIdNative> {
        self.as_graph().resolve_symbolic(var)
    }

    fn get_name(&self, id: VariableIdNative) -> String {
        self.as_graph().get_name(id)
    }
}

impl VariableIdResolver for biodivine_lib_param_bn::RegulatoryGraph {
    fn resolve_name(&self, name: &str) -> Option<VariableIdNative> {
        self.find_variable(name)
    }

    fn resolve_id(&self, id: usize) -> Option<VariableIdNative> {
        if id < self.num_vars() {
            Some(VariableIdNative::from_index(id))
        } else {
            None
        }
    }

    fn resolve_symbolic(&self, _var: BddVariableNative) -> Option<VariableIdNative> {
        None
    }

    fn get_name(&self, id: VariableIdNative) -> String {
        self.get_variable_name(id).clone()
    }
}

impl VariableIdResolver for biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph {
    fn resolve_name(&self, name: &str) -> Option<VariableIdNative> {
        self.symbolic_context().resolve_name(name)
    }

    fn resolve_id(&self, id: usize) -> Option<VariableIdNative> {
        self.symbolic_context().resolve_id(id)
    }

    fn resolve_symbolic(&self, var: BddVariableNative) -> Option<VariableIdNative> {
        self.symbolic_context().resolve_symbolic(var)
    }

    fn get_name(&self, id: VariableIdNative) -> String {
        self.symbolic_context().get_name(id)
    }
}

impl VariableIdResolver for biodivine_lib_param_bn::symbolic_async_graph::SymbolicContext {
    fn resolve_name(&self, name: &str) -> Option<VariableIdNative> {
        self.find_network_variable(name)
    }

    fn resolve_id(&self, id: usize) -> Option<VariableIdNative> {
        if id < self.num_state_variables() {
            Some(VariableIdNative::from_index(id))
        } else {
            None
        }
    }

    fn resolve_symbolic(&self, var: BddVariableNative) -> Option<VariableIdNative> {
        self.find_state_variable(var)
    }

    fn get_name(&self, id: VariableIdNative) -> String {
        self.get_network_variable_name(id)
    }
}
