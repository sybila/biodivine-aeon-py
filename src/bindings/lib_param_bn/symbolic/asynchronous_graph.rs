use crate::bindings::lib_bdd::bdd::Bdd;
use crate::bindings::lib_bdd::boolean_expression::BooleanExpression;
use crate::bindings::lib_param_bn::boolean_network::BooleanNetwork;
use crate::bindings::lib_param_bn::symbolic::set_color::ColorSet;
use crate::bindings::lib_param_bn::symbolic::set_colored_vertex::ColoredVertexSet;
use crate::bindings::lib_param_bn::symbolic::set_vertex::VertexSet;
use crate::bindings::lib_param_bn::symbolic::symbolic_context::SymbolicContext;

use crate::bindings::lib_hctl_model_checker::hctl_formula::HctlFormula;
use crate::bindings::lib_param_bn::NetworkVariableContext;
use crate::bindings::lib_param_bn::symbolic::model_vertex::VertexModel;
use crate::bindings::lib_param_bn::variable_id::VariableId;
use crate::pyo3_utils::BoolLikeValue;
use crate::{AsNative, runtime_error, throw_runtime_error, throw_type_error};
use biodivine_hctl_model_checker::mc_utils::get_extended_symbolic_graph;
use biodivine_lib_bdd::BddValuation;
use biodivine_lib_bdd::boolean_expression::BooleanExpression as RsBooleanExpression;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColors, SymbolicAsyncGraph};
use either::{Left, Right};
use pyo3::IntoPyObjectExt;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::collections::HashMap;

#[pyclass(module = "biodivine_aeon", frozen, subclass)]
pub struct AsynchronousGraph {
    ctx: Py<SymbolicContext>,
    native: SymbolicAsyncGraph,
}

impl NetworkVariableContext for AsynchronousGraph {
    fn resolve_network_variable(
        &self,
        variable: &Bound<'_, PyAny>,
    ) -> PyResult<biodivine_lib_param_bn::VariableId> {
        NetworkVariableContext::resolve_network_variable(self.ctx.get(), variable)
    }

    fn get_network_variable_name(&self, variable: biodivine_lib_param_bn::VariableId) -> String {
        NetworkVariableContext::get_network_variable_name(self.ctx.get(), variable)
    }
}

impl AsNative<SymbolicAsyncGraph> for AsynchronousGraph {
    fn as_native(&self) -> &SymbolicAsyncGraph {
        &self.native
    }

    fn as_native_mut(&mut self) -> &mut SymbolicAsyncGraph {
        &mut self.native
    }
}

#[pymethods]
impl AsynchronousGraph {
    /// A new `AsynchronousGraph` is constructed from a `BooleanNetwork`. Optionally, you can also provide
    /// a `SymbolicContext` (that is compatible with said network), or a `unit_bdd` which restricts the set
    /// of vertices and colors of the `AsynchronousGraph`.
    ///
    /// Note that the graph structure is immutable: if you change the original network, you have to create
    /// a new `AsynchronousGraph`.
    #[new]
    #[pyo3(signature=(network, context = None, unit_bdd = None))]
    pub fn new(
        py: Python,
        network: Py<BooleanNetwork>,
        context: Option<Py<SymbolicContext>>,
        unit_bdd: Option<Bdd>,
    ) -> PyResult<Self> {
        let ctx = match context {
            Some(ctx) => ctx,
            None => Py::new(py, SymbolicContext::new(py, network.clone(), None)?)?,
        };
        let unit_bdd = match unit_bdd {
            Some(bdd) => bdd.as_native().clone(),
            None => ctx.borrow(py).as_native().mk_constant(true),
        };

        let network_ref = network.borrow(py);
        let context_clone = ctx.get().as_native().clone();
        let native = SymbolicAsyncGraph::with_custom_context(
            network_ref.as_native(),
            context_clone,
            unit_bdd,
        );
        match native {
            Ok(native) => Ok(AsynchronousGraph { ctx, native }),
            Err(e) => throw_runtime_error(e),
        }
    }

