use crate::bindings::lib_bdd::bdd::Bdd;
use crate::bindings::lib_param_bn::symbolic::model_color::ColorModel;
use crate::bindings::lib_param_bn::symbolic::model_vertex::VertexModel;
use crate::bindings::lib_param_bn::symbolic::set_color::ColorSet;
use crate::bindings::lib_param_bn::symbolic::set_colored_space::ColoredSpaceSet;
use crate::bindings::lib_param_bn::symbolic::set_vertex::VertexSet;
use crate::bindings::lib_param_bn::symbolic::symbolic_context::SymbolicContext;
use crate::bindings::lib_param_bn::symbolic::symbolic_space_context::SymbolicSpaceContext;
use crate::bindings::lib_param_bn::NetworkVariableContext;
use crate::AsNative;
use biodivine_lib_bdd::Bdd as RsBdd;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::projected_iteration::{
    OwnedRawSymbolicIterator, RawProjection,
};
use biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices;
use either::Either;
use num_bigint::BigInt;
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::types::PyList;
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

/// An internal class that allows iterating over pairs of `ColorModel` and `VertexModel` instances.
#[pyclass(module = "biodivine_aeon")]
pub struct _ColorVertexModelIterator {
    ctx: Py<SymbolicContext>,
    native: OwnedRawSymbolicIterator,
    retained_explicit: Vec<biodivine_lib_param_bn::ParameterId>,
    retained_implicit: Vec<biodivine_lib_param_bn::VariableId>,
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

    pub fn __richcmp__(&self, py: Python, other: &Self, op: CompareOp) -> Py<PyAny> {
        match op {
            CompareOp::Eq => ColoredVertexSet::semantic_eq(self, other).into_py(py),
            CompareOp::Ne => ColoredVertexSet::semantic_eq(self, other).not().into_py(py),
            _ => py.NotImplemented(),
        }
    }

    pub fn __str__(&self) -> String {
        format!(
            "ColoredVertexSet(cardinality={}, symbolic_size={})",
            self.cardinality(),
            self.symbolic_size(),
        )
    }

