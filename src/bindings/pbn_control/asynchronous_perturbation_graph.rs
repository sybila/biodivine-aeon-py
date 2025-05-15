use std::collections::{HashMap, HashSet};

use biodivine_lib_param_bn::biodivine_std::bitvector::ArrayBitVector;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, GraphColors};
use biodivine_pbn_control::perturbation::PerturbationGraph;
use macros::Wrapper;
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::bindings::lib_bdd::bdd_variable::BddVariable;
use crate::bindings::lib_param_bn::boolean_network::BooleanNetwork;
use crate::bindings::lib_param_bn::parameter_id::ParameterId;
use crate::bindings::lib_param_bn::symbolic::asynchronous_graph::AsynchronousGraph;
use crate::bindings::lib_param_bn::symbolic::set_color::ColorSet;
use crate::bindings::lib_param_bn::symbolic::set_colored_vertex::ColoredVertexSet;
use crate::bindings::lib_param_bn::variable_id::VariableId;
use crate::bindings::lib_param_bn::NetworkVariableContext;
use crate::bindings::pbn_control::control::sanitize_control_map;
use crate::bindings::pbn_control::set_colored_perturbation::ColoredPerturbationSet;
use crate::bindings::pbn_control::{PerturbationModel, PerturbationSet};
use crate::pyo3_utils::BoolLikeValue;
use crate::{throw_runtime_error, throw_type_error, AsNative};

/// An extension of `AsynchronousGraph` that admits various variable perturbations through
/// additional colors/parameters. Such graph can then be analyzed to extract control strategies
/// (perturbations) that are sufficient to achieve a particular outcome (an attractor or
/// a phenotype).
///
/// This representation is similar to `SymbolicSpaceContext` in the sense that it introduces
/// additional variables into the symbolic encoding in order to encode more complex modes of
/// behavior in a BN. However, in this case, it is also necessary to modify the actual update
/// functions of the network. Hence, this implementation extends the `AsynchronousGraph` directly.
///
/// To represent perturbations, `AsynchronousPerturbedGraph` introduces the following
/// changes to the network dynamics:
///
///  - For each variable (that can be perturbed), we create an explicit Boolean
///  "perturbation parameter".
///  - Implicit parameters are given explicit names, since we may need to augment the update
///  functions of these variables with perturbation parameters.
///  - We maintain two versions of network dynamics: *original* (unperturbed), meaning the additional
///  parameters have no impact on the update functions, and *perturbed*, where a variable is
///  allowed to evolve only if it is not perturbed.
///  - This representation allows us to also encode sets of perturbations, since for a perturbed
///  variable, we can use the state variable (that would otherwise be unused) to represent
///  the value to which the variable is perturbed.
///
/// Note that this encoding does not implicitly assume any perturbation temporality (one-step,
/// permanent, temporary). These aspects are managed by the analysis algorithms.
///
/// *By default, `PerturbationAsynchronousGraph` behaves as if all variables are unperturbed
/// and the newly introduced parameters are set to `False`, i.e. unperturbed. The perturbation
/// parameters always appear in the symbolic encoding, they are just not considered in the update
/// functions. To access the "perturbed" dynamics,
/// see `AsynchronousPerturbationGraph.to_perturbed`.*
///
#[pyclass(module="biodivine_aeon", extends=AsynchronousGraph, frozen)]
#[derive(Clone, Wrapper)]
pub struct AsynchronousPerturbationGraph(PerturbationGraph);

#[pymethods]
impl AsynchronousPerturbationGraph {
    /// Build a new `AsynchronousPerturbationGraph` for the given `BooleanNetwork`. Optionally
    /// also specify a list of variables that can be perturbed in the resulting graph
    /// (otherwise all variables can be perturbed).
    #[new]
    #[pyo3(signature = (network, perturb = None))]
    pub fn new(
        py: Python,
        network: &Bound<'_, BooleanNetwork>,
        perturb: Option<&Bound<'_, PyList>>,
    ) -> PyResult<(AsynchronousPerturbationGraph, AsynchronousGraph)> {
        let n_ref = network.borrow();
        let perturb_native = if let Some(perturb) = perturb {
            perturb
                .iter()
                .map(|it| n_ref.resolve_network_variable(&it))
                .collect::<PyResult<Vec<_>>>()?
        } else {
            Vec::from_iter(n_ref.as_native().variables())
        };

        let implicit_params = n_ref.as_native().implicit_parameters();
        if !implicit_params.is_empty() {
            return throw_runtime_error(
                "`PerturbationGraph` cannot be created from a network with implicit parameters.\nUse `BooleanNetwork.name_implicit_parameters` to assign names to all anonymous functions."
            );
        }

        let stg = PerturbationGraph::with_restricted_variables(n_ref.as_native(), perturb_native);
        let parent = stg.as_original().clone();

        Ok((stg.into(), AsynchronousGraph::wrap_native(py, parent)?))
    }

