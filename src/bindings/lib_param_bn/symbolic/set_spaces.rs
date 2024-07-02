use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::ops::Not;

use biodivine_lib_bdd::Bdd as RsBdd;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::projected_iteration::{
    OwnedRawSymbolicIterator, RawProjection,
};
use biodivine_lib_param_bn::trap_spaces::{NetworkColoredSpaces, NetworkSpaces};
use num_bigint::BigInt;
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::types::PyList;

use crate::bindings::lib_bdd::bdd::Bdd;
use crate::bindings::lib_param_bn::symbolic::model_space::SpaceModel;
use crate::bindings::lib_param_bn::symbolic::set_color::ColorSet;
use crate::bindings::lib_param_bn::symbolic::set_colored_space::ColoredSpaceSet;
use crate::bindings::lib_param_bn::symbolic::set_vertex::VertexSet;
use crate::bindings::lib_param_bn::symbolic::symbolic_context::SymbolicContext;
use crate::bindings::lib_param_bn::symbolic::symbolic_space_context::SymbolicSpaceContext;
use crate::bindings::lib_param_bn::NetworkVariableContext;
use crate::AsNative;

/// A symbolic representation of a set of "spaces", i.e. hypercubes in the state space
/// of a particular `BooleanNetwork`.
#[pyclass(module = "biodivine_aeon", frozen)]
#[derive(Clone)]
pub struct SpaceSet {
    ctx: Py<SymbolicSpaceContext>,
    native: NetworkSpaces,
}

/// An internal class used for iterating over `SpaceModel` instances of a `VertexSet`.
#[pyclass(module = "biodivine_aeon")]
pub struct _SpaceModelIterator {
    ctx: Py<SymbolicSpaceContext>,
    native: OwnedRawSymbolicIterator,
}

impl AsNative<NetworkSpaces> for SpaceSet {
    fn as_native(&self) -> &NetworkSpaces {
        &self.native
    }

    fn as_native_mut(&mut self) -> &mut NetworkSpaces {
        &mut self.native
    }
}

#[pymethods]
impl SpaceSet {
    /// Normally, a new `SpaceSet` is derived using a `SymbolicSpaceContext`. However, in some
    /// cases you may want to create it manually from a `SymbolicSpaceContext` and a `Bdd`.
    ///
    /// Just keep in mind that this method does not check that the provided `Bdd` is semantically
    /// a valid set of spaces.
    #[new]
    pub fn new(ctx: Py<SymbolicSpaceContext>, bdd: &Bdd) -> Self {
        Self {
            ctx: ctx.clone(),
            native: NetworkSpaces::new(bdd.as_native().clone(), ctx.get().as_native()),
        }
    }

    fn __richcmp__(&self, py: Python, other: &Self, op: CompareOp) -> Py<PyAny> {
        match op {
            CompareOp::Eq => SpaceSet::semantic_eq(self, other).into_py(py),
            CompareOp::Ne => SpaceSet::semantic_eq(self, other).not().into_py(py),
            _ => py.NotImplemented(),
        }
    }

    fn __str__(&self) -> String {
        format!(
            "SpaceSet(cardinality={}, symbolic_size={})",
            self.cardinality(),
            self.symbolic_size(),
        )
    }

    fn __repr__(&self) -> String {
        format!(
            "SpaceSet(cardinality={}, symbolic_size={})",
            self.cardinality(),
            self.symbolic_size(),
        )
    }

    fn __copy__(self_: Py<Self>) -> Py<Self> {
        self_.clone()
    }

    fn __deepcopy__(self_: Py<Self>, _memo: &Bound<'_, PyAny>) -> Py<Self> {
        self_.clone()
    }

    fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.as_native().hash(&mut hasher);
        hasher.finish()
    }

    fn __iter__(&self, py: Python) -> PyResult<_SpaceModelIterator> {
        self.items(None, py)
    }

    pub fn __ctx__(&self) -> Py<SymbolicSpaceContext> {
        self.ctx.clone()
    }

    /// Returns the number of spaces that are represented in this set.
    pub fn cardinality(&self) -> BigInt {
        self.as_native().exact_cardinality()
    }

    /// Set intersection.
    fn intersect(&self, other: &Self) -> Self {
        self.mk_derived(self.as_native().intersect(other.as_native()))
    }

    /// Set difference.
    fn minus(&self, other: &Self) -> Self {
        self.mk_derived(self.as_native().minus(other.as_native()))
    }

    /// Set union.
    fn union(&self, other: &Self) -> Self {
        self.mk_derived(self.as_native().union(other.as_native()))
    }

    /// True if this set is empty.
    fn is_empty(&self) -> bool {
        self.as_native().is_empty()
    }

    /// True if this set is a subset of the other set.
    ///
    /// Should be faster than just calling `set.minus(superset).is_empty()`
    fn is_subset(&self, other: &Self) -> bool {
        self.as_native().is_subset(other.as_native())
    }

    /// True if this set is a singleton, i.e. a single subspace.
    fn is_singleton(&self) -> bool {
        self.as_native().is_singleton()
    }

    /// Deterministically pick a subset of this set that contains exactly a single subspace.
    ///
    /// If this set is empty, the result is also empty.
    fn pick_singleton(&self) -> Self {
        self.mk_derived(self.as_native().pick_singleton())
    }

    /// The number of `Bdd` nodes that are used to represent this set.
    fn symbolic_size(&self) -> usize {
        self.as_native().symbolic_size()
    }

    /// Obtain the underlying `Bdd` of this `SpaceSet`.
    fn to_bdd(&self, py: Python) -> Bdd {
        let rs_bdd = self.as_native().as_bdd().clone();
        let ctx = self.ctx.borrow(py);
        Bdd::new_raw_2(ctx.as_ref().bdd_variable_set(), rs_bdd)
    }

    /// Extend this set of spaces with all the colors from the given set.
    ///
    /// This is essentially a cartesian product with the given `ColorSet`.
    fn extend_with_colors(&self, colors: &ColorSet) -> ColoredSpaceSet {
        let colors = colors.as_native().as_bdd();
        let bdd = self.native.as_bdd().and(colors);
        let ctx = self.ctx.get();
        let native_set = NetworkColoredSpaces::new(bdd, ctx.as_native());
        ColoredSpaceSet::wrap_native(self.ctx.clone(), native_set)
    }

    /// Returns an iterator over all sub-spaces in this `SpaceSet` with an optional projection to
    /// a subset of network variables.
    ///
    /// When no `retained` collection is specified, this is equivalent to `SpaceSet.__iter__`.
    /// However, if a retained set is given, the resulting iterator only considers unique
    /// valuations for the `retained` variables. Consequently, the resulting `SpaceModel` instances
    /// will fail with an `IndexError` if a value of a variable outside the `retained` set is
    /// requested.
    #[pyo3(signature = (retained = None))]
    fn items(
        &self,
        retained: Option<&Bound<'_, PyList>>,
        py: Python,
    ) -> PyResult<_SpaceModelIterator> {
        let ctx = self.ctx.borrow(py);
        let retained = if let Some(retained) = retained {
            let mut retained_vars = Vec::new();
            for var in retained {
                let var = ctx.as_ref().resolve_network_variable(&var)?;
                retained_vars.push(ctx.as_native().get_positive_variable(var));
                retained_vars.push(ctx.as_native().get_negative_variable(var));
            }
            retained_vars
        } else {
            ctx.as_ref().as_native().all_extra_state_variables().clone()
        };
        let projection = RawProjection::new(retained, self.as_native().as_bdd());
        Ok(_SpaceModelIterator {
            ctx: self.ctx.clone(),
            native: projection.into_iter(),
        })
    }

    /// Produce a set of vertices that are contained within the subspaces represented in this set.
    pub fn to_vertices(&self, ctx: Py<SymbolicSpaceContext>, py: Python) -> PyResult<VertexSet> {
        let native = self.as_native().to_vertices(ctx.get().as_native());
        let parent = ctx.extract::<Py<SymbolicContext>>(py)?;
        Ok(VertexSet::mk_native(parent, native))
    }
}

impl SpaceSet {
    pub fn wrap_native(ctx: Py<SymbolicSpaceContext>, native: NetworkSpaces) -> SpaceSet {
        SpaceSet { ctx, native }
    }

    pub fn mk_derived(&self, native: NetworkSpaces) -> Self {
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
}

#[pymethods]
impl _SpaceModelIterator {
    fn __iter__(self_: Py<Self>) -> Py<Self> {
        self_
    }

    fn __next__(&mut self) -> Option<SpaceModel> {
        self.native
            .next()
            .map(|it| SpaceModel::new_native(self.ctx.clone(), it))
    }
}
