use crate::bindings::lib_bdd::bdd::Bdd;
use crate::bindings::lib_bdd::bdd_variable::BddVariable;
use crate::bindings::lib_param_bn::boolean_network::BooleanNetwork;
use crate::bindings::lib_param_bn::symbolic::asynchronous_graph::AsynchronousGraph;
use crate::bindings::lib_param_bn::symbolic::set_colored_space::ColoredSpaceSet;
use crate::bindings::lib_param_bn::symbolic::set_spaces::SpaceSet;
use crate::bindings::lib_param_bn::symbolic::symbolic_context::SymbolicContext;
use crate::pyo3_utils::{resolve_boolean, richcmp_eq_by_key};
use crate::{throw_type_error, AsNative};
use biodivine_lib_param_bn::symbolic_async_graph::GraphColors;
use biodivine_lib_param_bn::trap_spaces::{NetworkColoredSpaces, NetworkSpaces};
use biodivine_lib_param_bn::{ExtendedBoolean, Space};
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::types::PyDict;

/// An extension of the `SymbolicContext` which supports symbolic representation of network
/// sub-spaces.
///
/// To implement this, `SymbolicSpaceContext` uses the "extra variables" feature of the
/// standard `SymbolicContext`. On its own, `SymbolicSpaceContext` currently does not allow
/// having any extra variables aside from those used for space representation. However, nothing
/// fundamentally prevents us from including additional variables in the future.
#[pyclass(module="biodivine_aeon", extends=SymbolicContext, frozen)]
pub struct SymbolicSpaceContext(biodivine_lib_param_bn::trap_spaces::SymbolicSpaceContext);

impl AsNative<biodivine_lib_param_bn::trap_spaces::SymbolicSpaceContext> for SymbolicSpaceContext {
    fn as_native(&self) -> &biodivine_lib_param_bn::trap_spaces::SymbolicSpaceContext {
        &self.0
    }

    fn as_native_mut(&mut self) -> &mut biodivine_lib_param_bn::trap_spaces::SymbolicSpaceContext {
        &mut self.0
    }
}

#[pymethods]
impl SymbolicSpaceContext {
    /// A `SymbolicSpaceContext` is created from a `BooleanNetwork`, just as a regular
    /// `SymbolicContext`. However, no extra symbolic variables can be specified in this case.
    #[new]
    pub fn new(
        py: Python,
        network: &BooleanNetwork,
    ) -> PyResult<(SymbolicSpaceContext, SymbolicContext)> {
        let ctx =
            biodivine_lib_param_bn::trap_spaces::SymbolicSpaceContext::new(network.as_native());
        let inner = SymbolicContext::wrap_native(py, ctx.inner_context().clone())?;
        Ok((SymbolicSpaceContext(ctx), inner))
    }

    fn __richcmp__(&self, py: Python, other: &Self, op: CompareOp) -> Py<PyAny> {
        richcmp_eq_by_key(py, op, &self, &other, |x| x.as_native().inner_context())
    }

    pub fn __str__(self_: PyRef<SymbolicSpaceContext>) -> String {
        format!(
            "SymbolicSpaceContext(network_variables={}, space_variables={}, explicit_functions={}, implicit_functions={})",
            self_.as_ref().network_variable_count(),
            self_.as_ref().extra_bdd_variable_count(),
            self_.as_ref().explicit_function_count(),
            self_.as_ref().implicit_function_count(),
        )
    }

    fn __copy__(self_: Py<SymbolicSpaceContext>) -> Py<SymbolicSpaceContext> {
        self_.clone()
    }

    /* TODO: Enable later.
    fn __deepcopy__(self_: PyRef<SymbolicSpaceContext>, _memo: &PyAny) -> (SymbolicSpaceContext, SymbolicContext) {
        (self_.clone(), self_.as_ref().clone())
    }*/

    /// The symbolic variable that encodes the fact that a specified `network_variable` can have value `True`
    /// in a particular subspace.
    pub fn get_positive_space_variable(
        self_: PyRef<SymbolicSpaceContext>,
        network_variable: &PyAny,
    ) -> PyResult<BddVariable> {
        let var = self_.as_ref().resolve_network_variable(network_variable)?;
        let var = self_.as_native().get_positive_variable(var);
        Ok(BddVariable::from(var))
    }

