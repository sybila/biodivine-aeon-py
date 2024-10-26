use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::ops::Not;

use crate::bindings::lib_bdd::bdd::Bdd;
use crate::bindings::lib_param_bn::symbolic::model_color::ColorModel;
use crate::AsNative;
use biodivine_lib_param_bn::symbolic_async_graph::projected_iteration::{
    OwnedRawSymbolicIterator, RawProjection,
};
use biodivine_lib_param_bn::trap_spaces::NetworkColoredSpaces;
use either::Either;
use num_bigint::BigInt;
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::types::PyList;

use crate::bindings::lib_param_bn::symbolic::set_color::ColorSet;

use crate::bindings::lib_param_bn::symbolic::model_space::SpaceModel;
use crate::bindings::lib_param_bn::symbolic::set_colored_vertex::ColoredVertexSet;
use crate::bindings::lib_param_bn::symbolic::set_spaces::SpaceSet;
use crate::bindings::lib_param_bn::symbolic::symbolic_context::SymbolicContext;
use crate::bindings::lib_param_bn::symbolic::symbolic_space_context::SymbolicSpaceContext;
use crate::bindings::lib_param_bn::NetworkVariableContext;
use biodivine_lib_bdd::Bdd as RsBdd;
use biodivine_lib_param_bn::biodivine_std::traits::Set;

/// A symbolic representation of a colored relation of "spaces", i.e. hypercubes in the state space
/// of a particular partially specified `BooleanNetwork` together with the instantiations of
/// individual interpretations of network parameters.
#[pyclass(module = "biodivine_aeon", frozen)]
#[derive(Clone)]
pub struct ColoredSpaceSet {
    ctx: Py<SymbolicSpaceContext>,
    native: NetworkColoredSpaces,
}

/// An internal class that allows iterating over pairs of `ColorModel` and `SpaceModel` instances.
#[pyclass(module = "biodivine_aeon")]
pub struct _ColorSpaceModelIterator {
    ctx: Py<SymbolicSpaceContext>,
    native: OwnedRawSymbolicIterator,
    retained_explicit: Vec<biodivine_lib_param_bn::ParameterId>,
    retained_implicit: Vec<biodivine_lib_param_bn::VariableId>,
}

impl AsNative<NetworkColoredSpaces> for ColoredSpaceSet {
    fn as_native(&self) -> &NetworkColoredSpaces {
        &self.native
    }

    fn as_native_mut(&mut self) -> &mut NetworkColoredSpaces {
        &mut self.native
    }
}

#[pymethods]
impl ColoredSpaceSet {
    /// Normally, a new `ColoredSpaceSet` is derived using an `SymbolicSpaceContext`. However, in some
    /// cases you may want to create it manually from a `SymbolicSpaceContext` and a `Bdd`.
    ///
    /// Just keep in mind that this method does not check that the provided `Bdd` is semantically
    /// a valid colored set of vertices.
    #[new]
    pub fn new(ctx: Py<SymbolicSpaceContext>, bdd: &Bdd) -> Self {
        Self {
            ctx: ctx.clone(),
            native: NetworkColoredSpaces::new(bdd.as_native().clone(), ctx.get().as_native()),
        }
    }

    fn __richcmp__(&self, py: Python, other: &Self, op: CompareOp) -> Py<PyAny> {
        match op {
            CompareOp::Eq => Self::semantic_eq(self, other).into_py(py),
            CompareOp::Ne => Self::semantic_eq(self, other).not().into_py(py),
            _ => py.NotImplemented(),
        }
    }

    fn __str__(&self) -> String {
        format!(
            "ColoredSpaceSet(cardinality={}, symbolic_size={})",
            self.cardinality(),
            self.symbolic_size(),
        )
    }

