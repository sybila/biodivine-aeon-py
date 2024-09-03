use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::ops::Not;

use biodivine_lib_bdd::Bdd as RsBdd;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::projected_iteration::{
    OwnedRawSymbolicIterator, RawProjection,
};
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, GraphColors};
use biodivine_lib_param_bn::trap_spaces::NetworkColoredSpaces;
use either::Either;
use num_bigint::BigInt;
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::types::PyList;

use crate::bindings::lib_bdd::bdd::Bdd;
use crate::bindings::lib_param_bn::symbolic::model_color::ColorModel;
use crate::bindings::lib_param_bn::symbolic::set_colored_space::ColoredSpaceSet;
use crate::bindings::lib_param_bn::symbolic::set_colored_vertex::ColoredVertexSet;
use crate::bindings::lib_param_bn::symbolic::set_spaces::SpaceSet;
use crate::bindings::lib_param_bn::symbolic::set_vertex::VertexSet;
use crate::bindings::lib_param_bn::symbolic::symbolic_context::SymbolicContext;
use crate::bindings::pbn_control::{ColoredPerturbationSet, PerturbationSet};
use crate::AsNative;

/// A symbolic representation of a set of "colours", i.e. interpretations of explicit and
/// implicit parameters within a particular `BooleanNetwork`.
#[pyclass(module = "biodivine_aeon", frozen)]
#[derive(Clone)]
pub struct ColorSet {
    ctx: Py<SymbolicContext>,
    native: GraphColors,
}

/// An internal class used for iterating over `ColorModel` instances of a `ColorSet`.
#[pyclass(module = "biodivine_aeon")]
pub struct _ColorModelIterator {
    ctx: Py<SymbolicContext>,
    native: OwnedRawSymbolicIterator,
    retained_explicit: Vec<biodivine_lib_param_bn::ParameterId>,
    retained_implicit: Vec<biodivine_lib_param_bn::VariableId>,
}

impl AsNative<GraphColors> for ColorSet {
    fn as_native(&self) -> &GraphColors {
        &self.native
    }

    fn as_native_mut(&mut self) -> &mut GraphColors {
        &mut self.native
    }
}

impl ColorSet {
    pub fn into_native(self) -> GraphColors {
        self.native
    }
}

#[pymethods]
impl ColorSet {
    /// Normally, a new `ColorSet` is derived using an `AsynchronousGraph`. However, in some
    /// cases you may want to create it manually from a `SymbolicContext` and a `Bdd`.
    ///
    /// Just keep in mind that this method does not check that the provided `Bdd` is semantically
    /// a valid set of colors.
    #[new]
    pub fn new(py: Python, ctx: Py<SymbolicContext>, bdd: &Bdd) -> ColorSet {
        ColorSet {
            ctx: ctx.clone(),
            native: GraphColors::new(bdd.as_native().clone(), ctx.borrow(py).as_native()),
        }
    }

    pub fn __richcmp__(&self, py: Python, other: &Self, op: CompareOp) -> Py<PyAny> {
        match op {
            CompareOp::Eq => ColorSet::semantic_eq(self, other).into_py(py),
            CompareOp::Ne => ColorSet::semantic_eq(self, other).not().into_py(py),
            _ => py.NotImplemented(),
        }
    }

    pub fn __str__(&self) -> String {
        format!(
            "ColorSet(cardinality={}, symbolic_size={})",
            self.cardinality(),
            self.symbolic_size(),
        )
    }

    pub fn __repr__(&self) -> String {
        format!(
            "ColorSet(cardinality={}, symbolic_size={})",
            self.cardinality(),
            self.symbolic_size(),
        )
    }

    pub fn __copy__(self_: Py<ColorSet>) -> Py<ColorSet> {
        self_.clone()
    }

    pub fn __deepcopy__(self_: Py<ColorSet>, _memo: &Bound<'_, PyAny>) -> Py<ColorSet> {
        self_.clone()
    }