    /// The symbolic variable that encodes the fact that a specified `network_variable` can have value `False`
    /// in a particular subspace.
    pub fn get_negative_space_variable(
        self_: PyRef<SymbolicSpaceContext>,
        network_variable: &PyAny,
    ) -> PyResult<BddVariable> {
        let var = self_.as_ref().resolve_network_variable(network_variable)?;
        let var = self_.as_native().get_negative_variable(var);
        Ok(BddVariable::from(var))
    }

    /// Compute a `Bdd` which encodes all spaces in which the value of `function` can be
    /// `true` for some state. We assume that `function` can depend on state variables and
    /// parameter variables, but not on the dual variables used for space encoding.
    ///
    /// In other words, a space `S` satisfies the result `Bdd` if and only if there exists
    /// a state `x \in S` such that `function(x) = true`.
    ///
    /// To compute this, we evaluate the following (informal) expression:
    ///     `exists s_1...s_n: [(s_i => s_i_T) & (!s_i => s_i_F)] & function(s_1, ..., s_n)`
    ///
    /// **WARNING:** The resulting BDD also allows invalid space encodings, mostly because
    /// these are to some extent still interesting in some applications. You'll need to
    /// intersect it with `SymbolicSpaceContext.mk_unit_bdd`.
    pub fn mk_can_go_to_true(&self, function: &Bdd) -> Bdd {
        Bdd::new_raw_2(
            function.__ctx__(),
            self.as_native().mk_can_go_to_true(function.as_native()),
        )
    }

    /// Compute an empty colored subspace relation.
    pub fn mk_empty_colored_spaces(self_: Py<SymbolicSpaceContext>) -> ColoredSpaceSet {
        let set = self_.get().as_native().mk_empty_colored_spaces();
        ColoredSpaceSet::wrap_native(self_.clone(), set)
    }

    /// Compute an empty set of network subspaces.
    pub fn mk_empty_spaces(self_: Py<SymbolicSpaceContext>) -> SpaceSet {
        let set = self_.get().as_native().mk_empty_spaces();
        SpaceSet::wrap_native(self_.clone(), set)
    }

    /// Compute the colored set of all network sub-spaces.
    ///
    /// Note that `SymbolicSpaceContext` has no notion of "unit colors". By default, the final relation contains
    /// all colors. If you want to restrict the relation in the same manner as
    /// `AsynchronousGraph.mk_unit_colored_vertices`, you have to provide an optional `AsynchronousGraph`.
    #[pyo3(signature = (graph = None))]
    pub fn mk_unit_colored_spaces(
        self_: Py<SymbolicSpaceContext>,
        graph: Option<&AsynchronousGraph>,
    ) -> ColoredSpaceSet {
        let ctx = self_.get();
        let unit_colors = if let Some(graph) = graph {
            graph.as_native().mk_unit_colors()
        } else {
            GraphColors::new(
                ctx.as_native().bdd_variable_set().mk_true(),
                ctx.as_native().inner_context(),
            )
        };
        let unit_spaces = ctx.as_native().mk_unit_spaces();
        let unit = unit_colors.as_bdd().and(unit_spaces.as_bdd());
        ColoredSpaceSet::wrap_native(
            self_.clone(),
            NetworkColoredSpaces::new(unit, ctx.as_native()),
        )
    }

    /// Compute the set of all network sub-spaces.
    ///
    /// Note that this is different from a `Bdd` function `True` because not every valuation of the dual variables
    /// encodes a valid network space.
    pub fn mk_unit_spaces(self_: Py<SymbolicSpaceContext>) -> SpaceSet {
        let set = self_.get().as_native().mk_unit_spaces();
        SpaceSet::wrap_native(self_.clone(), set)
    }

    /// Compute the `Bdd` which contains all correctly encoded spaces tracked by this `SymbolicSpaceContext`.
    ///
    /// The `Bdd` only constraints the space variables and it has no impact on the network/parameter
    /// variables. Also note that this is not the `true` function, since not every valuation of space variables
    /// correctly encodes a space.
    pub fn mk_unit_bdd(self_: Py<SymbolicSpaceContext>, py: Python) -> Bdd {
        let ctx = self_.borrow(py);
        let bdd = ctx.as_native().mk_unit_bdd();
        Bdd::new_raw_2(ctx.as_ref().bdd_variable_set(), bdd)
    }

