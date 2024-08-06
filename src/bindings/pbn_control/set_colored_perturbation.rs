use std::collections::{HashMap, HashSet};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::ops::{Not, Shr};

use biodivine_lib_bdd::{Bdd as RsBdd, BddPartialValuation};
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::projected_iteration::{
    OwnedRawSymbolicIterator, RawProjection,
};
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, GraphColors};
use either::Either;
use num_bigint::BigInt;
use pyo3::basic::CompareOp;
use pyo3::prelude::PyListMethods;
use pyo3::types::{PyAnyMethods, PyDict, PyList};
use pyo3::{pyclass, pymethods, Bound, IntoPy, Py, PyAny, PyResult, Python};

use crate::bindings::lib_bdd::bdd::Bdd;
use crate::bindings::lib_param_bn::symbolic::model_color::ColorModel;
use crate::bindings::lib_param_bn::symbolic::set_color::ColorSet;
use crate::bindings::lib_param_bn::symbolic::set_colored_vertex::ColoredVertexSet;
use crate::bindings::lib_param_bn::symbolic::symbolic_context::SymbolicContext;
use crate::bindings::lib_param_bn::NetworkVariableContext;
use crate::bindings::pbn_control::asynchronous_perturbation_graph::AsynchronousPerturbationGraph;
use crate::bindings::pbn_control::{PerturbationModel, PerturbationSet};
use crate::{throw_runtime_error, AsNative};

/// A symbolic representation of a colored set of "perturbations". A perturbation specifies for
/// each variable whether it is fixed or not, and if it is fixed, it prescribes a value. To do so,
/// it uses a combination of state variables and perturbation parameters declared by an
/// `AsynchronousPerturbationGraph`. The colors then prescribes the interpretations of the
/// remaining network parameters.
#[pyclass(module = "biodivine_aeon", frozen)]
#[derive(Clone)]
pub struct ColoredPerturbationSet {
    ctx: Py<AsynchronousPerturbationGraph>,
    native: GraphColoredVertices,
}

/// An internal class that allows iterating over pairs of `ColorModel` and `PerturbationModel` instances.
#[pyclass(module = "biodivine_aeon")]
pub struct _ColorPerturbationModelIterator {
    graph: Py<AsynchronousPerturbationGraph>,
    ctx: Py<SymbolicContext>,
    native: OwnedRawSymbolicIterator,
    retained_explicit: Vec<biodivine_lib_param_bn::ParameterId>,
    retained_implicit: Vec<biodivine_lib_param_bn::VariableId>,
    parameter_mapping: HashMap<biodivine_lib_param_bn::VariableId, biodivine_lib_bdd::BddVariable>,
}

#[pymethods]
impl ColoredPerturbationSet {
    /// Normally, a new `ColoredPerturbationSet` is derived using an
    /// `AsynchronousPerturbationGraph`. However, in some cases you may want to create it
    /// manually from an `AsynchronousPerturbationGraph` and a `Bdd`.
    ///
    /// Just keep in mind that this method does not check that the provided `Bdd` is semantically
    /// a valid colored set of perturbations.
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

    fn __richcmp__(&self, py: Python, other: &Self, op: CompareOp) -> Py<PyAny> {
        match op {
            CompareOp::Eq => ColoredPerturbationSet::semantic_eq(self, other).into_py(py),
            CompareOp::Ne => ColoredPerturbationSet::semantic_eq(self, other)
                .not()
                .into_py(py),
            _ => py.NotImplemented(),
        }
    }

    fn __str__(&self) -> String {
        format!(
            "ColoredPerturbationSet(cardinality={}, symbolic_size={})",
            self.cardinality(),
            self.symbolic_size(),
        )
    }