    /// Create a new `AsynchronousGraph` whose `SymbolicContext` contains enough additional
    /// symbolic variables to model-check the `requirement` HCTL formula (or explicit
    /// quantified variable count).
    ///
    /// This uses the "extra" variables supported by the `SymbolicContext` object and hence
    /// cannot be used with other features that also rely on "extra" variables
    /// (like `SymbolicSpaceContext`).
    ///
    /// Note that the symbolic representation in the resulting `AsynchronousGraph` likely
    /// won't be compatible with the "default" `AsynchronousGraph` either. However, you can
    /// translate between these representations using `AsynchronousGraph.transfer_from`.
    #[staticmethod]
    pub fn mk_for_model_checking(
        py: Python,
        network: &BooleanNetwork,
        requirement: &Bound<'_, PyAny>,
    ) -> PyResult<Self> {
        let var_count = if let Ok(count) = requirement.extract::<usize>() {
            count
        } else if let Ok(formula) = requirement.extract::<HctlFormula>() {
            let vars = formula.used_state_variables();
            vars.len()
        } else if let Ok(_formula_str) = requirement.extract::<String>() {
            // We don't use the string because the constructor already does all the error handling
            // for the parsing step for us. We just need to check if the argument is a string or not.
            let formula = HctlFormula::new(requirement, true, None)?;
            let vars = formula.used_state_variables();
            vars.len()
        } else {
            return throw_type_error("Expected `int`, `str`, or `HctlFormula`.");
        };
        let var_count = match u16::try_from(var_count) {
            Ok(count) => count,
            Err(_) => return throw_runtime_error("Cannot represent more than 2^16 variables."),
        };

        match get_extended_symbolic_graph(network.as_native(), var_count) {
            Ok(graph) => {
                let py_ctx = Py::new(
                    py,
                    SymbolicContext::wrap_native(py, graph.symbolic_context().clone())?,
                )?;
                Ok(AsynchronousGraph {
                    ctx: py_ctx,
                    native: graph,
                })
            }
            Err(problem) => throw_runtime_error(problem),
        }
    }

    pub fn __str__(&self, py: Python) -> String {
        format!("AsynchronousGraph({})", self.ctx.borrow(py).__str__())
    }

    pub fn __copy__(self_: Py<AsynchronousGraph>) -> Py<AsynchronousGraph> {
        self_.clone()
    }

    pub fn __deepcopy__(
        self_: Py<AsynchronousGraph>,
        _memo: &Bound<'_, PyAny>,
    ) -> Py<AsynchronousGraph> {
        self_.clone()
    }

    /// The underlying `SymbolicContext` of this graph.
    pub fn symbolic_context(&self) -> Py<SymbolicContext> {
        self.ctx.clone()
    }

    /// The number of the network variables (or state variables).
    pub fn network_variable_count(&self) -> usize {
        self.ctx.get().network_variable_count()
    }

    /// The names of the network variables.
    pub fn network_variable_names(&self) -> Vec<String> {
        self.ctx.get().network_variable_names()
    }

    /// The `VariableId` identifiers of the network variables.
    pub fn network_variables(&self) -> Vec<VariableId> {
        self.ctx.get().network_variables()
    }

    /// Return a `VariableId` of the specified network variable, assuming such variable exists.
    pub fn find_network_variable(
        &self,
        variable: &Bound<'_, PyAny>,
    ) -> PyResult<Option<VariableId>> {
        self.ctx.get().find_network_variable(variable)
    }

    /// The name of a particular network variable.
    pub fn get_network_variable_name(&self, variable: &Bound<'_, PyAny>) -> PyResult<String> {
        self.ctx.get().get_network_variable_name(variable)
    }

    /// Try to reconstruct the underlying `BooleanNetwork` from the symbolic functions of
    /// this `AsynchronousGraph`.
    ///
    /// This is only possible when the graph does not use any non-trivial uninterpreted functions
    /// (i.e. arity more than zero), because there is no suitable way to reconstruct
    /// a function expression form a partially specified function. The only exception are
    /// implicit parameters (i.e. fully erased functions) that can be reconstructed as well.
    pub fn reconstruct_network(&self, py: Python) -> PyResult<Py<BooleanNetwork>> {
        let Some(native) = self.native.reconstruct_network() else {
            return throw_runtime_error("Cannot reconstruct network: complex parameters found.");
        };
        BooleanNetwork::from(native).export_to_python(py)
    }