    /*
       Currently, we override those methods of `AsynchronousGraph` that create new `ColorSet`
       instances. In these cases, we need to further restrict the result to ensure that the
       perturbation parameters are set to `False` by default.

        Note that we do not override `transfer_from`, since here, it is expected that the
        perturbation parameters remain unconstrained (furthermore, one could use transfer from
        with another perturbed graph, in which case these must be preserved).
    */

    pub fn __str__(_self: Bound<'_, Self>) -> String {
        let ctx = _self.borrow().as_ref().symbolic_context();
        format!("AsynchronousPerturbationGraph({})", ctx.get().__str__())
    }

    /// Reconstruct the `BooleanNetwork` that represents the *unperturbed* dynamics of this graph.
    /// The network does not contain any perturbation parameters.
    ///
    /// (see also `AsynchronousGraph.reconstruct_network`).
    pub fn reconstruct_network(_self: Bound<'_, Self>, py: Python) -> PyResult<Py<BooleanNetwork>> {
        let result = _self.borrow().as_ref().reconstruct_network(py)?;
        let result_ref = result.borrow(py);
        result_ref.prune_unused_parameters(py)
    }

    /// Return a "unit" (i.e. full) `ColoredVertexSet`, with the perturbation
    /// parameters all fixed to `False`.
    pub fn mk_unit_colored_vertices(_self: Bound<'_, Self>) -> ColoredVertexSet {
        let unit = _self.borrow().as_ref().mk_unit_colored_vertices();
        Self::mk_unperturbable_colored_vertex_set(&_self, unit.as_native())
    }

    /// Return a "unit" (i.e. full) `ColorSet`, with the perturbation
    /// parameters all fixed to `False`.
    pub fn mk_unit_colors(_self: Bound<'_, Self>) -> ColorSet {
        let unit = _self.borrow().as_ref().mk_unit_colors();
        Self::mk_unperturbable_color_set(&_self, unit.as_native())
    }

    /// A version of `AsynchronousGraph.mk_function_row_colors` that also fixes the perturbation
    /// parameters for `False`.
    pub fn mk_function_row_colors(
        _self: Bound<'_, Self>,
        function: &Bound<'_, PyAny>,
        row: &Bound<'_, PyList>,
        value: BoolLikeValue,
    ) -> PyResult<ColorSet> {
        let result = _self
            .borrow()
            .as_ref()
            .mk_function_row_colors(function, row, value)?;
        Ok(Self::mk_unperturbable_color_set(&_self, result.as_native()))
    }

    /// A version of `AsynchronousGraph.mk_function_colors` that also fixes the perturbation
    /// parameters for `False`.
    pub fn mk_function_colors(
        _self: Bound<'_, Self>,
        function: &Bound<'_, PyAny>,
        value: &Bound<'_, PyAny>,
    ) -> PyResult<ColorSet> {
        let result = _self
            .borrow()
            .as_ref()
            .mk_function_colors(function, value)?;
        Ok(Self::mk_unperturbable_color_set(&_self, result.as_native()))
    }

    /// A version of `AsynchronousGraph.mk_subspace` that also fixes the perturbation
    /// parameters for `False`.
    pub fn mk_subspace(
        _self: Bound<'_, Self>,
        subspace: &Bound<'_, PyAny>,
    ) -> PyResult<ColoredVertexSet> {
        let result = _self.borrow().as_ref().mk_subspace(subspace)?;
        Ok(Self::mk_unperturbable_colored_vertex_set(
            &_self,
            result.as_native(),
        ))
    }

