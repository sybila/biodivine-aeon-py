use crate::bindings::lib_bdd::bdd::Bdd;
use crate::bindings::lib_param_bn::symbolic::color_set::ColorSet;
use crate::bindings::lib_param_bn::symbolic::symbolic_context::SymbolicContext;
use crate::bindings::lib_param_bn::symbolic::vertex_set::VertexSet;
use crate::AsNative;
use biodivine_lib_bdd::Bdd as RsBdd;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices;
use num_bigint::BigInt;
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::ops::Not;

/// A symbolic representation of a relation of "coloured vertices", i.e. pairs of vertices
/// (see `VertexSet`) and colors (see `ColorSet`).
///
/// Together, such pair represents a specific interpretation of network parameters and
/// valuation of network variables.
///
#[pyclass(module = "biodivine_aeon", frozen)]
#[derive(Clone)]
pub struct ColoredVertexSet {
    ctx: Py<SymbolicContext>,
    native: GraphColoredVertices,
}

impl AsNative<GraphColoredVertices> for ColoredVertexSet {
    fn as_native(&self) -> &GraphColoredVertices {
        &self.native
    }

    fn as_native_mut(&mut self) -> &mut GraphColoredVertices {
        &mut self.native
    }
}

#[pymethods]
impl ColoredVertexSet {
    /// Normally, a new `ColoredVertexSet` is derived using an `AsynchronousGraph`. However, in some
    /// cases you may want to create it manually from a `SymbolicContext` and a `Bdd`.
    ///
    /// Just keep in mind that this method does not check that the provided `Bdd` is semantically
    /// a valid colored set of vertices.
    #[new]
    pub fn new(ctx: Py<SymbolicContext>, bdd: &Bdd) -> Self {
        Self {
            ctx: ctx.clone(),
            native: GraphColoredVertices::new(bdd.as_native().clone(), ctx.get().as_native()),
        }
    }

    fn __richcmp__(&self, py: Python, other: &Self, op: CompareOp) -> Py<PyAny> {
        match op {
            CompareOp::Eq => ColoredVertexSet::semantic_eq(self, other).into_py(py),
            CompareOp::Ne => ColoredVertexSet::semantic_eq(self, other).not().into_py(py),
            _ => py.NotImplemented(),
        }
    }

    fn __str__(&self) -> String {
        format!(
            "ColoredVertexSet(cardinality={}, symbolic_size={})",
            self.cardinality(),
            self.symbolic_size(),
        )
    }

    fn __repr__(&self) -> String {
        format!(
            "ColoredVertexSet(cardinality={}, colors={}, vertices={}, symbolic_size={})",
            self.cardinality(),
            self.colors().cardinality(),
            self.vertices().cardinality(),
            self.symbolic_size(),
        )
    }

    fn __copy__(self_: Py<Self>) -> Py<Self> {
        self_.clone()
    }

    fn __deepcopy__(self_: Py<Self>, _memo: &PyAny) -> Py<Self> {
        self_.clone()
    }

    fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.as_native().hash(&mut hasher);
        hasher.finish()
    }

    /// Returns the number of vertex-color pairs that are represented in this set.
    fn cardinality(&self) -> BigInt {
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

    /// True if this set is a singleton, i.e. a single vertex-color pair.
    fn is_singleton(&self) -> bool {
        self.as_native().is_singleton()
    }

    /// True if this set is a subspace, i.e. it can be expressed using a single conjunctive clause.
    fn is_subspace(&self) -> bool {
        self.as_native().is_subspace()
    }

    /// The number of `Bdd` nodes that are used to represent this set.
    fn symbolic_size(&self) -> usize {
        self.as_native().symbolic_size()
    }

    /// Compute the existential projection of this relation to the color component. I.e. returns a set of colors
    /// such that for each color, there is at least one vertex-color pair in the original set.
    fn colors(&self) -> ColorSet {
        ColorSet::mk_native(self.ctx.clone(), self.as_native().colors())
    }

    /// Compute the existential projection of this relation to the vertex component. I.e. returns a set of vertices
    /// such that for each vertex, there is at least one vertex-color pair in the original set.
    fn vertices(&self) -> VertexSet {
        VertexSet::mk_native(self.ctx.clone(), self.as_native().vertices())
    }

    /// Retain only those vertex-color pairs for which the color is also contained in the given `colors` set.
    fn intersect_colors(&self, colors: &ColorSet) -> Self {
        self.mk_derived(self.as_native().intersect_colors(colors.as_native()))
    }

    /// Retain only those vertex-color pairs for which the vertex is also contained in the given `vertex` set.
    fn intersect_vertices(&self, vertices: &VertexSet) -> Self {
        self.mk_derived(self.as_native().intersect_vertices(vertices.as_native()))
    }

    /// Remove all vertex-color pairs for which the color is present in the given `colors` set.
    fn minus_colors(&self, colors: &ColorSet) -> Self {
        self.mk_derived(self.as_native().minus_colors(colors.as_native()))
    }

    /// Remove all vertex-color pairs for which the vertex is present in the given `vertex` set.
    fn minus_vertices(&self, vertices: &VertexSet) -> Self {
        self.mk_derived(self.as_native().minus_vertices(vertices.as_native()))
    }

    /// Pick a subset of this relation such that each color that is in the original relation is only present
    /// with a single vertex in the result relation.
    ///
    /// I.e. for each `color` that appears in this set, `result.intersect_colors(color)` is a singleton.
    fn pick_color(&self) -> Self {
        self.mk_derived(self.as_native().pick_color())
    }

    /// Pick a subset of this relation such that each vertex that is in the original relation is only present
    /// with a single color in the result relation.
    ///
    /// I.e. for each `vertex` that appears in this set, `result.intersect_vertices(vertex)` is a singleton.
    fn pick_vertex(&self) -> Self {
        self.mk_derived(self.as_native().pick_vertex())
    }

    /// Deterministically pick a subset of this set that contains exactly a single vertex-color pair.
    ///
    /// If this set is empty, the result is also empty.
    fn pick_singleton(&self) -> Self {
        self.mk_derived(self.as_native().pick_singleton())
    }

    /// Obtain the underlying `Bdd` of this `VertexSet`.
    fn to_bdd(&self, py: Python) -> Bdd {
        let rs_bdd = self.as_native().as_bdd().clone();
        let ctx = self.ctx.borrow(py);
        Bdd::new_raw_2(ctx.bdd_variable_set(), rs_bdd)
    }
}

impl ColoredVertexSet {
    pub fn mk_native(ctx: Py<SymbolicContext>, native: GraphColoredVertices) -> Self {
        Self { ctx, native }
    }

    pub fn mk_derived(&self, native: GraphColoredVertices) -> Self {
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