    /// Return an empty `ColoredVertexSet`.
    pub fn mk_empty_colored_vertices(&self) -> ColoredVertexSet {
        ColoredVertexSet::mk_native(self.ctx.clone(), self.native.mk_empty_colored_vertices())
    }

    /// Return an empty `ColorSet`.
    pub fn mk_empty_colors(&self) -> ColorSet {
        ColorSet::mk_native(self.ctx.clone(), self.native.mk_empty_colors())
    }

    /// Return an empty `VertexSet`.
    pub fn mk_empty_vertices(&self) -> VertexSet {
        VertexSet::mk_native(self.ctx.clone(), self.native.mk_empty_vertices())
    }

    /// Return a "unit" (i.e. full) `ColoredVertexSet`.
    pub fn mk_unit_colored_vertices(&self) -> ColoredVertexSet {
        ColoredVertexSet::mk_native(self.ctx.clone(), self.native.mk_unit_colored_vertices())
    }

    /// Return a "unit" (i.e. full) `ColorSet`.
    pub fn mk_unit_colors(&self) -> ColorSet {
        ColorSet::mk_native(self.ctx.clone(), self.native.mk_unit_colors())
    }

    /// Return a "unit" (i.e. full) `VertexSet`.
    pub fn mk_unit_vertices(&self) -> VertexSet {
        VertexSet::mk_native(self.ctx.clone(), self.native.mk_unit_vertices())
    }

    /// Compute the set of colors where the function of the given `id` outputs the specified
    /// `value` for the given input `row`.
    ///
    /// Note that this does not reflect the static constraints of the underlying network. It simply
    /// outputs all functions where given inputs evaluate to the given output.
    pub fn mk_function_row_colors(
        &self,
        function: &Bound<'_, PyAny>,
        row: &Bound<'_, PyList>,
        value: BoolLikeValue,
    ) -> PyResult<ColorSet> {
        let ctx = self.ctx.get();
        let output = value.bool();
        let table = match ctx.resolve_function(function)? {
            Left(var) => ctx.as_native().get_implicit_function_table(var).unwrap(),
            Right(par) => ctx.as_native().get_explicit_function_table(par),
        };
        let arity = table.arity as usize;
        if row.len() != arity {
            return throw_runtime_error(format!(
                "Expected {} argument(s), but {} found.",
                arity,
                row.len()
            ));
        }

        let mut input = Vec::new();
        for it in row {
            input.push(it.extract::<BoolLikeValue>()?.bool());
        }

        for (i, var) in table {
            if i == input {
                let bdd = ctx.as_native().bdd_variable_set().mk_literal(var, output);
                let native_set = GraphColors::new(bdd, ctx.as_native());
                return Ok(ColorSet::mk_native(self.ctx.clone(), native_set));
            }
        }

        unreachable!("The table does not cover all inputs.");
    }

