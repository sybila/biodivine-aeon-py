use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::ops::{Not, Shr};

use biodivine_lib_bdd::{bdd, Bdd as RsBdd};
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::projected_iteration::{
    OwnedRawSymbolicIterator, RawProjection,
};
use biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices;
use num_bigint::BigInt;
use pyo3::basic::CompareOp;
use pyo3::prelude::PyListMethods;
use pyo3::types::PyList;
use pyo3::{pyclass, pymethods, Bound, IntoPyObjectExt, Py, PyAny, PyResult, Python};

use crate::bindings::lib_bdd::bdd::Bdd;
use crate::bindings::lib_param_bn::symbolic::set_color::ColorSet;
use crate::bindings::lib_param_bn::symbolic::set_colored_vertex::ColoredVertexSet;
use crate::bindings::lib_param_bn::NetworkVariableContext;
use crate::bindings::pbn_control::asynchronous_perturbation_graph::AsynchronousPerturbationGraph;
use crate::bindings::pbn_control::{ColoredPerturbationSet, PerturbationModel};
use crate::{throw_runtime_error, AsNative};

/// A symbolic representation of a set of "perturbations". A perturbation specifies for each
/// variable whether it is fixed or not, and if it is fixed, it prescribes a value. To do so,
/// it uses a combination of state variables and perturbation parameters declared by an
/// `AsynchronousPerturbationGraph`.
///
/// Internally, the representation therefore uses the state variables of the perturbable network
/// variables, plus the perturbation parameters. If a variable is not perturbed, the state variable
/// should remain unconstrained, as this is the most "natural" representation. However, this
/// introduces some issues in terms of cardinality computation and iterators, since we now have
/// to account for the fact that if a variable is unperturbed, it actually generates two
/// perturbation instances (one with state variable `true`, one with `false`). We generally
/// address this by manually fixing the state variable to `false` within these operations.
#[pyclass(module = "biodivine_aeon", frozen)]
#[derive(Clone)]
pub struct PerturbationSet {
    ctx: Py<AsynchronousPerturbationGraph>,
    native: GraphColoredVertices,
}

/// An internal class used for iterating over `PerturbationModel` instances of a `PerturbationSet`.
#[pyclass(module = "biodivine_aeon")]
pub struct _PerturbationModelIterator {
    ctx: Py<AsynchronousPerturbationGraph>,
    native: OwnedRawSymbolicIterator,
    parameter_mapping: HashMap<biodivine_lib_param_bn::VariableId, biodivine_lib_bdd::BddVariable>,
}

#[pymethods]
impl PerturbationSet {
    /// Normally, a new `PerturbationSet` is derived using an `AsynchronousPerturbationGraph`.
    /// However, in some cases you may want to create it manually from an
    /// `AsynchronousPerturbationGraph` and a `Bdd`.
    ///
    /// Keep in mind that this method does not check that the provided `Bdd` is semantically
    /// a valid set of perturbations.
    #[new]
    pub fn new(py: Python, ctx: Py<AsynchronousPerturbationGraph>, bdd: &Bdd) -> Self {
        let parent_ref = ctx.borrow(py);
        let inner_ctx = parent_ref.as_ref().symbolic_context();
        let ctx_ref = inner_ctx.borrow(py);
        Self {
            ctx: ctx.clone(),
            native: GraphColoredVertices::new(bdd.as_native().clone(), ctx_ref.as_native()),
        }
    }

    pub fn __richcmp__(&self, py: Python, other: &Self, op: CompareOp) -> PyResult<Py<PyAny>> {
        match op {
            CompareOp::Eq => PerturbationSet::semantic_eq(self, other).into_py_any(py),
            CompareOp::Ne => PerturbationSet::semantic_eq(self, other)
                .not()
                .into_py_any(py),
            _ => Ok(py.NotImplemented()),
        }
    }

    pub fn __str__(&self) -> String {
        format!(
            "PerturbationSet(cardinality={}, symbolic_size={})",
            self.cardinality(),
            self.symbolic_size(),
        )
    }