    /*
       Following are the additional methods introduced for handling perturbations.
    */

    /// A copy of the *base* `BooleanNetwork` that was used to create this graph,
    /// without additional perturbation parameters or any modification (e.g. still with all
    /// implicit parameters).
    ///
    /// Returns the original Boolean network without perturbation parameters or modifications.
    ///
    /// This operation does not preserve annotations from the original network.
    ///
    /// # Examples
    ///
    /// ```
    /// let py = Python::acquire_gil().python();
    /// let base_net = graph.base_network(py).unwrap();
    /// ```    pub fn base_network(&self, py: Python) -> PyResult<Py<BooleanNetwork>> {
        // Here, `unwrap` is safe because we know that perturbed graph is only created with
        // a network object.
        let bn = self
            .as_native()
            .as_non_perturbable()
            .as_network()
            .unwrap()
            .clone();
        BooleanNetwork::from(bn).export_to_python(py, None)
    }

    /// A copy of the `BooleanNetwork` with the extra perturbation parameters, but with the
    /// update functions unaffected.
    ///
    /// Returns the Boolean network with perturbation parameters included but original (unperturbed) update functions.
    ///
    /// The resulting network contains the perturbation parameters as variables, but the update logic remains unchanged from the original network. This operation does not preserve annotations from the original network.
    ///
    /// # Examples
    ///
    /// ```
    /// let py = Python::acquire_gil().python();
    /// let net = graph.unperturbed_network(py).unwrap();
    /// assert!(net.is_instance_of::<BooleanNetwork>(py).unwrap());
    /// ```    pub fn unperturbed_network(&self, py: Python) -> PyResult<Py<BooleanNetwork>> {
        // Here, `unwrap` is safe because we know that perturbed graph is only created with
        // a network object.
        let bn = self.as_native().as_original().as_network().unwrap().clone();
        BooleanNetwork::from(bn).export_to_python(py, None)
    }

    /// A copy of the `BooleanNetwork` with the extra perturbation parameters and with the
    /// update functions changed to reflect the perturbations.
    ///
    /// Returns the Boolean network with perturbation parameters and perturbed update functions.
    ///
    /// The resulting network includes perturbation parameters for each perturbable variable, and its update functions are modified so that perturbed variables cannot update. This operation does not preserve annotations from the original network.
    ///
    /// # Examples
    ///
    /// ```
    /// let py = Python::acquire_gil().python();
    /// let perturbed = graph.perturbed_network(py).unwrap();
    /// assert!(perturbed.has_perturbation_parameters());
    /// ```    pub fn perturbed_network(&self, py: Python) -> PyResult<Py<BooleanNetwork>> {
        // Here, `unwrap` is safe because we know that perturbed graph is only created with
        // a network object.
        let bn = self
            .as_native()
            .as_perturbed()
            .as_network()
            .unwrap()
            .clone();
        BooleanNetwork::from(bn).export_to_python(py, None)
    }

    /// A copy of the `AsynchronousGraph` that represents the *unperturbed* asynchronous
    /// dynamics of this network. It supports the additional parameters necessary to represent
    /// perturbations, but does not actually use them in any meaningful way.
    ///
    /// This is effectively the "parent" implementation of this instance, so you can already
    /// access these methods directly by calling them on this graph. Just keep in mind that
    /// methods that return color sets do not fix the perturbation parameters to `False` in
    /// the "parent" implementation.
    ///
    /// See also `AsynchronousPerturbationGraph.to_perturbed()`.
    pub fn to_original(&self, py: Python) -> PyResult<AsynchronousGraph> {
        AsynchronousGraph::wrap_native(py, self.as_native().as_original().clone())
    }

    /// A copy of the `AsynchronousGraph` that represents the *perturbed* asynchronous
    /// dynamics of this network. It supports the additional parameters necessary to represent
    /// perturbations, and they do affect the state-transitions: In colors where a variable
    /// is perturbed, it cannot be updated.
    ///
    /// See also `AsynchronousPerturbationGraph.to_original()`.
    pub fn to_perturbed(&self, py: Python) -> PyResult<AsynchronousGraph> {
        AsynchronousGraph::wrap_native(py, self.as_native().as_perturbed().clone())
    }

    /// List of variables that can be perturbed in this graph.
    pub fn perturbable_network_variables(&self) -> Vec<VariableId> {
        self.as_native()
            .perturbable_variables()
            .iter()
            .map(|it| VariableId::from(*it))
            .collect()
    }

    /// List of names of variables that can be perturbed in this graph.
    pub fn perturbable_network_variable_names(&self) -> Vec<String> {
        self.as_native()
            .perturbable_variables()
            .iter()
            .map(|it| self.as_native().as_original().get_variable_name(*it))
            .collect()
    }

    /// Find the `ParameterId` which corresponds to the synthetic parameter that is used to
    /// encode that the given `variable` is perturbed (i.e. fixed and cannot evolve).
    pub fn get_perturbation_parameter(
        _self: &Bound<'_, AsynchronousPerturbationGraph>,
        variable: &Bound<'_, PyAny>,
    ) -> PyResult<ParameterId> {
        let graph = _self.borrow();
        let n_variable = graph.as_ref().resolve_network_variable(variable)?;
        let native = graph.as_native().get_perturbation_parameter(n_variable);
        if let Some(native) = native {
            Ok(ParameterId::from(native))
        } else {
            let name = graph.as_ref().get_network_variable_name(variable)?;
            throw_runtime_error(format!("Variable {:?} cannot be perturbed.", name))
        }
    }

    /// The list of `ParameterId` objects that identify the perturbation parameters
    /// of this graph.
    pub fn perturbation_parameters(&self) -> Vec<ParameterId> {
        self.as_native()
            .perturbable_variables()
            .iter()
            .map(|var| {
                let p = self.as_native().get_perturbation_parameter(*var).unwrap();
                p.into()
            })
            .collect()
    }

    /// The dictionary of all `VariableId`, `BddVariable` identifier pairs that correspond to
    /// the symbolic encoding of perturbation parameters of the respective network variables.
    pub fn perturbation_bdd_variables(&self) -> HashMap<VariableId, BddVariable> {
        let map = self
            .as_native()
            .get_perturbation_bdd_mapping(self.as_native().perturbable_variables());
        map.into_iter().map(|(a, b)| (a.into(), b.into())).collect()
    }

    /// Compute the `ColoredPerturbationSet` which causes the network to go from the `source`
    /// state into one of the `target` states.
    ///
    /// In other words, if you fix all the variables prescribed by one of the resulting
    /// perturbations in the `source` state, you obtain one of the `target` states.
    ///
    /// This operation is mostly used internally in various control algorithms.
    ///
    pub fn post_via_perturbation(
        _self: Bound<'_, Self>,
        source: &Bound<'_, PyList>,
        target: &ColoredVertexSet,
    ) -> PyResult<ColoredPerturbationSet> {
        let source = source
            .iter()
            .map(|it| it.extract::<BoolLikeValue>().map(bool::from))
            .collect::<PyResult<Vec<bool>>>()?;
        let source = ArrayBitVector::from(source);
        let result = _self
            .borrow()
            .as_native()
            .post_perturbation(&source, target.as_native());
        Ok(sanitize_control_map(
            _self.unbind(),
            result.as_bdd().clone(),
        ))
    }

    /// Return the set of all perturbations that are valid in this graph.
    pub fn mk_unit_perturbations(_self: Py<Self>) -> PerturbationSet {
        Self::mk_unit_colored_perturbations(_self).perturbations()
    }

    /// Return an empty set of perturbations.
    pub fn mk_empty_perturbations(_self: Py<Self>) -> PerturbationSet {
        let empty = _self.get().as_native().mk_empty_colored_vertices();
        PerturbationSet::mk_native(_self, empty)
    }

    /// Return the set of all perturbation-color pairs that are valid in this graph.
    pub fn mk_unit_colored_perturbations(_self: Py<Self>) -> ColoredPerturbationSet {
        let unit = _self.get().as_native().mk_unit_colored_vertices();
        ColoredPerturbationSet::mk_native(_self, unit)
    }

    /// Return an empty set of color-perturbation pairs.
    pub fn mk_empty_colored_perturbations(_self: Py<Self>) -> ColoredPerturbationSet {
        let empty = _self.get().as_native().mk_empty_colored_vertices();
        ColoredPerturbationSet::mk_native(_self, empty)
    }

    /// Create a `ColorSet` with unconstrained perturbation parameters, meaning every variable
    /// that is declared as perturbable can be actually perturbed.
    ///
    /// Meanwhile, `AsynchronousPerturbationGraph.mk_unit_colors` returns a color set where
    /// perturbation parameters are set to `False` to better resemble the behavior of a "normal"
    /// `AsynchronousGraph`.
    pub fn mk_perturbable_unit_colors(_self: Py<Self>, py: Python) -> ColorSet {
        // Using the parent implementation means that the perturbation parameters
        // remain unconstrained.
        _self.borrow(py).as_ref().mk_unit_colors()
    }

    /// Create a `ColoredVertexSet` with unconstrained perturbation parameters, meaning every
    /// variable that is declared as perturbable can be actually perturbed.
    ///
    /// Meanwhile, `AsynchronousPerturbationGraph.mk_unit_colored_vertices` returns a color set
    /// where perturbation parameters are set to `False` to better resemble the behavior of
    /// a "normal" `AsynchronousGraph`.
    pub fn mk_perturbable_unit_colored_vertices(_self: Py<Self>, py: Python) -> ColoredVertexSet {
        // Using the parent implementation means that the perturbation parameters
        // remain unconstrained.
        _self.borrow(py).as_ref().mk_unit_colored_vertices()
    }

    /// Create a singleton `PerturbationSet` based on the given values of perturbable variables.
    ///
    /// The difference between this method and `AsynchronousPerturbationGraph.mk_perturbations`
    /// is in how missing values are treated: In `mk_perturbations`, a variable with an unspecified
    /// value is treated as unconstrained: i.e. it can be unperturbed, or perturbed to
    /// `False`/`True`. Meanwhile, `mk_perturbation` treats any unspecified value as unperturbed,
    /// since the result must always represent a single perturbation.
    pub fn mk_perturbation(
        _self: Py<Self>,
        py: Python,
        perturbation: &Bound<'_, PyAny>,
    ) -> PyResult<PerturbationSet> {
        let self_borrow = _self.borrow(py);
        let parent = self_borrow.as_ref();
        let mut partial_valuation = biodivine_lib_bdd::BddPartialValuation::empty();
        let perturbable = _self.get().as_native().perturbable_variables();
        let map = _self
            .get()
            .as_native()
            .get_perturbation_bdd_mapping(perturbable);

        // Init the partial valuation such that everything is unperturbed initially.
        for bdd_var in map.values() {
            partial_valuation.set_value(*bdd_var, false);
        }

        let perturbation = Self::resolve_perturbation(&self_borrow, perturbation)?;

        // Read data from the dictionary.
        for (k, v) in perturbation {
            let s_var = parent.as_native().symbolic_context().get_state_variable(k);
            let p_var = *map.get(&k).unwrap();

            match v {
                None => partial_valuation.set_value(p_var, false),
                Some(val) => {
                    partial_valuation.set_value(p_var, true);
                    partial_valuation.set_value(s_var, val);
                }
            }
        }

        let bdd = parent
            .as_native()
            .symbolic_context()
            .bdd_variable_set()
            .mk_conjunctive_clause(&partial_valuation);
        let set = GraphColoredVertices::new(bdd, parent.as_native().symbolic_context());
        Ok(PerturbationSet::mk_native(_self.clone(), set))
    }

    /// Create a set of perturbations that match the given dictionary of values.
    ///
    /// The dictionary should contain `True`/`False` for a perturbed variable and `None` for
    /// an unperturbed variable. If all perturbable variables are specified, the result is
    /// a singleton set. If some of the perturbable variables is missing from the dictionary,
    /// it is unconstrained and the result contains any perturbation that matches the description
    /// w.r.t. the remaining (specified) variables.
    ///
    pub fn mk_perturbations(
        _self: Py<Self>,
        py: Python,
        perturbations: &Bound<'_, PyAny>,
    ) -> PyResult<PerturbationSet> {
        let self_borrow = _self.borrow(py);
        let parent = self_borrow.as_ref();
        let mut partial_valuation = biodivine_lib_bdd::BddPartialValuation::empty();
        let perturbable = _self.get().as_native().perturbable_variables();
        let map = _self
            .get()
            .as_native()
            .get_perturbation_bdd_mapping(perturbable);
        let perturbations = Self::resolve_perturbation(&self_borrow, perturbations)?;
        for (k, v) in perturbations {
            let s_var = parent.as_native().symbolic_context().get_state_variable(k);
            let p_var = *map.get(&k).unwrap();

            match v {
                None => partial_valuation.set_value(p_var, false),
                Some(val) => {
                    partial_valuation.set_value(p_var, true);
                    partial_valuation.set_value(s_var, val);
                }
            }
        }

        let bdd = parent
            .as_native()
            .symbolic_context()
            .bdd_variable_set()
            .mk_conjunctive_clause(&partial_valuation);
        let set = GraphColoredVertices::new(bdd, parent.as_native().symbolic_context());
        Ok(PerturbationSet::mk_native(_self.clone(), set))
    }

    /// Create a set of perturbations of the given exact size (in terms of perturbed variables).
    /// If `size` is greater or equal to the number of perturbable variables, the result is
    /// equivalent to `AsynchronousPerturbationGraph.mk_unit_perturbations`.
    ///
    /// If the `up_to` parameter is given, the result contains all perturbations up to (including)
    /// the specified size.
    pub fn mk_perturbations_with_size(
        _self: Py<Self>,
        py: Python,
        size: usize,
        up_to: bool,
    ) -> PerturbationSet {
        let self_borrow = _self.borrow(py);
        let parent = self_borrow.as_ref();
        let perturbable = _self.get().as_native().perturbable_variables();
        let bdd_vars = _self
            .get()
            .as_native()
            .get_perturbation_bdd_mapping(perturbable)
            .into_values()
            .collect::<Vec<_>>();

        let bdd_ctx = parent.as_native().symbolic_context().bdd_variable_set();
        let bdd = if up_to {
            bdd_ctx.mk_sat_up_to_k(size, &bdd_vars)
        } else {
            bdd_ctx.mk_sat_exactly_k(size, &bdd_vars)
        };

        let set = GraphColoredVertices::new(bdd, parent.as_native().symbolic_context());
        PerturbationSet::mk_native(_self.clone(), set)
    }

    /// Compute the *robustness* of the given color set w.r.t. the unit color set.
    ///
    /// Note that this is essentially just `set.cardinality() / unit.cardinality()` with some
    /// additional measures taken to prevent floating point overflow with large numbers.
    pub fn colored_robustness(_self: Bound<'_, Self>, set: &ColorSet) -> PyResult<f64> {
        // We do not need to take perturbation parameters into account because they should be
        // unconstrained in any properly constructed `ColorSet`. So in the robustness fraction,
        // they cancel each other out and have no impact on the final value.

        let unit = AsynchronousPerturbationGraph::mk_unit_colors(_self);

        // The following method always give an approximation up to 6 decimal places, even if the
        // cardinality would overflow to f64::infinity.

        let p_card = set.as_native().exact_cardinality() * 1_000_000;
        let u_card = unit.as_native().exact_cardinality();

        let robustness: BigInt = p_card / u_card;

        Ok(robustness.to_f64().unwrap_or(f64::NAN) / 1_000_000.0)
    }
}