    pub fn __repr__(&self) -> String {
        format!(
            "ColoredVertexSet(cardinality={}, colors={}, vertices={}, symbolic_size={})",
            self.cardinality(),
            self.colors().cardinality(),
            self.vertices().cardinality(),
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

    pub fn __iter__(&self) -> PyResult<_ColorVertexModelIterator> {
        self.items(None, None)
    }

    pub fn __ctx__(&self) -> Py<SymbolicContext> {
        self.ctx.clone()
    }

    /// Returns the number of vertex-color pairs that are represented in this set.
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

    /// True if this set is a singleton, i.e. a single vertex-color pair.
    pub fn is_singleton(&self) -> bool {
        self.as_native().is_singleton()
    }

    /// True if this set is a subspace, i.e. it can be expressed using a single conjunctive clause.
    pub fn is_subspace(&self) -> bool {
        self.as_native().is_subspace()
    }

    /// The number of `Bdd` nodes that are used to represent this set.
    pub fn symbolic_size(&self) -> usize {
        self.as_native().symbolic_size()
    }

    /// Compute the existential projection of this relation to the color component. I.e. returns a set of colors
    /// such that for each color, there is at least one vertex-color pair in the original set.
    pub fn colors(&self) -> ColorSet {
        ColorSet::mk_native(self.ctx.clone(), self.as_native().colors())
    }

    /// Compute the existential projection of this relation to the vertex component. I.e. returns a set of vertices
    /// such that for each vertex, there is at least one vertex-color pair in the original set.
    pub fn vertices(&self) -> VertexSet {
        VertexSet::mk_native(self.ctx.clone(), self.as_native().vertices())
    }

    /// Retain only those vertex-color pairs for which the color is also contained in the given `colors` set.
    pub fn intersect_colors(&self, colors: &ColorSet) -> Self {
        self.mk_derived(self.as_native().intersect_colors(colors.as_native()))
    }

    /// Retain only those vertex-color pairs for which the vertex is also contained in the given `vertex` set.
    pub fn intersect_vertices(&self, vertices: &VertexSet) -> Self {
        self.mk_derived(self.as_native().intersect_vertices(vertices.as_native()))
    }

    /// Remove all vertex-color pairs for which the color is present in the given `colors` set.
    pub fn minus_colors(&self, colors: &ColorSet) -> Self {
        self.mk_derived(self.as_native().minus_colors(colors.as_native()))
    }

    /// Remove all vertex-color pairs for which the vertex is present in the given `vertex` set.
    pub fn minus_vertices(&self, vertices: &VertexSet) -> Self {
        self.mk_derived(self.as_native().minus_vertices(vertices.as_native()))
    }

    /// Pick a subset of this relation such that each color that is in the original relation is only present
    /// with a single vertex in the result relation.
    ///
    /// I.e. for each `color` that appears in this set, `result.intersect_colors(color)` is a singleton.
    pub fn pick_color(&self) -> Self {
        self.mk_derived(self.as_native().pick_color())
    }

    /// Pick a subset of this relation such that each vertex that is in the original relation is only present
    /// with a single color in the result relation.
    ///
    /// I.e. for each `vertex` that appears in this set, `result.intersect_vertices(vertex)` is a singleton.
    pub fn pick_vertex(&self) -> Self {
        self.mk_derived(self.as_native().pick_vertex())
    }

    /// Deterministically pick a subset of this set that contains exactly a single vertex-color pair.
    ///
    /// If this set is empty, the result is also empty.
    pub fn pick_singleton(&self) -> Self {
        self.mk_derived(self.as_native().pick_singleton())
    }

    /// Obtain the underlying `Bdd` of this `ColoredVertexSet`.
    pub fn to_bdd(&self, py: Python) -> Bdd {
        let rs_bdd = self.as_native().as_bdd().clone();
        let ctx = self.ctx.borrow(py);
        Bdd::new_raw_2(ctx.bdd_variable_set(), rs_bdd)
    }

    /// Returns an iterator over all interpretation-vertex pairs in this `ColoredVertexSet` relation, with an optional
    /// projection to a subset of network variables and uninterpreted functions.
    ///
    /// When no `retained` collections are specified, this is equivalent to `ColoredVertexSet.__iter__`. However, if
    /// a retained collection is given, the resulting iterator only considers unique combinations of the `retained`
    /// functions and variables. Consequently, the resulting `ColorModel` and `VertexModel` instances will fail with
    /// an `IndexError` if a value outside the `retained` set is requested.
    ///
    /// Note that if you set `retained_variables = []` and `retained_functions = None`, this is equivalent to
    /// `set.colors().items()`. Similarly, with `retained_variables = None` and `retained_functions = []`, this is
    /// equivalent to `set.vertices().items()`.
    #[pyo3(signature = (retained_variables = None, retained_functions = None))]
    pub fn items(
        &self,
        retained_variables: Option<&Bound<'_, PyList>>,
        retained_functions: Option<&Bound<'_, PyList>>,
    ) -> PyResult<_ColorVertexModelIterator> {
        let ctx = self.ctx.get();
        // First, extract all functions that should be retained (see also ColorSet.items).
        let mut retained_explicit = Vec::new();
        let mut retained_implicit = Vec::new();
        let mut retained_functions = if let Some(retained) = retained_functions {
            let mut result = Vec::new();
            for x in retained {
                let function = ctx.resolve_function(&x)?;
                let table = match function {
                    Either::Left(x) => {
                        if retained_implicit.contains(&x) {
                            continue;
                        }
                        retained_implicit.push(x);
                        ctx.as_native().get_implicit_function_table(x).unwrap()
                    }
                    Either::Right(x) => {
                        if retained_explicit.contains(&x) {
                            continue;
                        }
                        retained_explicit.push(x);
                        ctx.as_native().get_explicit_function_table(x)
                    }
                };
                result.append(&mut table.symbolic_variables().clone());
            }
            result
        } else {
            retained_explicit.append(&mut ctx.as_native().network_parameters().collect::<Vec<_>>());
            retained_implicit.append(&mut ctx.as_native().network_implicit_parameters());
            self.ctx.get().as_native().parameter_variables().clone()
        };
        retained_explicit.sort();
        retained_implicit.sort();

        // Then add all retained network variables (see also VertexSet.items).
        let mut retained_variables = if let Some(retained) = retained_variables {
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

        let mut retained = Vec::new();
        retained.append(&mut retained_functions);
        retained.append(&mut retained_variables);
        retained.sort();

        let projection = RawProjection::new(retained, self.as_native().as_bdd());
        Ok(_ColorVertexModelIterator {
            ctx: self.ctx.clone(),
            native: projection.into_iter(),
            retained_implicit,
            retained_explicit,
        })
    }

    /// Represent this colored set of vertices as a colored set of singleton subspaces instead.
    pub fn to_singleton_spaces(&self, ctx: Py<SymbolicSpaceContext>) -> ColoredSpaceSet {
        let native = self.as_native().to_singleton_spaces(ctx.get().as_native());
        ColoredSpaceSet::wrap_native(ctx, native)
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

#[pymethods]
impl _ColorVertexModelIterator {
    fn __iter__(self_: Py<Self>) -> Py<Self> {
        self_
    }

    fn __next__(&mut self) -> Option<(ColorModel, VertexModel)> {
        self.native.next().map(|it| {
            // Here, we have to create two copies of the partial valuation that don't have
            // any "invalid" variables, otherwise those could propagate further by instantiating
            // a symbolic set from the model object.
            let mut color_val = it.clone();
            let mut state_val = it.clone();
            let native_ctx = self.ctx.get().as_native();
            for s_var in native_ctx.state_variables() {
                color_val.unset_value(*s_var);
            }
            for p_var in native_ctx.parameter_variables() {
                state_val.unset_value(*p_var);
            }
            let color = ColorModel::new_native(
                self.ctx.clone(),
                color_val,
                self.retained_implicit.clone(),
                self.retained_explicit.clone(),
            );
            let vertex = VertexModel::new_native(self.ctx.clone(), state_val);
            (color, vertex)
        })
    }

    pub fn next(&mut self) -> Option<(ColorModel, VertexModel)> {
        self.__next__()
    }
}