    fn __repr__(&self, py: Python) -> PyResult<String> {
        Ok(format!(
            "ColoredSpaceSet(cardinality={}, colors={}, spaces={}, symbolic_size={})",
            self.cardinality(),
            self.colors(py)?.cardinality(),
            self.spaces().cardinality(),
            self.symbolic_size(),
        ))
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

    fn __iter__(&self, py: Python) -> PyResult<_ColorSpaceModelIterator> {
        self.items(None, None, py)
    }

    fn __ctx__(&self) -> Py<SymbolicSpaceContext> {
        self.ctx.clone()
    }

    /// Returns the number of space-color pairs that are represented in this set.
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

    /// The number of `Bdd` nodes that are used to represent this set.
    fn symbolic_size(&self) -> usize {
        self.as_native().symbolic_size()
    }

    /// Compute the existential projection of this relation to the color component. I.e. returns a set of colors
    /// such that for each color, there is at least one space-color pair in the original set.
    fn colors(&self, py: Python) -> PyResult<ColorSet> {
        let parent_ctx = self.ctx.extract::<Py<SymbolicContext>>(py)?;
        Ok(ColorSet::mk_native(parent_ctx, self.as_native().colors()))
    }

    /// Compute the existential projection of this relation to the subspace component. I.e. returns a set of spaces
    /// such that for each space, there is at least one space-color pair in the original set.
    fn spaces(&self) -> SpaceSet {
        SpaceSet::wrap_native(self.ctx.clone(), self.as_native().spaces())
    }

    /// Retain only those space-color pairs for which the color is also contained in the given `colors` set.
    fn intersect_colors(&self, colors: &ColorSet) -> Self {
        self.mk_derived(self.as_native().intersect_colors(colors.as_native()))
    }

    /// Retain only those space-color pairs for which the space is also contained in the given `spaces` set.
    fn intersect_spaces(&self, spaces: &SpaceSet) -> Self {
        self.mk_derived(self.as_native().intersect_spaces(spaces.as_native()))
    }

    /// Remove all space-color pairs for which the color is present in the given `colors` set.
    fn minus_colors(&self, colors: &ColorSet) -> Self {
        self.mk_derived(self.as_native().minus_colors(colors.as_native()))
    }

    /// Remove all space-color pairs for which the space is present in the given `spaces` set.
    fn minus_spaces(&self, spaces: &SpaceSet) -> Self {
        self.mk_derived(self.as_native().minus_spaces(spaces.as_native()))
    }

    /// Pick a subset of this relation such that each color that is in the original relation is only present
    /// with a single vertex in the result relation.
    ///
    /// I.e. for each `color` that appears in this set, `result.intersect_colors(color)` is a singleton.
    fn pick_color(&self) -> Self {
        self.mk_derived(self.as_native().pick_color())
    }

    /// Pick a subset of this relation such that each space that is in the original relation is only present
    /// with a single color in the result relation.
    ///
    /// I.e. for each `space` that appears in this set, `result.intersect_spaces(space)` is a singleton.
    fn pick_space(&self) -> Self {
        self.mk_derived(self.as_native().pick_space())
    }

    /// Deterministically pick a subset of this set that contains exactly a single space-color pair.
    ///
    /// If this set is empty, the result is also empty.
    fn pick_singleton(&self) -> Self {
        self.mk_derived(self.as_native().pick_singleton())
    }

    /// Obtain the underlying `Bdd` of this `ColoredSpaceSet`.
    fn to_bdd(&self, py: Python) -> Bdd {
        let rs_bdd = self.as_native().as_bdd().clone();
        Bdd::new_raw_2(self.ctx.borrow(py).as_ref().bdd_variable_set(), rs_bdd)
    }

    /// Returns an iterator over all interpretation-space pairs in this `ColoredSpaceSet` relation, with an optional
    /// projection to a subset of network variables and uninterpreted functions.
    ///
    /// When no `retained` collections are specified, this is equivalent to `ColoredSpaceSet.__iter__`. However, if
    /// a retained collection is given, the resulting iterator only considers unique combinations of the `retained`
    /// functions and variables. Consequently, the resulting `ColorModel` and `SpaceModel` instances will fail with
    /// an `IndexError` if a value outside the `retained` set is requested.
    ///
    /// Note that if you set `retained_variables = []` and `retained_functions = None`, this is equivalent to
    /// `set.colors().items()`. Similarly, with `retained_variables = None` and `retained_functions = []`, this is
    /// equivalent to `set.spaces().items()`.
    #[pyo3(signature = (retained_variables = None, retained_functions = None))]
    fn items(
        &self,
        retained_variables: Option<&Bound<'_, PyList>>,
        retained_functions: Option<&Bound<'_, PyList>>,
        py: Python,
    ) -> PyResult<_ColorSpaceModelIterator> {
        let ctx = self.ctx.borrow(py);
        let ctx_parent = ctx.as_ref();
        // First, extract all functions that should be retained (see also ColorSet.items).
        let mut retained_explicit = Vec::new();
        let mut retained_implicit = Vec::new();
        let mut retained_functions = if let Some(retained) = retained_functions {
            let mut result = Vec::new();
            for x in retained {
                let function = ctx_parent.resolve_function(&x)?;
                let table = match function {
                    Either::Left(x) => {
                        if retained_implicit.contains(&x) {
                            continue;
                        }
                        retained_implicit.push(x);
                        ctx_parent
                            .as_native()
                            .get_implicit_function_table(x)
                            .unwrap()
                    }
                    Either::Right(x) => {
                        if retained_explicit.contains(&x) {
                            continue;
                        }
                        retained_explicit.push(x);
                        ctx_parent.as_native().get_explicit_function_table(x)
                    }
                };
                result.append(&mut table.symbolic_variables().clone());
            }
            result
        } else {
            retained_explicit.append(
                &mut ctx_parent
                    .as_native()
                    .network_parameters()
                    .collect::<Vec<_>>(),
            );
            retained_implicit.append(&mut ctx_parent.as_native().network_implicit_parameters());
            ctx_parent.as_native().parameter_variables().clone()
        };
        retained_explicit.sort();
        retained_implicit.sort();

        // Then add all retained network variables (see also SpaceSet.items).
        let mut retained_variables = if let Some(retained) = retained_variables {
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

        let mut retained = Vec::new();
        retained.append(&mut retained_functions);
        retained.append(&mut retained_variables);
        retained.sort();

        let projection = RawProjection::new(retained, self.as_native().as_bdd());
        Ok(_ColorSpaceModelIterator {
            ctx: self.ctx.clone(),
            native: projection.into_iter(),
            retained_implicit,
            retained_explicit,
        })
    }

    /// Produce a set of vertices that are contained within the subspaces represented in this set.
    pub fn to_colored_vertices(
        &self,
        ctx: Py<SymbolicSpaceContext>,
        py: Python,
    ) -> PyResult<ColoredVertexSet> {
        let native = self.as_native().to_colored_vertices(ctx.get().as_native());
        let parent = ctx.extract::<Py<SymbolicContext>>(py)?;
        Ok(ColoredVertexSet::mk_native(parent, native))
    }

    /// Produce a set of spaces that is a superset of this set, and in addition contains
    /// all spaces that are a subspace of *some* item in this set.
    ///
    /// Colors are retained on a per-space basis.
    pub fn with_all_sub_spaces(&self) -> Self {
        let ctx = self.ctx.get().as_native();
        Self::wrap_native(self.ctx.clone(), self.as_native().with_all_sub_spaces(ctx))
    }

    /// Produce a set of spaces that is a superset of this set, and in addition contains
    /// all spaces that are a super-spaces of *some* item in this set.
    ///
    /// Colors are retained on a per-space basis.
    pub fn with_all_super_spaces(&self) -> Self {
        let ctx = self.ctx.get().as_native();
        Self::wrap_native(
            self.ctx.clone(),
            self.as_native().with_all_super_spaces(ctx),
        )
    }
}

impl ColoredSpaceSet {
    pub fn wrap_native(
        ctx: Py<SymbolicSpaceContext>,
        native: NetworkColoredSpaces,
    ) -> ColoredSpaceSet {
        ColoredSpaceSet { ctx, native }
    }

    pub fn mk_derived(&self, native: NetworkColoredSpaces) -> Self {
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
impl _ColorSpaceModelIterator {
    fn __iter__(self_: Py<Self>) -> Py<Self> {
        self_
    }

    fn __next__(&mut self, py: Python) -> PyResult<Option<(ColorModel, SpaceModel)>> {
        if let Some(it) = self.native.next() {
            let parent = self.ctx.extract::<Py<SymbolicContext>>(py)?;
            // Here, we have to create two copies of the partial valuation that don't have
            // any "invalid" variables, otherwise those could propagate further by instantiating
            // a symbolic set from the model object.
            let mut color_val = it.clone();
            let mut space_val = it.clone();
            let native_ctx = self.ctx.get().as_native();
            for s_var in native_ctx.inner_context().all_extra_state_variables() {
                color_val.unset_value(*s_var);
            }
            for p_var in native_ctx.inner_context().parameter_variables() {
                space_val.unset_value(*p_var);
            }
            let color = ColorModel::new_native(
                parent,
                color_val,
                self.retained_implicit.clone(),
                self.retained_explicit.clone(),
            );
            let vertex = SpaceModel::new_native(self.ctx.clone(), space_val);
            Ok(Some((color, vertex)))
        } else {
            Ok(None)
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self, py: Python) -> PyResult<Option<(ColorModel, SpaceModel)>> {
        self.__next__(py)
    }
}
