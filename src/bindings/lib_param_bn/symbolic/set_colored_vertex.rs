use crate::AsNative;
use crate::bindings::lib_bdd::bdd::Bdd;
use crate::bindings::lib_param_bn::argument_types::variable_id_type::VariableIdType;
use crate::bindings::lib_param_bn::symbolic::model_color::ColorModel;
use crate::bindings::lib_param_bn::symbolic::model_vertex::VertexModel;
use crate::bindings::lib_param_bn::symbolic::set_color::ColorSet;
use crate::bindings::lib_param_bn::symbolic::set_colored_space::ColoredSpaceSet;
use crate::bindings::lib_param_bn::symbolic::set_vertex::VertexSet;
use crate::bindings::lib_param_bn::symbolic::symbolic_context::SymbolicContext;
use crate::bindings::lib_param_bn::symbolic::symbolic_space_context::SymbolicSpaceContext;
use biodivine_lib_bdd::random_sampling::UniformValuationSampler;
use biodivine_lib_bdd::{Bdd as RsBdd, BddPartialValuation};
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices;
use biodivine_lib_param_bn::symbolic_async_graph::projected_iteration::{
    OwnedRawSymbolicIterator, RawProjection,
};
use num_bigint::BigUint;
use pyo3::IntoPyObjectExt;
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::types::PyList;
use rand::SeedableRng;
use rand::prelude::StdRng;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::ops::Not;

/// A symbolic representation of a relation of "colored vertices", i.e., pairs of vertices
/// (see `VertexSet`) and colors (see `ColorSet`).
///
/// Together, such a pair represents a specific interpretation of network parameters and
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

/// An internal class used for random uniform sampling of `ColorModel` and `VertexModel` pairs from a `ColoredVertexSet`.
/// On the Python side, it just looks like an infinite iterator.
///
/// Similar to [`_ColorVertexModelIterator`], but uses a sampler to get valuations from the projection
/// BDD instead of iterating it fully.
#[pyclass(module = "biodivine_aeon")]
pub struct _ColorVertexModelSampler {
    ctx: Py<SymbolicContext>,
    projection: RawProjection,
    sampler: UniformValuationSampler<StdRng>,
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
    /// Keep in mind that this method does not check that the provided `Bdd` is semantically
    /// a valid colored set of vertices.
    #[new]
    pub fn new(ctx: Py<SymbolicContext>, bdd: &Bdd) -> Self {
        Self {
            ctx: ctx.clone(),
            native: GraphColoredVertices::new(bdd.as_native().clone(), ctx.get().as_native()),
        }
    }