    /// Compute a set of colors that corresponds to the given function interpretation
    /// (functions other than `id` remain unconstrained).
    ///
    /// Note that the result of this operation does not have to be a subset of
    /// `AsynchronousGraph.mk_unit_colors`. In other words, this method allows you to
    /// also create instances of colors that represent valid functions, but are disallowed
    /// by the regulation constraints.
    ///
    /// The first argument must identify an unknown function (i.e. explicit or implicit parameter).
    /// The second argument then represents a Boolean function of the arity prescribed for the
    /// specified unknown function. Such a Boolean function can be represented as:
    ///  - A `BooleanExpression` (or a string that parses into a `BooleanExpression`) which
    ///    uses only variables `x_0 ... x_{A-1}` (`A` being the function arity).
    ///  - A `Bdd` that only depends on the first `A` symbolic variables.
    ///
    /// In both cases, the support set of the function can be of course a subset of the prescribed
    /// variables (e.g. `x_0 | x_3` is allowed for a function with `A=4`, even though `x_1` and
    /// `x_2` are unused).
    pub fn mk_function_colors(
        &self,
        function: &Bound<'_, PyAny>,
        value: &Bound<'_, PyAny>,
    ) -> PyResult<ColorSet> {
        let ctx = self.ctx.get();
        let table = match ctx.resolve_function(function)? {
            Left(var) => ctx.as_native().get_implicit_function_table(var).unwrap(),
            Right(par) => ctx.as_native().get_explicit_function_table(par),
        };
        let arity = table.arity as usize;

        let mut color_valuation = biodivine_lib_bdd::BddPartialValuation::empty();

        if let Ok(bdd) = value.extract::<Bdd>() {
            // Ensure that the function only depends on the expected variables.
            let support = bdd.as_native().support_set();
            for var in support {
                if var.to_index() >= arity {
                    return throw_runtime_error(format!(
                        "Provided function uses `BddVariable({})`, but expects {} variables.",
                        var.to_index(),
                        arity,
                    ));
                }
            }

            let mut val = BddValuation::new(vec![false; usize::from(bdd.as_native().num_vars())]);
            for (inputs, var) in table {
                for (i, x) in inputs.into_iter().enumerate() {
                    val[biodivine_lib_bdd::BddVariable::from_index(i)] = x;
                }
                color_valuation[var] = Some(bdd.as_native().eval_in(&val));
            }
        } else {
            let expr = BooleanExpression::resolve_expression(value)?;
            let expected_support = (0..arity)
                .map(|it| (format!("x_{it}"), it))
                .collect::<HashMap<_, _>>();

            // A helper function which evaluates while mapping variables to indices 1:1.
            fn eval(
                values: &[bool],
                names: &HashMap<String, usize>,
                e: &RsBooleanExpression,
            ) -> PyResult<bool> {
                match e {
                    RsBooleanExpression::Const(x) => Ok(*x),
                    RsBooleanExpression::Variable(x) => {
                        let index = names.get(x).cloned().ok_or_else(|| {
                            runtime_error(format!(
                                "Provided function uses variable `{}`, but expects {} variables.",
                                x,
                                names.len(),
                            ))
                        })?;
                        Ok(values[index])
                    }
                    RsBooleanExpression::Not(inner) => {
                        eval(values, names, inner.as_ref()).map(|it| !it)
                    }
                    RsBooleanExpression::And(l, r) => {
                        let l = eval(values, names, l.as_ref())?;
                        let r = eval(values, names, r.as_ref())?;

                        Ok(l && r)
                    }
                    RsBooleanExpression::Or(l, r) => {
                        let l = eval(values, names, l.as_ref())?;
                        let r = eval(values, names, r.as_ref())?;

                        Ok(l || r)
                    }
                    RsBooleanExpression::Imp(l, r) => {
                        let l = eval(values, names, l.as_ref())?;
                        let r = eval(values, names, r.as_ref())?;

                        Ok(!l || r)
                    }
                    RsBooleanExpression::Iff(l, r) => {
                        let l = eval(values, names, l.as_ref())?;
                        let r = eval(values, names, r.as_ref())?;

                        Ok(l == r)
                    }
                    RsBooleanExpression::Xor(l, r) => {
                        let l = eval(values, names, l.as_ref())?;
                        let r = eval(values, names, r.as_ref())?;

                        Ok(l != r)
                    }
                    RsBooleanExpression::Cond(test, branch1, branch2) => {
                        if eval(values, names, test.as_ref())? {
                            eval(values, names, branch1.as_ref())
                        } else {
                            eval(values, names, branch2.as_ref())
                        }
                    }
                }
            }

            for (inputs, var) in table {
                let result = eval(&inputs, &expected_support, expr.as_native())?;
                color_valuation[var] = Some(result);
            }
        }

        let bdd = self
            .ctx
            .get()
            .bdd_variable_set()
            .get()
            .as_native()
            .mk_conjunctive_clause(&color_valuation);
        let colors = GraphColors::new(bdd, self.symbolic_context().get().as_native());
        Ok(ColorSet::mk_native(self.symbolic_context().clone(), colors))
    }