    pub fn __repr__(&self) -> String {
        format!(
            "PerturbationSet(cardinality={}, symbolic_size={})",
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

    pub fn __iter__(&self, py: Python) -> PyResult<_PerturbationModelIterator> {
        self.items(py, None)
    }

    pub fn __ctx__(&self) -> Py<AsynchronousPerturbationGraph> {
        self.ctx.clone()
    }

    /// Returns the number of vertices that are represented in this set.
    pub fn cardinality(&self) -> BigInt {
        let pruner = Self::mk_duplicate_pruning_bdd(self.ctx.get());
        let pruned = self.as_native().as_bdd().and(&pruner);
        let all_variables = self
            .ctx
            .get()
            .as_native()
            .as_symbolic_context()
            .bdd_variable_set()
            .num_vars();
        let perturbation_variables =
            u16::try_from(self.ctx.get().as_native().perturbable_variables().len()).unwrap();
        // A perturbation set uses two times the perturbable variables (state var, plus parameter).
        let unused_variables = all_variables - 2 * perturbation_variables;
        pruned.exact_cardinality().shr(unused_variables)
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

    /// True if this set is a singleton, i.e., a single vertex.
    pub fn is_singleton(&self, py: Python) -> PyResult<bool> {
        let mut it = self.__iter__(py)?;
        let fst = it.native.next();
        let snd = it.native.next();
        Ok(fst.is_some() && snd.is_none())
    }

    /// Deterministically, pick a subset of this set that contains exactly a single vertex.
    ///
    /// If this set is empty, the result is also empty.
    pub fn pick_singleton(&self, py: Python) -> PyResult<Self> {
        let mut it = self.__iter__(py)?;
        let Some(model) = it.__next__() else {
            return Ok(AsynchronousPerturbationGraph::mk_empty_perturbations(
                self.ctx.clone(),
            ));
        };
        Ok(model.to_symbolic())
    }

    /// The number of `Bdd` nodes that are used to represent this set.
    pub fn symbolic_size(&self) -> usize {
        self.as_native().symbolic_size()
    }

    /// Get the underlying `Bdd` of this `PerturbationSet`.
    pub fn to_bdd(&self, py: Python) -> Bdd {
        let rs_bdd = self.as_native().as_bdd().clone();
        let ctx = self.ctx.borrow(py).as_ref().symbolic_context();
        let ctx_ref = ctx.borrow(py);
        Bdd::new_raw_2(ctx_ref.bdd_variable_set(), rs_bdd)
    }

    /// Get the internal representation of this `PerturbationSet`, which uses the
    /// `AsynchronousPerturbationGraph` encoding. This is a colored set of vertices, where
    /// the colors only depend on the perturbation parameters, and the vertices are only
    /// constrained in case the variable is perturbed.
    pub fn to_internal(&self, py: Python) -> ColoredVertexSet {
        ColoredVertexSet::mk_native(
            self.ctx.borrow(py).as_ref().symbolic_context(),
            self.native.clone(),
        )
    }

    /// Extend this set of perturbations with all the colors from the given set.
    ///
    /// This is essentially a cartesian product with the given `ColorSet`.
    pub fn extend_with_colors(&self, colors: &ColorSet) -> ColoredPerturbationSet {
        // We have to eliminate perturbation parameters from the colors
        // BDD because they are fixed to false.
        let ctx = self.ctx.get();
        let colors = colors.as_native().as_bdd();
        let perturbation_vars = ctx
            .as_native()
            .get_perturbation_bdd_mapping(ctx.as_native().perturbable_variables())
            .into_values()
            .collect::<Vec<_>>();
        let colors = colors.exists(&perturbation_vars);
        let bdd = self.native.as_bdd().and(&colors);
        let inner_ctx = ctx.as_native().as_original().symbolic_context();
        let native_set = GraphColoredVertices::new(bdd, inner_ctx);
        ColoredPerturbationSet::mk_native(self.ctx.clone(), native_set)
    }

    /// Returns an iterator over all perturbations in this `PerturbationSet` with an optional
    /// projection to a subset of network variables.
    ///
    /// When no `retained` collection is specified, this is equivalent to
    /// `PerturbationSet.__iter__`. However, if a retained set is given, the resulting iterator
    /// only considers unique combinations of the `retained` variables. Consequently, the
    /// resulting `PerturbationModel` instances will fail with an `IndexError` if a value of
    /// a variable outside the `retained` set is requested.
    ///
    /// Similarly, `IndexError` is raised if you request a value of a variable that is not
    /// perturbable in the underlying `AsynchronousPerturbationGraph`.
    #[pyo3(signature = (retained = None))]
    pub fn items(
        &self,
        py: Python,
        retained: Option<&Bound<'_, PyList>>,
    ) -> PyResult<_PerturbationModelIterator> {
        let self_ctx = self.ctx.borrow(py);
        let parent = self_ctx.as_ref();
        let ctx = self.ctx.get();
        let retained = if let Some(retained) = retained {
            retained
                .iter()
                .map(|it| parent.resolve_network_variable(&it))
                .collect::<PyResult<Vec<_>>>()?
        } else {
            ctx.as_native().perturbable_variables().clone()
        };
        let map = ctx.as_native().get_perturbation_bdd_mapping(&retained);
        let mut retained_symbolic = Vec::new();
        for net_var in retained {
            if let Some(p_var) = map.get(&net_var) {
                let state_var = parent
                    .as_native()
                    .symbolic_context()
                    .get_state_variable(net_var);
                retained_symbolic.push(state_var);
                retained_symbolic.push(*p_var);
            } else {
                return throw_runtime_error(format!("Variable {net_var} is not perturbable."));
            }
        }

        let pruner = Self::mk_duplicate_pruning_bdd(self.ctx.get());
        let bdd = self.as_native().as_bdd();

        let projection = RawProjection::new(retained_symbolic, &pruner.and(bdd));
        Ok(_PerturbationModelIterator {
            ctx: self.ctx.clone(),
            native: projection.into_iter(),
            parameter_mapping: map,
        })
    }
}

impl AsNative<GraphColoredVertices> for PerturbationSet {
    fn as_native(&self) -> &GraphColoredVertices {
        &self.native
    }

    fn as_native_mut(&mut self) -> &mut GraphColoredVertices {
        &mut self.native
    }
}

impl PerturbationSet {
    pub fn mk_native(ctx: Py<AsynchronousPerturbationGraph>, native: GraphColoredVertices) -> Self {
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

    /// Construct a `Bdd` which restricts free state variables to false whenever the variable is
    /// unperturbed.
    ///
    /// This is used when counting the actual perturbations or iterating over them.
    pub fn mk_duplicate_pruning_bdd(ctx: &AsynchronousPerturbationGraph) -> biodivine_lib_bdd::Bdd {
        let symbolic = ctx.as_native().as_symbolic_context().bdd_variable_set();
        let map = ctx
            .as_native()
            .get_perturbation_bdd_mapping(ctx.as_native().perturbable_variables());
        let mut bdd = symbolic.mk_true();
        for var in ctx.as_native().perturbable_variables() {
            let p_var = *map.get(var).unwrap();
            let s_var = ctx
                .as_native()
                .as_symbolic_context()
                .get_state_variable(*var);
            bdd = bdd!(symbolic, bdd & ((!p_var) => (!s_var)))
        }
        bdd
    }
}

#[pymethods]
impl _PerturbationModelIterator {
    fn __iter__(self_: Py<Self>) -> Py<Self> {
        self_
    }

    pub fn __next__(&mut self) -> Option<PerturbationModel> {
        self.native.next().map(|mut it| {
            // Remove state variables if the value is unperturbed.
            let native_ctx = self.ctx.get().as_native();
            for (var, p_var) in &self.parameter_mapping {
                if let Some(false) = it.get_value(*p_var) {
                    let s_var = native_ctx.as_symbolic_context().get_state_variable(*var);
                    it.unset_value(s_var);
                }
            }
            PerturbationModel::new_native(self.ctx.clone(), it, self.parameter_mapping.clone())
        })
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<PerturbationModel> {
        self.__next__()
    }
}
