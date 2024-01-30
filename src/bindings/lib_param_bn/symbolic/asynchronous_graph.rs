use crate::bindings::lib_bdd::bdd::Bdd;
use crate::bindings::lib_param_bn::boolean_network::BooleanNetwork;
use crate::bindings::lib_param_bn::symbolic::set_color::ColorSet;
use crate::bindings::lib_param_bn::symbolic::set_colored_vertex::ColoredVertexSet;
use crate::bindings::lib_param_bn::symbolic::set_vertex::VertexSet;
use crate::bindings::lib_param_bn::symbolic::symbolic_context::SymbolicContext;
use crate::bindings::lib_param_bn::variable_id::VariableId;
use crate::pyo3_utils::resolve_boolean;
use crate::{throw_runtime_error, throw_type_error, AsNative};
use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
use pyo3::prelude::*;
use pyo3::types::PyDict;

#[pyclass(module = "biodivine_aeon", frozen)]
pub struct AsynchronousGraph {
    ctx: Py<SymbolicContext>,
    native: SymbolicAsyncGraph,
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

    pub fn __str__(&self, py: Python) -> String {
        format!("AsynchronousGraph({})", self.ctx.borrow(py).__str__())
    }

    pub fn __copy__(self_: Py<AsynchronousGraph>) -> Py<AsynchronousGraph> {
        self_.clone()
    }

    pub fn __deepcopy__(self_: Py<AsynchronousGraph>, _memo: &PyAny) -> Py<AsynchronousGraph> {
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
    pub fn find_network_variable(&self, variable: &PyAny) -> PyResult<Option<VariableId>> {
        self.ctx.get().find_network_variable(variable)
    }

    /// The name of a particular network variable.
    pub fn get_network_variable_name(&self, variable: &PyAny) -> PyResult<String> {
        self.ctx.get().get_network_variable_name(variable)
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

    /// Transfer a symbolic set (`ColorSet`, `VertexSet`, or `ColoredVertexSet`) from a compatible `AsynchronousGraph`
    /// into the encoding of this graph.
    pub fn transfer_from(
        &self,
        py: Python,
        set: &PyAny,
        original_ctx: &AsynchronousGraph,
    ) -> PyResult<PyObject> {
        let set = if let Ok(set) = set.extract::<ColorSet>() {
            self.as_native()
                .transfer_colors_from(set.as_native(), original_ctx.as_native())
                .map(|it| ColorSet::mk_native(self.ctx.clone(), it).into_py(py))
        } else if let Ok(set) = set.extract::<VertexSet>() {
            self.as_native()
                .transfer_vertices_from(set.as_native(), original_ctx.as_native())
                .map(|it| VertexSet::mk_native(self.ctx.clone(), it).into_py(py))
        } else if let Ok(set) = set.extract::<ColoredVertexSet>() {
            self.as_native()
                .transfer_from(set.as_native(), original_ctx.as_native())
                .map(|it| ColoredVertexSet::mk_native(self.ctx.clone(), it).into_py(py))
        } else {
            return throw_type_error("Expected `ColorSet`, `VartexSet`, or `ColoredVertexSet`.");
        };
        if let Some(set) = set {
            Ok(set)
        } else {
            throw_runtime_error("The two contexts are not compatible.")
        }
    }

    /// Create a symbolic `ColoredVertexSet` consisting of unit colors and vertices with the specified variables
    /// fixed to their respective values.
    pub fn mk_subspace(&self, subspace: &PyAny) -> PyResult<ColoredVertexSet> {
        let valuation = self.resolve_subspace_valuation(subspace)?;
        Ok(ColoredVertexSet::mk_native(
            self.ctx.clone(),
            self.as_native().mk_subspace(&valuation),
        ))
    }

    /// Create a symbolic `VertexSet` of vertices with the specified variables fixed to their respective values.
    pub fn mk_subspace_vertices(&self, subspace: &PyAny) -> PyResult<VertexSet> {
        let valuation = self.resolve_subspace_valuation(subspace)?;
        Ok(VertexSet::mk_native(
            self.ctx.clone(),
            self.as_native().mk_subspace(&valuation).vertices(),
        ))
    }

    /// Compute the `Bdd` representation of the update function that is associated with the given `variable`.
    pub fn mk_update_function(&self, variable: &PyAny) -> PyResult<Bdd> {
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
    pub fn var_post(&self, var: &PyAny, set: &ColoredVertexSet) -> PyResult<ColoredVertexSet> {
        let variable = self.ctx.get().resolve_network_variable(var)?;
        Ok(ColoredVertexSet::mk_native(
            self.ctx.clone(),
            self.as_native().var_post(variable, set.as_native()),
        ))
    }

    /// Compute the set of direct successors of the given `set` by updating the specified `var` that are *outside*
    /// of the given `set`.
    pub fn var_post_out(&self, var: &PyAny, set: &ColoredVertexSet) -> PyResult<ColoredVertexSet> {
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
        var: &PyAny,
        set: &ColoredVertexSet,
    ) -> PyResult<ColoredVertexSet> {
        let variable = self.ctx.get().resolve_network_variable(var)?;
        Ok(ColoredVertexSet::mk_native(
            self.ctx.clone(),
            self.as_native().var_post_within(variable, set.as_native()),
        ))
    }

    /// Compute the set of direct predecessors of the given `set` by updating the specified `var`.
    pub fn var_pre(&self, var: &PyAny, set: &ColoredVertexSet) -> PyResult<ColoredVertexSet> {
        let variable = self.ctx.get().resolve_network_variable(var)?;
        Ok(ColoredVertexSet::mk_native(
            self.ctx.clone(),
            self.as_native().var_pre(variable, set.as_native()),
        ))
    }

    /// Compute the set of direct predecessors of the given `set` by updating the specified `var` that are *outside*
    /// of the given `set`.
    pub fn var_pre_out(&self, var: &PyAny, set: &ColoredVertexSet) -> PyResult<ColoredVertexSet> {
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
        var: &PyAny,
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
    pub fn var_can_post(&self, var: &PyAny, set: &ColoredVertexSet) -> PyResult<ColoredVertexSet> {
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
        var: &PyAny,
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
        var: &PyAny,
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
    pub fn var_can_pre(&self, var: &PyAny, set: &ColoredVertexSet) -> PyResult<ColoredVertexSet> {
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
        var: &PyAny,
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
        var: &PyAny,
        set: &ColoredVertexSet,
    ) -> PyResult<ColoredVertexSet> {
        let variable = self.ctx.get().resolve_network_variable(var)?;
        Ok(ColoredVertexSet::mk_native(
            self.ctx.clone(),
            self.as_native()
                .var_can_pre_within(variable, set.as_native()),
        ))
    }
}

impl AsynchronousGraph {
    pub fn resolve_subspace_valuation(
        &self,
        subspace: &PyAny,
    ) -> PyResult<Vec<(biodivine_lib_param_bn::VariableId, bool)>> {
        let mut result = Vec::new();
        if let Ok(dict) = subspace.downcast::<PyDict>() {
            for (k, v) in dict {
                let k = self.ctx.get().resolve_network_variable(k)?;
                let v = resolve_boolean(v)?;
                result.push((k, v));
            }
            return Ok(result);
        }
        throw_type_error("Expected a dictionary of `VariableIdType` keys and `BoolType` values.")
    }
}