    /// Transfer a symbolic set (`ColorSet`, `VertexSet`, or `ColoredVertexSet`) from a compatible `AsynchronousGraph`
    /// into the encoding of this graph.
    pub fn transfer_from(
        &self,
        py: Python,
        set: &Bound<'_, PyAny>,
        original_ctx: &AsynchronousGraph,
    ) -> PyResult<Py<PyAny>> {
        let set = if let Ok(set) = set.extract::<ColorSet>() {
            self.as_native()
                .transfer_colors_from(set.as_native(), original_ctx.as_native())
                .map(|it| ColorSet::mk_native(self.ctx.clone(), it).into_py_any(py))
        } else if let Ok(set) = set.extract::<VertexSet>() {
            self.as_native()
                .transfer_vertices_from(set.as_native(), original_ctx.as_native())
                .map(|it| VertexSet::mk_native(self.ctx.clone(), it).into_py_any(py))
        } else if let Ok(set) = set.extract::<ColoredVertexSet>() {
            self.as_native()
                .transfer_from(set.as_native(), original_ctx.as_native())
                .map(|it| ColoredVertexSet::mk_native(self.ctx.clone(), it).into_py_any(py))
        } else {
            return throw_type_error("Expected `ColorSet`, `VertexSet`, or `ColoredVertexSet`.");
        }
        .transpose()?;
        if let Some(set) = set {
            Ok(set)
        } else {
            throw_runtime_error("The two contexts are not compatible.")
        }
    }

    /// Create a symbolic `ColoredVertexSet` consisting of unit colors and vertices with the specified variables
    /// fixed to their respective values.
    pub fn mk_subspace(&self, subspace: &Bound<'_, PyAny>) -> PyResult<ColoredVertexSet> {
        let valuation = self.resolve_subspace_valuation(subspace)?;
        Ok(ColoredVertexSet::mk_native(
            self.ctx.clone(),
            self.as_native().mk_subspace(&valuation),
        ))
    }

    /// Create a symbolic `VertexSet` of vertices with the specified variables fixed to their respective values.
    pub fn mk_subspace_vertices(&self, subspace: &Bound<'_, PyAny>) -> PyResult<VertexSet> {
        let valuation = self.resolve_subspace_valuation(subspace)?;
        Ok(VertexSet::mk_native(
            self.ctx.clone(),
            self.as_native().mk_subspace(&valuation).vertices(),
        ))
    }

    /// Compute the `Bdd` representation of the update function that is associated with the given `variable`.
    pub fn mk_update_function(&self, variable: &Bound<'_, PyAny>) -> PyResult<Bdd> {
        let variable = self.ctx.get().resolve_network_variable(variable)?;
        let update = self.as_native().get_symbolic_fn_update(variable);
        Ok(Bdd::new_raw_2(
            self.ctx.get().bdd_variable_set(),
            update.clone(),
        ))
    }

    /// Compute the set of direct successors of the given `set`.
    pub fn post(&self, set: &ColoredVertexSet) -> ColoredVertexSet {
        ColoredVertexSet::mk_native(self.ctx.clone(), self.as_native().post(set.as_native()))
    }

    /*
        /// Compute the set of direct successors of the given `set` that are *outside* of the given `set`.
        pub fn post_out(&self, set: &ColoredVertexSet) -> ColoredVertexSet {
            ColoredVertexSet::mk_native(self.ctx.clone(), self.as_native().will_post_out(set.as_native()))
        }

        /// Compute the set of direct successors of the given `set` that are *within* the given `set`.
        pub fn post_within(&self, set: &ColoredVertexSet) -> ColoredVertexSet {
            ColoredVertexSet::mk_native(self.ctx.clone(), self.as_native().will_post_within(set.as_native()))
        }
    */
    /// Compute the set of direct predecessors of the given `set`.
    pub fn pre(&self, set: &ColoredVertexSet) -> ColoredVertexSet {
        ColoredVertexSet::mk_native(self.ctx.clone(), self.as_native().pre(set.as_native()))
    }
    /*
        /// Compute the set of direct predecessors of the given `set` that are *outside* of the given `set`.
        pub fn pre_out(&self, set: &ColoredVertexSet) -> ColoredVertexSet {
            ColoredVertexSet::mk_native(self.ctx.clone(), self.as_native().will_pre_out(set.as_native()))
        }

        /// Compute the set of direct predecessors of the given `set` that are *within* the given `set`.
        pub fn pre_within(&self, set: &ColoredVertexSet) -> ColoredVertexSet {
            ColoredVertexSet::mk_native(self.ctx.clone(), self.as_native().will_pre_within(set.as_native()))
        }
    */
    /// Compute the set of direct successors of the given `set` by updating the specified `var`.
    pub fn var_post(
        &self,
        var: &Bound<'_, PyAny>,
        set: &ColoredVertexSet,
    ) -> PyResult<ColoredVertexSet> {
        let variable = self.ctx.get().resolve_network_variable(var)?;
        Ok(ColoredVertexSet::mk_native(
            self.ctx.clone(),
            self.as_native().var_post(variable, set.as_native()),
        ))
    }