impl AsynchronousPerturbationGraph {
    /// Create a `ColorSet` where all the perturbation parameters are fixed to `false`, meaning
    /// no variable can be perturbed.
    pub fn mk_unperturbable_color_set(
        _self: &Bound<'_, AsynchronousPerturbationGraph>,
        set: &GraphColors,
    ) -> ColorSet {
        let self_ref = _self.borrow();
        let perturbable = _self.get().as_native().perturbable_variables();
        let map = _self
            .get()
            .as_native()
            .get_perturbation_bdd_mapping(perturbable)
            .into_values()
            .map(|v| (v, false))
            .collect::<Vec<_>>();
        let bdd = set.as_bdd();
        let bdd = bdd.select(&map);

        let ctx = self_ref.as_ref().symbolic_context();
        let set = GraphColors::new(bdd, ctx.get().as_native());
        ColorSet::mk_native(self_ref.as_ref().symbolic_context(), set)
    }

    pub fn mk_unperturbable_colored_vertex_set(
        _self: &Bound<'_, AsynchronousPerturbationGraph>,
        set: &GraphColoredVertices,
    ) -> ColoredVertexSet {
        let self_ref = _self.borrow();
        let perturbable = _self.get().as_native().perturbable_variables();
        let map = _self
            .get()
            .as_native()
            .get_perturbation_bdd_mapping(perturbable)
            .into_values()
            .map(|v| (v, false))
            .collect::<Vec<_>>();
        let bdd = set.as_bdd();
        let bdd = bdd.select(&map);

        let ctx = self_ref.as_ref().symbolic_context();
        let set = GraphColoredVertices::new(bdd, ctx.get().as_native());
        ColoredVertexSet::mk_native(self_ref.as_ref().symbolic_context(), set)
    }