    pub fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.as_native().hash(&mut hasher);
        hasher.finish()
    }

    pub fn __iter__(&self) -> PyResult<_ColorModelIterator> {
        self.items(None)
    }

    pub fn __ctx__(&self) -> Py<SymbolicContext> {
        self.ctx.clone()
    }

    /// Returns the number of "colors" (function interpretations) that are represented in this set.
    pub fn cardinality(&self) -> BigInt {
        self.as_native().exact_cardinality()
    }

    /// Set intersection.
    pub fn intersect(&self, other: &ColorSet) -> ColorSet {
        self.mk_derived(self.as_native().intersect(other.as_native()))
    }

    /// Set difference.
    pub fn minus(&self, other: &ColorSet) -> ColorSet {
        self.mk_derived(self.as_native().minus(other.as_native()))
    }

    /// Set union.
    pub fn union(&self, other: &ColorSet) -> ColorSet {
        self.mk_derived(self.as_native().union(other.as_native()))
    }

    /// True if this set is empty.
    pub fn is_empty(&self) -> bool {
        self.as_native().is_empty()
    }

    /// True if this set is a subset of the other set.
    ///
    /// Should be faster than just calling `set.minus(superset).is_empty()`
    pub fn is_subset(&self, other: &ColorSet) -> bool {
        self.as_native().is_subset(other.as_native())
    }

    /// True if this set is a singleton, i.e. a single function interpretation.
    pub fn is_singleton(&self) -> bool {
        self.as_native().is_singleton()
    }

    /// True if this set is a subspace, i.e. it can be expressed using a single conjunctive clause.
    pub fn is_subspace(&self) -> bool {
        self.as_native().is_subspace()
    }

    /// Deterministically pick a subset of this set that contains exactly a single function
    /// interpretation.
    ///
    /// If this set is empty, the result is also empty.
    pub fn pick_singleton(&self) -> ColorSet {
        self.mk_derived(self.as_native().pick_singleton())
    }

    /// The number of `Bdd` nodes that are used to represent this set.
    pub fn symbolic_size(&self) -> usize {
        self.as_native().symbolic_size()
    }

    /// Obtain the underlying `Bdd` of this `ColorSet`.
    pub fn to_bdd(&self, py: Python) -> Bdd {
        let rs_bdd = self.as_native().as_bdd().clone();
        let ctx = self.ctx.borrow(py);
        Bdd::new_raw_2(ctx.bdd_variable_set(), rs_bdd)
    }

    /// Extend this set of colors with all the vertices from the given set.
    ///
    /// This is essentially a cartesian product with the given `VertexSet`.
    pub fn extend_with_vertices(&self, vertices: &VertexSet) -> ColoredVertexSet {
        let vertices = vertices.as_native().as_bdd();
        let bdd = self.native.as_bdd().and(vertices);
        let ctx = self.ctx.get();
        let native_set = GraphColoredVertices::new(bdd, ctx.as_native());
        ColoredVertexSet::mk_native(self.ctx.clone(), native_set)
    }

    /// Extend this set of colors with all the spaces from the given set.
    ///
    /// This is essentially a cartesian product with the given `SpaceSet`.
    pub fn extend_with_spaces(&self, spaces: &SpaceSet) -> ColoredSpaceSet {
        let space_bdd = spaces.as_native().as_bdd();
        let bdd = self.native.as_bdd().and(space_bdd);
        let ctx = spaces.__ctx__();
        let native_set = NetworkColoredSpaces::new(bdd, ctx.get().as_native());
        ColoredSpaceSet::wrap_native(ctx, native_set)
    }

    pub fn extend_with_perturbations(
        &self,
        perturbations: &PerturbationSet,
    ) -> ColoredPerturbationSet {
        let graph_ctx = perturbations.__ctx__();
        let perturbation_vars = graph_ctx
            .get()
            .as_native()
            .get_perturbation_bdd_mapping(graph_ctx.get().as_native().perturbable_variables())
            .into_values()
            .collect::<Vec<_>>();
        let colors = self.native.as_bdd().exists(&perturbation_vars);
        let bdd = perturbations.as_native().as_bdd().and(&colors);
        let ctx = self.ctx.get();
        let native_set = GraphColoredVertices::new(bdd, ctx.as_native());
        ColoredPerturbationSet::mk_native(graph_ctx, native_set)
    }

    /// Returns an iterator over all interpretations in this `ColorSet` with an optional projection to a subset
    /// of uninterpreted functions.
    ///
    /// When no `retained` collection is specified, this is equivalent to `ColorSet.__iter__`. However, if a retained
    /// set is given, the resulting iterator only considers unique combinations of the `retained` functions.
    /// Consequently, the resulting `ColorModel` instances will fail with an `IndexError` if a value of a function
    /// outside the `retained` set is requested.
    #[pyo3(signature = (retained = None))]
    pub fn items(&self, retained: Option<&Bound<'_, PyList>>) -> PyResult<_ColorModelIterator> {
        let ctx = self.ctx.get();
        let mut retained_explicit = Vec::new();
        let mut retained_implicit = Vec::new();
        let retained = if let Some(retained) = retained {
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

        let projection = RawProjection::new(retained, self.as_native().as_bdd());
        Ok(_ColorModelIterator {
            ctx: self.ctx.clone(),
            native: projection.into_iter(),
            retained_implicit,
            retained_explicit,
        })
    }
}

impl ColorSet {
    pub fn mk_native(ctx: Py<SymbolicContext>, native: GraphColors) -> Self {
        Self { ctx, native }
    }

    pub fn mk_derived(&self, native: GraphColors) -> ColorSet {
        ColorSet {
            ctx: self.ctx.clone(),
            native,
        }
    }

    pub fn semantic_eq(a: &ColorSet, b: &ColorSet) -> bool {
        let a = a.as_native().as_bdd();
        let b = b.as_native().as_bdd();
        if a.num_vars() != b.num_vars() {
            return false;
        }

        RsBdd::binary_op_with_limit(1, a, b, biodivine_lib_bdd::op_function::xor).is_some()
    }
}

#[pymethods]
impl _ColorModelIterator {
    fn __iter__(self_: Py<Self>) -> Py<Self> {
        self_
    }

    fn __next__(&mut self) -> Option<ColorModel> {
        self.native.next().map(|it| {
            ColorModel::new_native(
                self.ctx.clone(),
                it,
                self.retained_implicit.clone(),
                self.retained_explicit.clone(),
            )
        })
    }

    pub fn next(&mut self) -> Option<ColorModel> {
        self.__next__()
    }
}