    /// Compute the set of direct successors of the given `set` by updating the specified `var` that are *outside*
    /// of the given `set`.
    pub fn var_post_out(
        &self,
        var: &Bound<'_, PyAny>,
        set: &ColoredVertexSet,
    ) -> PyResult<ColoredVertexSet> {
        let variable = self.ctx.get().resolve_network_variable(var)?;
        Ok(ColoredVertexSet::mk_native(
            self.ctx.clone(),
            self.as_native().var_post_out(variable, set.as_native()),
        ))
    }

    /// Compute the set of direct successors of the given `set` by updating the specified `var` that are *within*
    /// the given `set`.
    pub fn var_post_within(
        &self,
        var: &Bound<'_, PyAny>,
        set: &ColoredVertexSet,
    ) -> PyResult<ColoredVertexSet> {
        let variable = self.ctx.get().resolve_network_variable(var)?;
        Ok(ColoredVertexSet::mk_native(
            self.ctx.clone(),
            self.as_native().var_post_within(variable, set.as_native()),
        ))
    }

    /// Compute the set of direct predecessors of the given `set` by updating the specified `var`.
    pub fn var_pre(
        &self,
        var: &Bound<'_, PyAny>,
        set: &ColoredVertexSet,
    ) -> PyResult<ColoredVertexSet> {
        let variable = self.ctx.get().resolve_network_variable(var)?;
        Ok(ColoredVertexSet::mk_native(
            self.ctx.clone(),
            self.as_native().var_pre(variable, set.as_native()),
        ))
    }

    /// Compute the set of direct predecessors of the given `set` by updating the specified `var` that are *outside*
    /// of the given `set`.
    pub fn var_pre_out(
        &self,
        var: &Bound<'_, PyAny>,
        set: &ColoredVertexSet,
    ) -> PyResult<ColoredVertexSet> {
        let variable = self.ctx.get().resolve_network_variable(var)?;
        Ok(ColoredVertexSet::mk_native(
            self.ctx.clone(),
            self.as_native().var_pre_out(variable, set.as_native()),
        ))
    }

    /// Compute the set of direct predecessors of the given `set` by updating the specified `var` that are *within*
    /// the given `set`.
    pub fn var_pre_within(
        &self,
        var: &Bound<'_, PyAny>,
        set: &ColoredVertexSet,
    ) -> PyResult<ColoredVertexSet> {
        let variable = self.ctx.get().resolve_network_variable(var)?;
        Ok(ColoredVertexSet::mk_native(
            self.ctx.clone(),
            self.as_native().var_pre_within(variable, set.as_native()),
        ))
    }

    /// Compute the subset of the given `set` that has a successor.
    pub fn can_post(&self, set: &ColoredVertexSet) -> ColoredVertexSet {
        ColoredVertexSet::mk_native(self.ctx.clone(), self.as_native().can_post(set.as_native()))
    }

    /*
    /// Compute the subset of the given `set` that has a successor that is *outside* of the given `set`.
    pub fn can_post_out(&self, set: &ColoredVertexSet) -> ColoredVertexSet {
        ColoredVertexSet::mk_native(self.ctx.clone(), self.as_native().can_post_out(set.as_native()))
    }

    /// Compute the subset of the given `set` that has a successor that is *within* the given `set`.
    pub fn can_post_within(&self, set: &ColoredVertexSet) -> ColoredVertexSet {
        ColoredVertexSet::mk_native(self.ctx.clone(), self.as_native().can_post_within(set.as_native()))
    }*/

    /// Compute the subset of the given `set` that has a predecessor.
    pub fn can_pre(&self, set: &ColoredVertexSet) -> ColoredVertexSet {
        ColoredVertexSet::mk_native(self.ctx.clone(), self.as_native().can_pre(set.as_native()))
    }