    /// Create a `ColorSet` where all perturbation parameters are unconstrained, meaning everything
    /// is perturbable.
    pub fn mk_perturbable_color_set(
        _self: &Bound<'_, AsynchronousPerturbationGraph>,
        set: &GraphColors,
    ) -> ColorSet {
        let self_ref = _self.borrow();
        let perturbable = _self.get().as_native().perturbable_variables();
        let vars = _self
            .get()
            .as_native()
            .get_perturbation_bdd_mapping(perturbable)
            .into_values()
            .collect::<Vec<_>>();
        let bdd = set.as_bdd();
        let bdd = bdd.exists(&vars);

        let ctx = self_ref.as_ref().symbolic_context();
        let set = GraphColors::new(bdd, ctx.get().as_native());
        ColorSet::mk_native(self_ref.as_ref().symbolic_context(), set)
    }

    pub fn mk_perturbable_color_vartex_set(
        _self: &Bound<'_, AsynchronousPerturbationGraph>,
        set: &GraphColoredVertices,
    ) -> ColoredVertexSet {
        let self_ref = _self.borrow();
        let perturbable = _self.get().as_native().perturbable_variables();
        let vars = _self
            .get()
            .as_native()
            .get_perturbation_bdd_mapping(perturbable)
            .into_values()
            .collect::<Vec<_>>();
        let bdd = set.as_bdd();
        let bdd = bdd.exists(&vars);

        let ctx = self_ref.as_ref().symbolic_context();
        let set = GraphColoredVertices::new(bdd, ctx.get().as_native());
        ColoredVertexSet::mk_native(self_ref.as_ref().symbolic_context(), set)
    }