    /// Extend the given `set` with all the sub-spaces for every element of the set.
    ///
    /// For colored sets, this extension is happening color-wise, so new sub-spaces are added with the same color
    /// as their parent space.
    pub fn mk_sub_spaces(
        self_: Py<SymbolicSpaceContext>,
        set: &PyAny,
        py: Python,
    ) -> PyResult<PyObject> {
        let ctx = self_.get();
        if let Ok(set) = set.extract::<ColoredSpaceSet>() {
            let bdd = ctx.as_native().mk_sub_spaces(set.as_native().as_bdd());
            let set = NetworkColoredSpaces::new(bdd, ctx.as_native());
            return Ok(ColoredSpaceSet::wrap_native(self_.clone(), set).into_py(py));
        }
        if let Ok(set) = set.extract::<SpaceSet>() {
            let bdd = ctx.as_native().mk_sub_spaces(set.as_native().as_bdd());
            let set = NetworkSpaces::new(bdd, ctx.as_native());
            return Ok(SpaceSet::wrap_native(self_.clone(), set).into_py(py));
        }
        if let Ok(bdd) = set.extract::<Bdd>() {
            let bdd = ctx.as_native().mk_sub_spaces(bdd.as_native());
            let bdd = Bdd::new_raw_2(self_.borrow(py).as_ref().bdd_variable_set(), bdd);
            return Ok(bdd.into_py(py));
        }
        throw_type_error("Expected `ColoredSpaceSet`, `SpaceSet`, or `Bdd`.")
    }

    /// Extend the given `set` with all the sub-spaces for every element of the set.
    ///
    /// For colored sets, this extension is happening color-wise, so new sub-spaces are added with the same color
    /// as their parent space.
    pub fn mk_super_spaces(
        self_: Py<SymbolicSpaceContext>,
        set: &PyAny,
        py: Python,
    ) -> PyResult<PyObject> {
        let ctx = self_.get();
        if let Ok(set) = set.extract::<ColoredSpaceSet>() {
            let bdd = ctx.as_native().mk_super_spaces(set.as_native().as_bdd());
            let set = NetworkColoredSpaces::new(bdd, ctx.as_native());
            return Ok(ColoredSpaceSet::wrap_native(self_.clone(), set).into_py(py));
        }
        if let Ok(set) = set.extract::<SpaceSet>() {
            let bdd = ctx.as_native().mk_super_spaces(set.as_native().as_bdd());
            let set = NetworkSpaces::new(bdd, ctx.as_native());
            return Ok(SpaceSet::wrap_native(self_.clone(), set).into_py(py));
        }
        if let Ok(bdd) = set.extract::<Bdd>() {
            let bdd = ctx.as_native().mk_super_spaces(bdd.as_native());
            let bdd = Bdd::new_raw_2(self_.borrow(py).as_ref().bdd_variable_set(), bdd);
            return Ok(bdd.into_py(py));
        }
        throw_type_error("Expected `ColoredSpaceSet`, `SpaceSet`, or `Bdd`.")
    }

    /// Compute the `SpaceSet` that represents a single network subspace.
    ///
    /// See also `AsynchronousGraph.mk_subspace`.
    pub fn mk_singleton(
        self_: Py<SymbolicSpaceContext>,
        space: &PyAny,
        py: Python,
    ) -> PyResult<SpaceSet> {
        let network_valuation =
            SymbolicSpaceContext::resolve_subspace_valuation(self_.clone(), space, py)?;
        let mut space = Space::new_raw(
            self_
                .get()
                .as_native()
                .inner_context()
                .num_state_variables(),
        );
        for (var, value) in network_valuation {
            space[var] = ExtendedBoolean::from(value)
        }
        let bdd = self_.get().as_native().mk_space(&space);
        let set = NetworkSpaces::new(bdd, self_.get().as_native());
        Ok(SpaceSet::wrap_native(self_.clone(), set))
    }
}

impl SymbolicSpaceContext {
    pub fn resolve_subspace_valuation(
        self_: Py<SymbolicSpaceContext>,
        subspace: &PyAny,
        py: Python,
    ) -> PyResult<Vec<(biodivine_lib_param_bn::VariableId, bool)>> {
        let mut result = Vec::new();
        if let Ok(dict) = subspace.downcast::<PyDict>() {
            for (k, v) in dict {
                let k = self_.borrow(py).as_ref().resolve_network_variable(k)?;
                let v = resolve_boolean(v)?;
                result.push((k, v));
            }
            return Ok(result);
        }
        throw_type_error("Expected a dictionary of VariableIdType keys and BoolType values.")
    }
}