    pub fn __richcmp__(&self, py: Python, other: &Self, op: CompareOp) -> PyResult<Py<PyAny>> {
        match op {
            CompareOp::Eq => ColoredVertexSet::semantic_eq(self, other).into_py_any(py),
            CompareOp::Ne => ColoredVertexSet::semantic_eq(self, other)
                .not()
                .into_py_any(py),
            _ => Ok(py.NotImplemented()),
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

    fn __getnewargs__(&self, py: Python) -> (Py<SymbolicContext>, Bdd) {
        let bdd = self.to_bdd(py);
        (self.ctx.clone(), bdd)
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
    pub fn cardinality(&self) -> BigUint {
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

    /// True if this set is a singleton, i.e., a single vertex-color pair.
    pub fn is_singleton(&self) -> bool {
        self.as_native().is_singleton()
    }

    /// True if this set is a subspace, i.e., it can be expressed using a single conjunctive clause.
    pub fn is_subspace(&self) -> bool {
        self.as_native().is_subspace()
    }

    /// The number of `Bdd` nodes that are used to represent this set.
    pub fn symbolic_size(&self) -> usize {
        self.as_native().symbolic_size()
    }

    /// Compute the existential projection of this relation to the color component. I.e., returns a set of colors
    /// such that for each color, there is at least one vertex-color pair in the original set.
    pub fn colors(&self) -> ColorSet {
        ColorSet::mk_native(self.ctx.clone(), self.as_native().colors())
    }

    /// Compute the existential projection of this relation to the vertex component. I.e., returns a set of vertices
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

    /// Pick a subset of this relation such that each vertex in the original relation is only present
    /// with a single color in the result relation.
    ///
    /// I.e., for each `vertex` that appears in this set `result.intersect_vertices(vertex)` is a singleton.
    pub fn pick_vertex(&self) -> Self {
        self.mk_derived(self.as_native().pick_vertex())
    }

    /// Deterministically pick a subset of this set that exactly contains a single vertex-color pair.
    ///
    /// If this set is empty, the result is also empty.
    pub fn pick_singleton(&self) -> Self {
        self.mk_derived(self.as_native().pick_singleton())
    }

    /// Get the underlying `Bdd` of this `ColoredVertexSet`.
    pub fn to_bdd(&self, py: Python) -> Bdd {
        let rs_bdd = self.as_native().as_bdd().clone();
        let ctx = self.ctx.borrow(py);
        Bdd::new_raw_2(ctx.bdd_variable_set(), rs_bdd)
    }

    /// Returns an iterator over all interpretation-vertex pairs in this `ColoredVertexSet` relation, with an optional
    /// projection to a subset of network variables and uninterpreted functions.
    ///
    /// When no `retained` collections are specified, this is an equivalent to `ColoredVertexSet.__iter__`. However, if
    /// a retained collection is given, the resulting iterator only considers unique combinations of the `retained`
    /// functions and variables. Consequently, the resulting `ColorModel` and `VertexModel` instances will fail with
    /// an `IndexError` if a value outside the `retained` set is requested.
    ///
    /// Note that if you set `retained_variables = []` and `retained_functions = None`, this is an equivalent to
    /// `set.colors().items()`. Similarly, with `retained_variables = None` and `retained_functions = []`, this is
    /// equivalent to `set.vertices().items()`.
    #[pyo3(signature = (retained_variables = None, retained_functions = None))]
    pub fn items(
        &self,
        retained_variables: Option<Vec<VariableIdType>>,
        retained_functions: Option<&Bound<'_, PyList>>,
    ) -> PyResult<_ColorVertexModelIterator> {
        let ctx = self.ctx.get();
        let (retained, implicit, explicit) =
            Self::compute_retained(ctx, retained_variables, retained_functions)?;
        let projection = RawProjection::new(retained, self.as_native().as_bdd());
        Ok(_ColorVertexModelIterator {
            ctx: self.ctx.clone(),
            native: projection.into_iter(),
            retained_implicit: implicit,
            retained_explicit: explicit,
        })
    }

    /// Returns a sampler for random uniform sampling of interpretation-vertex pairs from this `ColoredVertexSet` with an
    /// optional projection to a subset of network variables and uninterpreted functions. **If a projection is specified,
    /// the sampling is uniform with respect to the projected set.**
    ///
    /// See also the `items` method regarding the `retained_variables` and `retained_functions` projection sets.
    ///
    /// You can specify an optional seed to make the sampling random but deterministic.
    #[pyo3(signature = (retained_variables = None, retained_functions = None, seed = None))]
    pub fn sample_items(
        &self,
        retained_variables: Option<Vec<VariableIdType>>,
        retained_functions: Option<&Bound<'_, PyList>>,
        seed: Option<u64>,
    ) -> PyResult<_ColorVertexModelSampler> {
        let ctx = self.ctx.get();
        let (retained, implicit, explicit) =
            Self::compute_retained(ctx, retained_variables, retained_functions)?;
        let projection = RawProjection::new(retained, self.as_native().as_bdd());
        let rng = StdRng::seed_from_u64(seed.unwrap_or_default());
        let sampler = projection.bdd().mk_uniform_valuation_sampler(rng);
        Ok(_ColorVertexModelSampler {
            ctx: self.ctx.clone(),
            projection,
            sampler,
            retained_explicit: explicit,
            retained_implicit: implicit,
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

    /// Helper function to compute retained variables and functions.
    /// This is shared between `items` and `sample_items` to avoid duplication.
    fn compute_retained(
        ctx: &SymbolicContext,
        retained_variables: Option<Vec<VariableIdType>>,
        retained_functions: Option<&Bound<'_, PyList>>,
    ) -> PyResult<(
        Vec<biodivine_lib_bdd::BddVariable>,
        Vec<biodivine_lib_param_bn::VariableId>,
        Vec<biodivine_lib_param_bn::ParameterId>,
    )> {
        // First, extract all functions that should be retained (see also ColorSet.items).
        let (mut retained_functions, implicit, explicit) =
            ColorSet::read_retained_functions(ctx, retained_functions)?;

        // Then add all retained network variables (see also VertexSet.items).
        let mut retained_variables =
            VertexSet::compute_retained_variables(ctx, retained_variables)?;

        let mut retained = Vec::new();
        retained.append(&mut retained_functions);
        retained.append(&mut retained_variables);
        retained.sort();

        Ok((retained, implicit, explicit))
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

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<(ColorModel, VertexModel)> {
        self.__next__()
    }
}

#[pymethods]
impl _ColorVertexModelSampler {
    fn __iter__(self_: Py<Self>) -> Py<Self> {
        self_
    }

    fn __next__(&mut self) -> Option<(ColorModel, VertexModel)> {
        self.projection
            .bdd()
            .random_valuation_sample(&mut self.sampler)
            .map(|it| {
                let mut retained_color = BddPartialValuation::empty();
                let mut retained_state = BddPartialValuation::empty();
                let native_ctx = self.ctx.get().as_native();
                // Only state / color variables should be retained
                for x in self.projection.retained_variables() {
                    if native_ctx.find_state_variable(*x).is_some() {
                        retained_state.set_value(*x, it[*x]);
                    } else {
                        retained_color.set_value(*x, it[*x]);
                    }
                }

                let color = ColorModel::new_native(
                    self.ctx.clone(),
                    retained_color,
                    self.retained_implicit.clone(),
                    self.retained_explicit.clone(),
                );
                let vertex = VertexModel::new_native(self.ctx.clone(), retained_state);
                (color, vertex)
            })
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<(ColorModel, VertexModel)> {
        self.__next__()
    }
}

impl ColoredVertexSet {
    pub fn ctx(&self) -> Py<SymbolicContext> {
        self.ctx.clone()
    }
}
