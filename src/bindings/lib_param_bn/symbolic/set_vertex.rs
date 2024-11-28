use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ops::Not;

use biodivine_lib_bdd::Bdd as RsBdd;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::projected_iteration::{
    OwnedRawSymbolicIterator, RawProjection,
};
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, GraphVertices};
use biodivine_lib_param_bn::{ExtendedBoolean, Space};
use num_bigint::BigInt;
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::types::PyList;
use pyo3::IntoPyObjectExt;

use crate::bindings::lib_bdd::bdd::Bdd;
use crate::bindings::lib_param_bn::symbolic::model_vertex::VertexModel;
use crate::bindings::lib_param_bn::symbolic::set_color::ColorSet;
use crate::bindings::lib_param_bn::symbolic::set_colored_vertex::ColoredVertexSet;
use crate::bindings::lib_param_bn::symbolic::set_spaces::SpaceSet;
use crate::bindings::lib_param_bn::symbolic::symbolic_context::SymbolicContext;
use crate::bindings::lib_param_bn::symbolic::symbolic_space_context::SymbolicSpaceContext;
use crate::bindings::lib_param_bn::variable_id::VariableId;
use crate::bindings::lib_param_bn::NetworkVariableContext;
use crate::AsNative;

/// A symbolic representation of a set of "vertices", i.e. valuations of variables
/// of a particular `BooleanNetwork`.
#[pyclass(module = "biodivine_aeon", frozen)]
#[derive(Clone)]
pub struct VertexSet {
    ctx: Py<SymbolicContext>,
    native: GraphVertices,
}

/// An internal class used for iterating over `VertexModel` instances of a `VertexSet`.
#[pyclass(module = "biodivine_aeon")]
pub struct _VertexModelIterator {
    ctx: Py<SymbolicContext>,
    native: OwnedRawSymbolicIterator,
}

impl AsNative<GraphVertices> for VertexSet {
    fn as_native(&self) -> &GraphVertices {
        &self.native
    }

    fn as_native_mut(&mut self) -> &mut GraphVertices {
        &mut self.native
    }
}

#[pymethods]
impl VertexSet {
    /// Normally, a new `VertexSet` is derived using an `AsynchronousGraph`. However, in some
    /// cases you may want to create it manually from a `SymbolicContext` and a `Bdd`.
    ///
    /// Just keep in mind that this method does not check that the provided `Bdd` is semantically
    /// a valid set of vertices.
    #[new]
    pub fn new(py: Python, ctx: Py<SymbolicContext>, bdd: &Bdd) -> Self {
        Self {
            ctx: ctx.clone(),
            native: GraphVertices::new(bdd.as_native().clone(), ctx.borrow(py).as_native()),
        }
    }

    pub fn __richcmp__(&self, py: Python, other: &Self, op: CompareOp) -> PyResult<Py<PyAny>> {
        match op {
            CompareOp::Eq => VertexSet::semantic_eq(self, other).into_py_any(py),
            CompareOp::Ne => VertexSet::semantic_eq(self, other).not().into_py_any(py),
            _ => Ok(py.NotImplemented()),
        }
    }

    pub fn __str__(&self) -> String {
        format!(
            "VertexSet(cardinality={}, symbolic_size={})",
            self.cardinality(),
            self.symbolic_size(),
        )
    }

    pub fn __repr__(&self) -> String {
        format!(
            "VertexSet(cardinality={}, symbolic_size={})",
            self.cardinality(),
            self.symbolic_size(),
        )
    }

    pub fn __copy__(self_: Py<Self>) -> Py<Self> {
        self_.clone()
    }

    pub fn __deepcopy__(self_: Py<Self>, _memo: &Bound<'_, PyAny>) -> Py<Self> {
        self_.clone()
    }