    /*
    /// Compute the subset of the given `set` that has a predecessor that is *outside* of the given `set`.
    pub fn can_pre_out(&self, set: &ColoredVertexSet) -> ColoredVertexSet {
        ColoredVertexSet::mk_native(self.ctx.clone(), self.as_native().can_pre_out(set.as_native()))
    }

    /// Compute the subset of the given `set` that has a predecessor that is *within* the given `set`.
    pub fn can_pre_within(&self, set: &ColoredVertexSet) -> ColoredVertexSet {
        ColoredVertexSet::mk_native(self.ctx.clone(), self.as_native().can_pre_within(set.as_native()))
    }
    */

    /// Compute the subset of the given `set` that has a successor by updating the variable `var`.
    pub fn var_can_post(
        &self,
        var: &Bound<'_, PyAny>,
        set: &ColoredVertexSet,
    ) -> PyResult<ColoredVertexSet> {
        let variable = self.ctx.get().resolve_network_variable(var)?;
        Ok(ColoredVertexSet::mk_native(
            self.ctx.clone(),
            self.as_native().var_can_post(variable, set.as_native()),
        ))
    }

    /// Compute the subset of the given `set` that has a successor by updating the variable `var` that is *outside*
    /// of the given `set`.
    pub fn var_can_post_out(
        &self,
        var: &Bound<'_, PyAny>,
        set: &ColoredVertexSet,
    ) -> PyResult<ColoredVertexSet> {
        let variable = self.ctx.get().resolve_network_variable(var)?;
        Ok(ColoredVertexSet::mk_native(
            self.ctx.clone(),
            self.as_native().var_can_post_out(variable, set.as_native()),
        ))
    }

    /// Compute the subset of the given `set` that has a successor by updating the variable `var` that is *within*
    /// the given `set`.
    pub fn var_can_post_within(
        &self,
        var: &Bound<'_, PyAny>,
        set: &ColoredVertexSet,
    ) -> PyResult<ColoredVertexSet> {
        let variable = self.ctx.get().resolve_network_variable(var)?;
        Ok(ColoredVertexSet::mk_native(
            self.ctx.clone(),
            self.as_native()
                .var_can_post_within(variable, set.as_native()),
        ))
    }

    /// Compute the subset of the given `set` that has a predecessor by updating the variable `var`.
    pub fn var_can_pre(
        &self,
        var: &Bound<'_, PyAny>,
        set: &ColoredVertexSet,
    ) -> PyResult<ColoredVertexSet> {
        let variable = self.ctx.get().resolve_network_variable(var)?;
        Ok(ColoredVertexSet::mk_native(
            self.ctx.clone(),
            self.as_native().var_can_pre(variable, set.as_native()),
        ))
    }

    /// Compute the subset of the given `set` that has a predecessor by updating the variable `var` that is *outside*
    /// of the given `set`.
    pub fn var_can_pre_out(
        &self,
        var: &Bound<'_, PyAny>,
        set: &ColoredVertexSet,
    ) -> PyResult<ColoredVertexSet> {
        let variable = self.ctx.get().resolve_network_variable(var)?;
        Ok(ColoredVertexSet::mk_native(
            self.ctx.clone(),
            self.as_native().var_can_pre_out(variable, set.as_native()),
        ))
    }

    /// Compute the subset of the given `set` that has a predecessor by updating the variable `var` that is *within*
    /// the given `set`.
    pub fn var_can_pre_within(
        &self,
        var: &Bound<'_, PyAny>,
        set: &ColoredVertexSet,
    ) -> PyResult<ColoredVertexSet> {
        let variable = self.ctx.get().resolve_network_variable(var)?;
        Ok(ColoredVertexSet::mk_native(
            self.ctx.clone(),
            self.as_native()
                .var_can_pre_within(variable, set.as_native()),
        ))
    }