    fn __repr__(&self, py: Python) -> String {
        format!(
            "ColoredPerturbationSet(cardinality={}, colors={}, perturbations={}, symbolic_size={})",
            self.cardinality(),
            self.colors(py).cardinality(),
            self.perturbations().cardinality(),
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

    fn __iter__(&self, py: Python) -> PyResult<_ColorPerturbationModelIterator> {
        self.items(py, None, None)
    }

    fn __ctx__(&self) -> Py<AsynchronousPerturbationGraph> {
        self.ctx.clone()
    }

    /// Returns the number of vertex-color pairs that are represented in this set.
    fn cardinality(&self) -> BigInt {
        let pruner = PerturbationSet::mk_duplicate_pruning_bdd(self.ctx.get());
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
        let parameter_variables = u16::try_from(
            self.ctx
                .get()
                .as_native()
                .as_symbolic_context()
                .num_parameter_variables(),
        )
        .unwrap();
        // A perturbation set uses all parameter variables, plus the portion of state variables
        // that is perturbable.
        let unused_variables = all_variables - parameter_variables - perturbation_variables;
        pruned.exact_cardinality().shr(unused_variables)
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

    /// True if this set is a singleton, i.e. a single perturbation-color pair.
    fn is_singleton(&self, py: Python) -> PyResult<bool> {
        let mut it = self.__iter__(py)?;
        let fst = it.native.next();
        let snd = it.native.next();
        Ok(fst.is_some() && snd.is_none())
    }

    /// The number of `Bdd` nodes that are used to represent this set.
    fn symbolic_size(&self) -> usize {
        self.as_native().symbolic_size()
    }

    /// Compute the existential projection of this relation to the color component.
    /// I.e. returns a set of colors such that for each color, there is at least one
    /// perturbation-color pair in the original set.
    ///
    /// *Note that this also fixes perturbation parameters to `False`, meaning the set no longer
    /// meaningfully depends on them. However, they do still appear in the symbolic encoding
    /// to ensure that the result is compatible with
    /// the underlying `AsynchronousPerturbationGraph`.*
    fn colors(&self, py: Python) -> ColorSet {
        let colors = self.as_native().colors();
        // This is slightly inefficient, but basically, we first want to project away all
        // perturbation parameters, and then we want to fix them to `false`.
        let colors =
            AsynchronousPerturbationGraph::mk_perturbable_color_set(self.ctx.bind(py), &colors);
        AsynchronousPerturbationGraph::mk_unperturbable_color_set(
            self.ctx.bind(py),
            colors.as_native(),
        )
    }

    /// Compute the existential projection of this relation to the perturbation component. I.e.
    /// returns a set of perturbations such that for each perturbation, there is at least one
    /// perturbation-color pair in the original set.
    pub fn perturbations(&self) -> PerturbationSet {
        // Project away everything that isn't a state variable or a perturbation parameter.
        let native_graph = self.ctx.get().as_native();
        let native_context = native_graph.as_original().symbolic_context();
        let mut retain = native_graph
            .get_perturbation_bdd_mapping(native_graph.perturbable_variables())
            .into_values()
            .collect::<HashSet<_>>();
        for s_var in native_context.state_variables() {
            retain.insert(*s_var);
        }

        let remove = native_context
            .bdd_variable_set()
            .variables()
            .into_iter()
            .filter(|it| !retain.contains(it))
            .collect::<Vec<_>>();

        let bdd = self.native.as_bdd().exists(&remove);
        PerturbationSet::mk_native(self.ctx.clone(), self.as_native().copy(bdd))
    }

    /// Retain only those perturbation-color pairs for which the color is also contained in the
    /// given `colors` set.
    fn intersect_colors(&self, py: Python, colors: &ColorSet) -> Self {
        let colors = AsynchronousPerturbationGraph::mk_perturbable_color_set(
            self.ctx.bind(py),
            colors.as_native(),
        );
        self.mk_derived(self.as_native().intersect_colors(colors.as_native()))
    }

    /// Retain only those perturbation-color pairs for which the perturbation is also contained
    /// in the given `perturbations` set.
    fn intersect_perturbations(&self, perturbations: &PerturbationSet) -> Self {
        self.mk_derived(self.as_native().intersect(perturbations.as_native()))
    }

    /// Remove all perturbation-color pairs for which the color is present in the
    /// given `colors` set.
    fn minus_colors(&self, py: Python, colors: &ColorSet) -> Self {
        let colors = AsynchronousPerturbationGraph::mk_perturbable_color_set(
            self.ctx.bind(py),
            colors.as_native(),
        );
        self.mk_derived(self.as_native().minus_colors(colors.as_native()))
    }

    /// Remove all perturbation-color pairs for which the perturbation is present in the
    /// given `perturbations` set.
    fn minus_perturbations(&self, perturbations: &PerturbationSet) -> Self {
        self.mk_derived(self.as_native().minus(perturbations.as_native()))
    }

    /// Return the subset of this relation that has all the perturbations fixed according to the
    /// provided values.
    ///
    /// To specify that a variable should be unperturbed, use `"var": None`. Any variable that
    /// should remain unrestricted should be completely omitted from the `perturbations`
    /// dictionary. This is similar to `AsynchronousPerturbationGraph.mk_perturbations`.
    fn select_perturbations(
        &self,
        py: Python,
        perturbations: &Bound<'_, PyDict>,
    ) -> PyResult<ColoredPerturbationSet> {
        let borrowed = self.ctx.borrow(py);
        let parent = borrowed.as_ref();
        let native_graph = self.ctx.get().as_native();
        let mapping =
            native_graph.get_perturbation_bdd_mapping(native_graph.perturbable_variables());
        let mut selection = biodivine_lib_bdd::BddPartialValuation::empty();

        // Go through the given perturbation and mark everything that should be perturbed.
        for (k, v) in perturbations {
            let k_var = parent.resolve_network_variable(&k)?;
            let s_var = parent
                .as_native()
                .symbolic_context()
                .get_state_variable(k_var);
            let Some(p_var) = mapping.get(&k_var).cloned() else {
                return throw_runtime_error(format!("Variable {k_var} cannot be perturbed."));
            };

            match v.extract::<Option<bool>>()? {
                Some(val) => {
                    selection.set_value(p_var, true);
                    selection.set_value(s_var, val);
                }
                None => {
                    selection.set_value(p_var, false);
                }
            }
        }

        let bdd = self.as_native().as_bdd().select(&selection.to_values());
        let native = GraphColoredVertices::new(bdd, native_graph.as_symbolic_context());

        Ok(ColoredPerturbationSet::mk_native(self.ctx.clone(), native))
    }

    /// Return the set of colors for which the given perturbation exists in this set.
    ///
    /// *Note that here, we assume that the dictionary represents a single perturbation. Therefore,
    /// any missing perturbable variables are treated as unperturbed.* This is the same behavior
    /// as in `AsynchronousPerturbationGraph.mk_perturbation`.
    ///
    fn select_perturbation(
        &self,
        py: Python,
        perturbation: &Bound<'_, PyDict>,
    ) -> PyResult<ColorSet> {
        let borrowed = self.ctx.borrow(py);
        let parent = borrowed.as_ref();
        let native_graph = self.ctx.get().as_native();
        let mapping =
            native_graph.get_perturbation_bdd_mapping(native_graph.perturbable_variables());
        let mut restriction = biodivine_lib_bdd::BddPartialValuation::empty();

        // Initially, set all variables to unperturbed.
        for var in mapping.values() {
            restriction.set_value(*var, false);
        }

        // Then go through the given perturbation and mark everything that should be perturbed.
        for (k, v) in perturbation {
            let k_var = parent.resolve_network_variable(&k)?;
            let s_var = parent
                .as_native()
                .symbolic_context()
                .get_state_variable(k_var);
            let Some(p_var) = mapping.get(&k_var).cloned() else {
                return throw_runtime_error(format!("Variable {k_var} cannot be perturbed."));
            };

            if let Some(val) = v.extract::<Option<bool>>()? {
                restriction.set_value(p_var, true);
                restriction.set_value(s_var, val);
            }
        }

        // This should remove all perturbation parameters and state variables (unperturbed
        // state variables should already be unconstrained).
        let restricted_bdd = self.as_native().as_bdd().restrict(&restriction.to_values());
        let colors = GraphColors::new(restricted_bdd, native_graph.as_symbolic_context());

        // Finally, fix all perturbation parameters to false, as per our convention.
        Ok(AsynchronousPerturbationGraph::mk_unperturbable_color_set(
            self.ctx.bind(py),
            &colors,
        ))
    }

    /// Return the global robustness of the given perturbation as represented in this colored set.
    ///
    /// This is the fraction of colors for which the perturbation is present w.r.t. all colors
    /// that are admissible in the unperturbed system. Note that this has no relation to the
    /// total set of colors stored in this relation (i.e. `set.colors()`).
    ///
    /// *Note that here, we assume that the dictionary represents a single perturbation. Therefore,
    /// any missing perturbable variables are treated as unperturbed.*
    ///
    /// See also `AsynchronousPerturbationGraph.colored_robustness`.
    fn perturbation_robustness(
        &self,
        py: Python,
        perturbation: &Bound<'_, PyDict>,
    ) -> PyResult<f64> {
        let colors = self.select_perturbation(py, perturbation)?;
        AsynchronousPerturbationGraph::colored_robustness(self.ctx.bind(py).clone(), &colors)
    }

    /// Only retain those perturbations that have the given `size`. If `up_to` is set to `True`,
    /// then retain perturbations that have smaller or equal size.
    ///
    /// This is similar to `AsynchronousPerturbationGraph.mk_perturbations_with_size`.
    fn select_by_size(&self, py: Python, size: usize, up_to: bool) -> ColoredPerturbationSet {
        let sized = AsynchronousPerturbationGraph::mk_perturbations_with_size(
            self.ctx.clone(),
            py,
            size,
            up_to,
        );
        self.intersect_perturbations(&sized)
    }

    /// Select all perturbations from this relation whose robustness is at least the given
    /// `threshold`.
    ///
    /// Since this operation cannot be completed symbolically, the result is a list of explicit
    /// `PerturbationModel` instances, together with their robustness and their `ColorSet`.
    /// The perturbations are returned from smallest to largest (in terms of the number of
    /// perturbed variables, not robustness). Optionally, you can use `result_limit` to restrict
    /// the maximal number of returned perturbations.
    ///
    /// You can also use `ColoredPerturbationSet.select_by_size` to first only select perturbations
    /// of a specific size and only then enumerate their robustness.
    #[pyo3(signature = (threshold, result_limit = None))]
    fn select_by_robustness(
        &self,
        py: Python,
        threshold: f64,
        result_limit: Option<usize>,
    ) -> PyResult<Vec<(PerturbationModel, f64, ColorSet)>> {
        if threshold <= 0.0 || threshold > 1.0 {
            return throw_runtime_error("Threshold should be in the (0, 1] interval.");
        }

        let mut results = Vec::new();

        let size_limit = self.ctx.get().as_native().perturbable_variables().len();
        let perturbations = self.perturbations();
        for size in 0..=size_limit {
            let size_perturbations = AsynchronousPerturbationGraph::mk_perturbations_with_size(
                self.ctx.clone(),
                py,
                size,
                false,
            );
            let limited_perturbations = perturbations.intersect(&size_perturbations);

            let mut iterator = limited_perturbations.items(py, None)?;
            while let Some(model) = iterator.__next__() {
                let color_set = self
                    .intersect_perturbations(&model.to_symbolic())
                    .colors(py);

                let robustness = AsynchronousPerturbationGraph::colored_robustness(
                    self.ctx.bind(py).clone(),
                    &color_set,
                )?;

                if robustness >= threshold {
                    results.push((model, robustness, color_set));
                }

                if let Some(limit) = result_limit {
                    if results.len() >= limit {
                        return Ok(results);
                    }
                }
            }
        }

        Ok(results)
    }

    /// Deterministically pick a subset of this set that contains exactly a single
    /// perturbation-color pair.
    ///
    /// If this set is empty, the result is also empty.
    fn pick_singleton(&self, py: Python) -> PyResult<Self> {
        let mut it = self.__iter__(py)?;
        let Some((c_model, p_model)) = it.__next__() else {
            return Ok(
                AsynchronousPerturbationGraph::mk_empty_colored_perturbations(self.ctx.clone()),
            );
        };
        let singleton = p_model
            .to_symbolic()
            .extend_with_colors(&c_model.to_symbolic());
        Ok(singleton)
    }

    /// Obtain the underlying `Bdd` of this `ColoredPerturbationSet`.
    pub fn to_bdd(&self, py: Python) -> Bdd {
        let rs_bdd = self.as_native().as_bdd().clone();
        let ctx = self.ctx.borrow(py).as_ref().symbolic_context();
        let ctx_ref = ctx.borrow(py);
        Bdd::new_raw_2(ctx_ref.bdd_variable_set(), rs_bdd)
    }

    /// Obtain the internal representation of this `ColoredPerturbationSet`, which uses the
    /// `AsynchronousPerturbationGraph` encoding. This is a colored set of vertices, where
    /// the colors depend on the perturbation parameters and normal parameters, but the vertices
    /// are only constrained in case the variable is perturbed.
    fn to_internal(&self, py: Python) -> ColoredVertexSet {
        ColoredVertexSet::mk_native(
            self.ctx.borrow(py).as_ref().symbolic_context(),
            self.native.clone(),
        )
    }

    /// Returns an iterator over all perturbation-color pairs in this `ColoredPerturbationSet`
    /// relation, with an optional projection to a subset of network variables and
    /// uninterpreted functions.
    ///
    /// *Note that the perturbation parameters are automatically projected away from the
    /// returned `ColorModel` instances.*
    ///
    /// When no `retained` collections are specified, this is equivalent to
    /// `ColoredPerturbationSet.__iter__`. However, if a retained collection is given, the
    /// resulting iterator only considers unique combinations of the `retained`
    /// functions and variables. Consequently, the resulting `ColorModel` and `PerturbationModel`
    /// instances will fail with an `IndexError` if a value outside the `retained` set
    /// is requested (for the `PerturbationModel`, `IndexError` is also thrown if a value
    /// of a variable that is not perturbable is requested).
    ///
    /// Note that if you set `retained_variables = []` and `retained_functions = None`, this is
    /// equivalent to `set.colors().items()`. Similarly, with `retained_variables = None` and
    /// `retained_functions = []`, this is equivalent to `set.perturbations().items()`.
    #[pyo3(signature = (retained_variables = None, retained_functions = None))]
    fn items(
        &self,
        py: Python,
        retained_variables: Option<&Bound<'_, PyList>>,
        retained_functions: Option<&Bound<'_, PyList>>,
    ) -> PyResult<_ColorPerturbationModelIterator> {
        let ctx = self.ctx.get();
        let symbolic_ctx_py = self.ctx.borrow(py).as_ref().symbolic_context();
        let symbolic_ctx = symbolic_ctx_py.get();
        // First, extract all functions that should be retained (see also ColorSet.items).
        // However, in this case, skip all perturbation parameters.
        let mut retained_explicit = Vec::new();
        let mut retained_implicit = Vec::new();
        let mut retained_functions = if let Some(retained) = retained_functions {
            let mut result = Vec::new();
            for x in retained {
                let function = symbolic_ctx.resolve_function(&x)?;
                let table = match function {
                    Either::Left(x) => {
                        if retained_implicit.contains(&x) {
                            continue;
                        }
                        retained_implicit.push(x);
                        symbolic_ctx
                            .as_native()
                            .get_implicit_function_table(x)
                            .unwrap()
                    }
                    Either::Right(x) => {
                        if retained_explicit.contains(&x) {
                            continue;
                        }
                        retained_explicit.push(x);
                        symbolic_ctx.as_native().get_explicit_function_table(x)
                    }
                };
                result.append(&mut table.symbolic_variables().clone());
            }
            result
        } else {
            retained_explicit.append(
                &mut symbolic_ctx
                    .as_native()
                    .network_parameters()
                    .collect::<Vec<_>>(),
            );
            retained_implicit.append(&mut symbolic_ctx.as_native().network_implicit_parameters());
            symbolic_ctx.as_native().parameter_variables().clone()
        };
        retained_explicit.sort();
        retained_implicit.sort();

        // Remove all perturbation parameters from retained functions:

        let perturbation_parameters = ctx
            .as_native()
            .perturbable_variables()
            .iter()
            .map(|v| ctx.as_native().get_perturbation_parameter(*v).unwrap())
            .collect::<HashSet<_>>();
        let perturbation_s_vars = ctx
            .as_native()
            .get_perturbation_bdd_mapping(ctx.as_native().perturbable_variables())
            .into_values()
            .collect::<HashSet<_>>();

        retained_explicit.retain(|it| !perturbation_parameters.contains(it));
        retained_functions.retain(|it| !perturbation_s_vars.contains(it));

        // Then add all retained network variables and their perturbation parameters
        // (see also PerturbationSet.items).
        let parameter_mapping = ctx
            .as_native()
            .get_perturbation_bdd_mapping(ctx.as_native().perturbable_variables());
        let retained_network_variables = if let Some(retained) = retained_variables {
            retained
                .iter()
                .map(|it| symbolic_ctx.resolve_network_variable(&it))
                .collect::<PyResult<Vec<_>>>()?
        } else {
            ctx.as_native().perturbable_variables().clone()
        };

        let mut retained_variables = Vec::new();
        for var in retained_network_variables {
            if let Some(p_var) = parameter_mapping.get(&var) {
                retained_variables.push(symbolic_ctx.as_native().get_state_variable(var));
                retained_variables.push(*p_var)
            } else {
                return throw_runtime_error(format!("Variable {var} is not perturbable."));
            }
        }

        let mut retained = Vec::new();
        retained.append(&mut retained_functions);
        retained.append(&mut retained_variables);
        retained.sort();

        let pruner = PerturbationSet::mk_duplicate_pruning_bdd(self.ctx.get());
        let bdd = self.as_native().as_bdd();

        let projection = RawProjection::new(retained, &pruner.and(bdd));
        Ok(_ColorPerturbationModelIterator {
            ctx: symbolic_ctx_py,
            graph: self.ctx.clone(),
            native: projection.into_iter(),
            retained_implicit,
            retained_explicit,
            parameter_mapping,
        })
    }
}

impl AsNative<GraphColoredVertices> for ColoredPerturbationSet {
    fn as_native(&self) -> &GraphColoredVertices {
        &self.native
    }

    fn as_native_mut(&mut self) -> &mut GraphColoredVertices {
        &mut self.native
    }
}

impl ColoredPerturbationSet {
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
}

#[pymethods]
impl _ColorPerturbationModelIterator {
    fn __iter__(self_: Py<Self>) -> Py<Self> {
        self_
    }

    fn __next__(&mut self) -> Option<(ColorModel, PerturbationModel)> {
        self.native.next().map(|it| {
            // Here, we have to create two copies of the partial valuation that don't have
            // any "invalid" variables, otherwise those could propagate further by instantiating
            // a symbolic set from the model object.
            let mut color_val = it.clone();
            let mut pert_val = BddPartialValuation::empty();
            let native_ctx = self.ctx.get().as_native();
            // From the color model, we only remove the state variables (perturbation parameters
            // are kept to ensure compatibility).
            // From `pert_val`, we copy available perturbation parameter,
            // and also copy state variable if it is perturbed.
            for n_var in native_ctx.network_variables() {
                let s_var = native_ctx.get_state_variable(n_var);

                // Clear every state variable from color valuation.
                color_val.unset_value(s_var);

                // Copy perturbation parameter (and state if relevant).
                if let Some(p_var) = self.parameter_mapping.get(&n_var) {
                    if let Some(value) = it.get_value(*p_var) {
                        pert_val.set_value(*p_var, value);
                        if value {
                            pert_val.set_value(s_var, it.get_value(s_var).unwrap())
                        }
                    }
                }
            }

            let color = ColorModel::new_native(
                self.ctx.clone(),
                color_val,
                self.retained_implicit.clone(),
                self.retained_explicit.clone(),
            );
            let vertex = PerturbationModel::new_native(
                self.graph.clone(),
                pert_val,
                self.parameter_mapping.clone(),
            );
            (color, vertex)
        })
    }
}