    pub fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.as_native().hash(&mut hasher);
        hasher.finish()
    }

    pub fn __iter__(&self) -> PyResult<_VertexModelIterator> {
        self.items(None)
    }

    pub fn __ctx__(&self) -> Py<SymbolicContext> {
        self.ctx.clone()
    }

    /// Returns the number of vertices that are represented in this set.
    pub fn cardinality(&self) -> BigInt {
        self.as_native().exact_cardinality()
    }

    /// Set intersection.
    pub fn intersect(&self, other: &Self) -> Self {
        self.mk_derived(self.as_native().intersect(other.as_native()))
    }

    /// Set difference.
    pub fn minus(&self, other: &Self) -> Self {
        self.mk_derived(self.as_native().minus(other.as_native()))
    }

    /// Set union.
    pub fn union(&self, other: &Self) -> Self {
        self.mk_derived(self.as_native().union(other.as_native()))
    }

    /// True if this set is empty.
    pub fn is_empty(&self) -> bool {
        self.as_native().is_empty()
    }

    /// True if this set is a subset of the other set.
    ///
    /// Should be faster than just calling `set.minus(superset).is_empty()`
    pub fn is_subset(&self, other: &Self) -> bool {
        self.as_native().is_subset(other.as_native())
    }

    /// True if this set is a singleton, i.e. a single vertex.
    pub fn is_singleton(&self) -> bool {
        self.as_native().is_singleton()
    }

    /// True if this set is a subspace, i.e. it can be expressed using a single conjunctive clause.
    pub fn is_subspace(&self) -> bool {
        self.as_native().is_subspace()
    }

    /// Deterministically pick a subset of this set that contains exactly a single vertex.
    ///
    /// If this set is empty, the result is also empty.
    pub fn pick_singleton(&self) -> Self {
        self.mk_derived(self.as_native().pick_singleton())
    }

    /// The number of `Bdd` nodes that are used to represent this set.
    pub fn symbolic_size(&self) -> usize {
        self.as_native().symbolic_size()
    }

    /// Obtain the underlying `Bdd` of this `VertexSet`.
    pub fn to_bdd(&self, py: Python) -> Bdd {
        let rs_bdd = self.as_native().as_bdd().clone();
        let ctx = self.ctx.borrow(py);
        Bdd::new_raw_2(ctx.bdd_variable_set(), rs_bdd)
    }

    /// Extend this set of vertices with all the colors from the given set.
    ///
    /// This is essentially a cartesian product with the given `ColorSet`.
    pub fn extend_with_colors(&self, colors: &ColorSet) -> ColoredVertexSet {
        let colors = colors.as_native().as_bdd();
        let bdd = self.native.as_bdd().and(colors);
        let ctx = self.ctx.get();
        let native_set = GraphColoredVertices::new(bdd, ctx.as_native());
        ColoredVertexSet::mk_native(self.ctx.clone(), native_set)
    }

    /// Returns an iterator over all vertices in this `VertexSet` with an optional projection to a subset
    /// of network variables.
    ///
    /// When no `retained` collection is specified, this is equivalent to `VertexSet.__iter__`. However, if a retained
    /// set is given, the resulting iterator only considers unique combinations of the `retained` variables.
    /// Consequently, the resulting `VertexModel` instances will fail with an `IndexError` if a value of a variable
    /// outside the `retained` set is requested.
    #[pyo3(signature = (retained = None))]
    pub fn items(&self, retained: Option<&Bound<'_, PyList>>) -> PyResult<_VertexModelIterator> {
        let ctx = self.ctx.get();
        let retained = if let Some(retained) = retained {
            retained
                .iter()
                .map(|it| {
                    ctx.resolve_network_variable(&it)
                        .map(|it| ctx.as_native().get_state_variable(it))
                })
                .collect::<PyResult<Vec<_>>>()?
        } else {
            self.ctx.get().as_native().state_variables().clone()
        };
        let projection = RawProjection::new(retained, self.as_native().as_bdd());
        Ok(_VertexModelIterator {
            ctx: self.ctx.clone(),
            native: projection.into_iter(),
        })
    }

    /// Represent this set of vertices as a set of singleton subspaces instead.
    pub fn to_singleton_spaces(&self, ctx: Py<SymbolicSpaceContext>) -> SpaceSet {
        let native = self.as_native().to_singleton_spaces(ctx.get().as_native());
        SpaceSet::wrap_native(ctx, native)
    }

    /// Compute the smallest enclosing subspace, represented as `dict[VariableId, bool]` with
    /// missing variables being treated as unrestricted.
    ///
    /// Returns `None` if the set is empty.
    pub fn enclosing_subspace(&self) -> Option<HashMap<VariableId, bool>> {
        if self.is_empty() {
            None
        } else {
            Some(
                self.enclosing_subspace_native()
                    .to_values()
                    .into_iter()
                    .map(|(a, b)| (a.into(), b))
                    .collect(),
            )
        }
    }

    /// Compute the smallest enclosing subspace, represented as `dict[str, bool]`, using variable
    /// names as keys and with missing variables being treated as unrestricted.
    ///
    /// Returns `None` if the set is empty.
    pub fn enclosing_named_subspace(&self) -> Option<HashMap<String, bool>> {
        let ctx = self.ctx.get().as_native();
        if self.is_empty() {
            None
        } else {
            Some(
                self.enclosing_subspace_native()
                    .to_values()
                    .into_iter()
                    .map(|(a, b)| (ctx.get_network_variable_name(a), b))
                    .collect(),
            )
        }
    }

    /// Compute the largest subspace that is fully enclosed in this vertex set. Note that such
    /// subspace may not be unique (i.e. there can be other subspaces that are just as large).
    ///
    /// Returns `None` if the set is empty.
    pub fn enclosed_subspace(&self) -> Option<HashMap<VariableId, bool>> {
        let ctx = self.ctx.get().as_native();
        let bdd = self.as_native().as_bdd();
        let clause = bdd.most_free_clause()?;
        Some(
            clause
                .to_values()
                .into_iter()
                .map(|(bdd_var, value)| {
                    let network_var = ctx
                        .find_state_variable(bdd_var)
                        .expect("Expected network variable.");
                    (VariableId::from(network_var), value)
                })
                .collect(),
        )
    }

    /// Same as `VertexSet.enclosed_subspace`, but uses names instead of IDs.
    pub fn enclosed_named_subspace(&self) -> Option<HashMap<String, bool>> {
        let ctx = self.ctx.get().as_native();
        let bdd = self.as_native().as_bdd();
        let clause = bdd.most_free_clause()?;
        Some(
            clause
                .to_values()
                .into_iter()
                .map(|(bdd_var, value)| {
                    let network_var = ctx
                        .find_state_variable(bdd_var)
                        .expect("Expected network variable.");
                    let name = ctx.get_network_variable_name(network_var);
                    (name, value)
                })
                .collect(),
        )
    }
}