    /// Compute a version of the `AsynchronousGraph` where the specified variable is inlined.
    ///
    /// This is similar to `BooleanNetwork.inline_variable`, but performs the operation
    /// symbolically on the `Bdd` update functions, not syntactically on the `UpdateFunction`
    /// of the `BooleanNetwork`.
    ///
    /// As such, the result is often much smaller than the syntactic inlining. However, the
    /// process does not produce an actual `BooleanNetwork` and is slightly unusual in the
    /// context of `biodivine_aeon`: The new system uses new `VariableId` identifiers,
    /// shifted due to the removed network variable. However, to maintain compatibility
    /// with the original `AsynchronousGraph`, it uses the same underlying `BddVariableSet`.
    ///
    /// In other words, the variable is inlined inside the `AsynchronousGraph`, but still
    /// (theoretically) exists in the symbolic representation, it is just eliminated everywhere,
    /// including the `SymbolicContext` of the `AsynchronousGraph`.
    ///
    /// *Currently, variables with a self-regulation cannot be inlined (raised a `RuntimeError`).*
    pub fn inline_variable_symbolic(
        &self,
        py: Python,
        variable: &Bound<'_, PyAny>,
    ) -> PyResult<AsynchronousGraph> {
        let variable = self.resolve_network_variable(variable)?;
        let Some(native_reduced) = self.native.inline_symbolic(variable) else {
            let name = self.as_native().get_variable_name(variable);
            return throw_runtime_error(format!(
                "Cannot inline `{name}`. Self-regulation detected."
            ));
        };

        let native_ctx = native_reduced.symbolic_context().clone();
        let py_ctx = SymbolicContext::wrap_native(py, native_ctx)?;
        let py_ctx = Py::new(py, py_ctx)?;
        Ok(AsynchronousGraph {
            native: native_reduced,
            ctx: py_ctx,
        })
    }

    /// Create a copy of this graph that is restricted only to the given set of values.
    ///
    /// This restriction can be a `ColoredVertexSet` (affects both colors and vertices),
    /// a `ColorSet`, or a `VertexSet` (only affects colors or vertices, respectively).
    ///
    /// Note that only the intersection with the existing unit set is considered in the result
    /// (i.e. this method cannot be used to *add* new values to the graph).
    ///
    /// Also note that this not only restricts vertices/colors, but also the edges of the graph.
    pub fn restrict(&self, vertices: &Bound<'_, PyAny>, py: Python) -> PyResult<AsynchronousGraph> {
        let native_result = if let Ok(vertices) = vertices.extract::<ColoredVertexSet>() {
            self.as_native().restrict(vertices.as_native())
        } else if let Ok(vertices) = vertices.extract::<VertexSet>() {
            let native_set = vertices.as_native();
            let colored = self
                .as_native()
                .unit_colored_vertices()
                .intersect_vertices(native_set);
            self.as_native().restrict(&colored)
        } else if let Ok(colors) = vertices.extract::<ColorSet>() {
            let native_set = colors.as_native();
            let colored = self
                .as_native()
                .unit_colored_vertices()
                .intersect_colors(native_set);
            self.as_native().restrict(&colored)
        } else {
            return throw_type_error("Expected `VertexSet`, `ColorSet`, or `ColoredVertexSet`.");
        };

        AsynchronousGraph::wrap_native(py, native_result)
    }

    /// Compute the logically unique subset of the given color set.
    /// This method returns a subset of colors that are logically unique
    /// within the context of this asynchronous graph.
    pub fn logically_unique_colors(&self, colors: &ColorSet) -> ColorSet {
        let result = self.as_native().logically_unique_subset(colors.as_native());
        ColorSet::mk_native(self.ctx.clone(), result)
    }
}

impl AsynchronousGraph {
    pub fn resolve_subspace_valuation(
        &self,
        subspace: &Bound<'_, PyAny>,
    ) -> PyResult<Vec<(biodivine_lib_param_bn::VariableId, bool)>> {
        let mut result = Vec::new();
        if let Ok(dict) = subspace.downcast::<PyDict>() {
            for (k, v) in dict {
                let k = self.ctx.get().resolve_network_variable(&k)?;
                let v = v.extract::<BoolLikeValue>()?;
                result.push((k, v.bool()));
            }
            return Ok(result);
        } else if let Ok(model) = subspace.downcast::<VertexModel>() {
            return Ok(model
                .get()
                .items()
                .into_iter()
                .map(|(a, b)| (a.into(), b))
                .collect());
        }
        throw_type_error(
            "Expected a dictionary of `VariableIdType` keys and `BoolType` values or a `VertexModel`.",
        )
    }

    pub fn wrap_native(py: Python, stg: SymbolicAsyncGraph) -> PyResult<AsynchronousGraph> {
        let ctx = Py::new(
            py,
            SymbolicContext::wrap_native(py, stg.symbolic_context().clone())?,
        )?;
        Ok(AsynchronousGraph { ctx, native: stg })
    }
}