    /// Returns a list of perturbed variables together with their values, or error if the
    /// variables are invalid (e.g. not perturbable). If a variable is not present, it is not
    /// returned. It is up to the caller to interpret this correctly.
    pub fn resolve_perturbation(
        _self: &PyRef<'_, AsynchronousPerturbationGraph>,
        value: &Bound<'_, PyAny>,
    ) -> PyResult<HashMap<biodivine_lib_param_bn::VariableId, Option<bool>>> {
        let parent_ref = _self.as_ref();
        let perturbable: HashSet<biodivine_lib_param_bn::VariableId> =
            HashSet::from_iter(_self.as_native().perturbable_variables().clone());
        let mut result = HashMap::new();
        if let Ok(dict) = value.downcast::<PyDict>() {
            for (k, v) in dict {
                let k_var = parent_ref.resolve_network_variable(&k)?;

                if !perturbable.contains(&k_var) {
                    return throw_runtime_error(format!("Variable {k_var} cannot be perturbed."));
                };

                let val = v.extract::<Option<bool>>()?;
                result.insert(k_var, val);
            }
        } else if let Ok(model) = value.downcast::<PerturbationModel>() {
            for (k, v) in model.get().items() {
                let k_var: biodivine_lib_param_bn::VariableId = k.into();

                if !perturbable.contains(&k_var) {
                    return throw_runtime_error(format!("Variable {k_var} cannot be perturbed."));
                };

                result.insert(k_var, v);
            }
        } else {
            return throw_type_error("Expected a dictionary of `VariableIdType` keys and `BoolType | None` values, or a `PerturbationModel`.");
        }
        Ok(result)
    }
}