impl VertexSet {
    pub fn mk_native(ctx: Py<SymbolicContext>, native: GraphVertices) -> Self {
        Self { ctx, native }
    }

    pub fn mk_derived(&self, native: GraphVertices) -> Self {
        Self {
            ctx: self.ctx.clone(),
            native,
        }
    }

    pub fn semantic_eq(a: &Self, b: &Self) -> bool {
        let a = a.as_native().as_bdd();
        let b = b.as_native().as_bdd();
        if a.num_vars() != b.num_vars() {
            return false;
        }

        RsBdd::binary_op_with_limit(1, a, b, biodivine_lib_bdd::op_function::xor).is_some()
    }

    /// Returns the smallest enclosing subspace, as long as the set is not empty.
    pub fn enclosing_subspace_native(&self) -> Space {
        let ctx = self.ctx.get().as_native();
        let mut space = Space::new_raw(ctx.num_state_variables());
        for var in ctx.network_variables() {
            let bdd_var = ctx.get_state_variable(var);
            let true_subset = self.native.as_bdd().var_select(bdd_var, true);
            let false_subset = self.native.as_bdd().var_select(bdd_var, false);
            assert!(!true_subset.is_false() || !false_subset.is_false());
            match (true_subset.is_false(), false_subset.is_false()) {
                (true, true) => unreachable!("The set is empty!"),
                (false, false) => space[var] = ExtendedBoolean::Any,
                (false, true) => space[var] = ExtendedBoolean::One,
                (true, false) => space[var] = ExtendedBoolean::Zero,
            }
        }
        space
    }
}

#[pymethods]
impl _VertexModelIterator {
    fn __iter__(self_: Py<Self>) -> Py<Self> {
        self_
    }

    fn __next__(&mut self) -> Option<VertexModel> {
        self.native
            .next()
            .map(|it| VertexModel::new_native(self.ctx.clone(), it))
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<VertexModel> {
        self.__next__()
    }
}
