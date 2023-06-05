use crate::bindings::lib_bdd::PyBddVariableSet;
use crate::bindings::lib_param_bn::{
    PyBooleanNetwork, PyGraphColoredVertices, PyGraphColors, PyGraphVertices, PyParameterId,
    PySymbolicAsyncGraph, PySymbolicContext, PyVariableId,
};
use crate::{throw_runtime_error, AsNative};
use biodivine_lib_param_bn::biodivine_std::bitvector::{ArrayBitVector, BitVector};
use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
use biodivine_lib_param_bn::VariableId;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::HashMap;

#[pymethods]
impl PySymbolicAsyncGraph {
    /// Create a new `SymbolicAsyncGraph` from a `BooleanNetwork`.
    #[new]
    pub fn new(network: PyBooleanNetwork) -> PyResult<Self> {
        let result = SymbolicAsyncGraph::new(network.into());
        match result {
            Ok(graph) => Ok(graph.into()),
            Err(e) => throw_runtime_error(e),
        }
    }

    /// Obtain a copy of the underlying `BooleanNetwork` used by this `SymbolicAsyncGraph`.
    pub fn network(&self, py: Python) -> PyResult<Py<PyBooleanNetwork>> {
        let network = self.as_native().as_network();
        let network = PyBooleanNetwork::from(network.clone());
        network.export_to_python(py)
    }

    /// Obtain a copy of the `SymbolicContext` used by this symbolic encoding.
    pub fn symbolic_context(&self) -> PySymbolicContext {
        self.as_native().symbolic_context().clone().into()
    }

    /// Obtain a copy of the `BddVariableSet` used during symbolic encoding
    /// in this `SymbolicAsyncGraph`.
    pub fn bdd_variables(&self) -> PyBddVariableSet {
        self.as_native()
            .symbolic_context()
            .bdd_variable_set()
            .clone()
            .into()
    }

    /// Create a `ColoredVertexSet` that contains every color-vertex pair with a specified
    /// Boolean network variable fixed to the given constant.
    ///
    /// The variable can be given either as a `VariableId` object, or as a name.
    pub fn fix_variable(&self, variable: &PyAny, value: bool) -> PyResult<PyGraphColoredVertices> {
        let id = self.resolve_variable(variable)?;
        Ok(self
            .as_native()
            .fix_network_variable(id.into(), value)
            .into())
    }

    /// Create a `VertexSet` that contains every vertex with a specified
    /// Boolean network variable fixed to the given constant.
    ///
    /// The variable can be given either as a `VariableId` object, or as a name.
    pub fn fix_variable_in_vertices(
        &self,
        variable: &PyAny,
        value: bool,
    ) -> PyResult<PyGraphVertices> {
        let id = self.resolve_variable(variable)?;
        Ok(self
            .as_native()
            .fix_vertices_with_network_variable(id.into(), value)
            .into())
    }

    /// Compute the smallest subspace that still contains all vertices in the provided
    /// `VertexSet`.
    pub fn wrap_in_subspace(&self, vertices: &PyGraphVertices) -> HashMap<PyVariableId, bool> {
        let space = self.as_native().wrap_in_subspace(vertices.as_native());
        let mut result = HashMap::new();
        for k in self.as_native().as_network().variables() {
            if let Some(value) = space[k].try_as_bool() {
                result.insert(k.into(), value);
            }
        }
        result
    }

    /// The same as `wrap_in_subspace`, but the input can be any symbolic set, and only the
    /// vertex component is expanded into a (symbolically represented) subspace.
    pub fn wrap_in_symbolic_subspace(
        &self,
        set: &PyGraphColoredVertices,
    ) -> PyGraphColoredVertices {
        self.as_native()
            .wrap_in_symbolic_subspace(set.as_native())
            .into()
    }

    /// Test if the given set is a trap set.
    pub fn is_trap_set(&self, set: &PyGraphColoredVertices) -> bool {
        self.as_native().is_trap_set(set.as_native())
    }
    /*
    /// Create a new graph where the update functions are restricted to the given variable
    /// subspace. Note that this completely eliminates any dependence on this variable in the
    /// remaining update functions, but does not remove the variable from the underlying network
    /// (i.e. the sets of states are still compatible, just the dynamics of the network is altered)
    pub fn restrict_variable_in_graph(
        &self,
        variable: &PyAny,
        value: bool,
    ) -> PyResult<PySymbolicAsyncGraph> {
        let variable = self.network().find_variable(variable)?;
        Ok(self
            .as_native()
            .restrict_variable_in_graph(variable.into(), value)
            .into())
    }*/

    /// Create a `ColoredVertexSet` that contains every color-vertex pair where the vertex
    /// corresponds to the list of Boolean values supplied as argument.
    pub fn fix_vertex(&self, vertex: Vec<bool>) -> PyGraphColoredVertices {
        self.as_native()
            .vertex(&ArrayBitVector::from_bool_vector(vertex))
            .into()
    }

    /// Takes a dictionary which maps variables (name or id) to Boolean values. Produces
    /// a `ColoredVertexSet` of states which agree with the given restriction.
    pub fn fix_subspace(&self, space: &PyDict) -> PyResult<PyGraphColoredVertices> {
        let mut valuation: Vec<(VariableId, bool)> = Vec::new();
        for (k, v) in space {
            let key = self.resolve_variable(k)?;
            let value = v.extract::<bool>()?;
            valuation.push((key.into(), value));
        }
        Ok(self.as_native().mk_subspace(&valuation).into())
    }

    /// Compute the set of all possible colours (instantiations) of this (main) network that
    /// are represented by the supplied more specific sub-network.
    ///
    /// The two networks must be *compatible* for this to work. See the original Rust method
    /// for conditions that must be satisfied by compatible networks.
    pub fn fix_subnetwork_colors(&self, network: &PyBooleanNetwork) -> PyResult<PyGraphColors> {
        match self.as_native().mk_subnetwork_colors(network.as_native()) {
            Ok(colors) => Ok(colors.into()),
            Err(error) => throw_runtime_error(error),
        }
    }

    /// Create a `ColorSet` in which a logical parameter is fixed to the given constant value.
    ///
    /// A logical parameter is any parameter of arity zero, and can be given either as a name,
    /// or as a `ParameterId` object.
    pub fn fix_parameter(&self, parameter: &PyAny, value: bool) -> PyResult<PyGraphColors> {
        // Find parameter and validate.
        let id = self.resolve_parameter(parameter)?;
        let param = self.as_native().as_network().get_parameter(id.into());
        if param.get_arity() != 0 {
            return throw_runtime_error(format!(
                "Parameter {} has non-zero arity.",
                param.get_name()
            ));
        }

        // Make a BDD using symbolic context.
        let ctx = self.as_native().symbolic_context();
        let bdd = ctx.mk_uninterpreted_function_is_true(id.into(), &[]);
        // Apply desired value.
        let bdd = if value { bdd } else { bdd.not() };
        // Export as color set.
        let unit_bdd = self.as_native().unit_colors().as_bdd();
        Ok(self
            .as_native()
            .empty_colors()
            .copy(bdd.and(unit_bdd))
            .into())
    }

    /// Create a `ColorSet` in which a single output of an uninterpreted function is fixed
    /// to the given constant value.
    ///
    /// The uninterpreted function is given either as `ParameterId` or using its name. The output
    /// is identified using a vector of Boolean constants that uniquely identify one row of the
    /// function's truth table (function arity and vector length must match).
    ///
    /// That is, given a function `f` of arity two, input vector `[True, False]` and
    /// a Boolean `X`, the resulting set will only have colours where `f(True, False) = X`
    /// that are otherwise valid (e.g. are not disallowed by monotonicity constraints).
    pub fn fix_explicit_function(
        &self,
        parameter: &PyAny,
        inputs: Vec<bool>,
        value: bool,
    ) -> PyResult<PyGraphColors> {
        let id = self.resolve_parameter(parameter)?;
        let param = self.as_native().as_network().get_parameter(id.into());
        if (param.get_arity() as usize) != inputs.len() {
            return throw_runtime_error(format!(
                "Arity mismatch for parameter {}.",
                param.get_name()
            ));
        }
        let ctx = self.as_native().symbolic_context();
        let table = ctx.get_explicit_function_table(id.into());
        let mut bdd = ctx.mk_constant(false);
        for (row, bdd_var) in table {
            if row == inputs {
                bdd = ctx.bdd_variable_set().mk_literal(bdd_var, value);
            }
        }
        let unit_bdd = self.as_native().unit_colors().as_bdd();
        Ok(self
            .as_native()
            .empty_colors()
            .copy(bdd.and(unit_bdd))
            .into())
    }

    /// Create a `ColorSet` in which a single output of an erased update function is fixed
    /// to the given constant value.
    ///
    /// The erased update function is identified either as `VariableId` or using the name of the
    /// associated variable. The output to be fixed is identified using a vector of Boolean
    /// constants that uniquely identify one row of the function's truth table (function arity
    /// and vector length must match).
    ///
    /// That is, assume a network variable `x` that has an implicit (erased) update function and
    /// two regulators. Now, given function row `[True, False]` and a Boolean `X`, the result
    /// will be a subset of the unit color set where `f_x(True, False) = X`.
    pub fn fix_implicit_function(
        &self,
        variable: &PyAny,
        inputs: Vec<bool>,
        value: bool,
    ) -> PyResult<PyGraphColors> {
        let id = self.resolve_variable(variable)?;
        if self.as_native().as_network().regulators(id.into()).len() != inputs.len() {
            let name = self.as_native().as_network().get_variable_name(id.into());
            return throw_runtime_error(format!("Artiy mismatch for variable {name}."));
        }
        let ctx = self.as_native().symbolic_context();
        let table = ctx.get_implicit_function_table(id.into());
        let mut bdd = ctx.mk_constant(false);
        for (row, bdd_var) in table {
            if row == inputs {
                bdd = ctx.bdd_variable_set().mk_literal(bdd_var, value);
            }
        }
        let unit_bdd = self.as_native().unit_colors().as_bdd();
        Ok(self
            .as_native()
            .empty_colors()
            .copy(bdd.and(unit_bdd))
            .into())
    }

    /// Create a `BooleanNetwork` that matches this graph, but its parameters are fully specified,
    /// and the specification is picked from the provided `ColorSet`.
    pub fn pick_witness(
        &self,
        py: Python,
        colors: &PyGraphColors,
    ) -> PyResult<Py<PyBooleanNetwork>> {
        let network = self.as_native().pick_witness(colors.as_native());
        let network = PyBooleanNetwork::from(network);
        network.export_to_python(py)
    }

    /// Make an empty `ColorSet`.
    pub fn empty_colors(&self) -> PyGraphColors {
        self.as_native().mk_empty_colors().into()
    }

    /// Make an empty `ColoredVertexSet`.
    pub fn empty_colored_vertices(&self) -> PyGraphColoredVertices {
        self.as_native().mk_empty_vertices().into()
    }

    /// Return a `ColorSet` of all colors valid in this graph (i.e. satisfying
    /// structural constraints).
    pub fn unit_colors(&self) -> PyGraphColors {
        self.as_native().mk_unit_colors().into()
    }

    /// Return a `ColoredVertexSet` of all color-vertex pairs valid in this graph (i.e. satisfting
    /// structural constraints).
    pub fn unit_colored_vertices(&self) -> PyGraphColoredVertices {
        self.as_native().mk_unit_colored_vertices().into()
    }

    /// Compute a `ColoredVertexSet` representing all successors of the given `ColoredVertexSet`.
    pub fn post(&self, set: &PyGraphColoredVertices) -> PyGraphColoredVertices {
        self.as_native().post(set.as_native()).into()
    }

    /// Compute a `ColoredVertexSet` representing all predecessors of the given `ColoredVertexSet`.
    pub fn pre(&self, set: &PyGraphColoredVertices) -> PyGraphColoredVertices {
        self.as_native().pre(set.as_native()).into()
    }

    /// Compute a `ColoredVertexSet` representing a subset of the given `ColoredVertexSet`
    /// that can perform some transition (i.e. vertex-color pairs where the vertex has an outgoing
    /// edge of the associated color).
    pub fn can_post(&self, set: &PyGraphColoredVertices) -> PyGraphColoredVertices {
        self.as_native().can_post(set.as_native()).into()
    }

    /// Compute a `ColoredVertexSet` representing a subset of the given `ColoredVertexSet`
    /// that can be reached by some transition (i.e. vertex-color pairs where the vertex has
    /// an incoming edge of the associated color).
    pub fn can_pre(&self, set: &PyGraphColoredVertices) -> PyGraphColoredVertices {
        self.as_native().can_pre(set.as_native()).into()
    }

    /// Compute a `ColoredVertexSet` representing a subset of the given `ColoredVertexSet`
    /// that can perform any transition that stays *within* the same set (i.e. vertex-color pairs
    /// where the vertex has an outgoing edge of the associated color, and that edge leads
    /// to the original set).
    pub fn can_post_within(&self, set: &PyGraphColoredVertices) -> PyGraphColoredVertices {
        self.as_native().can_post_within(set.as_native()).into()
    }

    /// Compute a `ColoredVertexSet` representing a subset of the given `ColoredVertexSet`
    /// that can be reached by a transition originating *within* the same set (i.e. vertex-color
    /// pairs where the vertex has an incoming edge of the associated color, and that edge
    /// originates in the initial set).
    pub fn can_pre_within(&self, set: &PyGraphColoredVertices) -> PyGraphColoredVertices {
        self.as_native().can_pre_within(set.as_native()).into()
    }

    /// Compute a `ColoredVertexSet` representing a subset of the given `ColoredVertexSet`
    /// that can perform any transition that leads *outside* of the set (i.e. vertex-color pairs
    /// where the vertex has an outgoing edge of the associated color, and that edge leads
    /// to the complement of the original set).
    pub fn can_post_out(&self, set: &PyGraphColoredVertices) -> PyGraphColoredVertices {
        self.as_native().can_post_out(set.as_native()).into()
    }

    /// Compute a `ColoredVertexSet` representing a subset of the given `ColoredVertexSet`
    /// that can be reached by a transition originating *outside* of the set (i.e. vertex-color
    /// pairs where the vertex has an incoming edge of the associated color, and that edge
    /// originates in the complement of the initial set).
    pub fn can_pre_out(&self, set: &PyGraphColoredVertices) -> PyGraphColoredVertices {
        self.as_native().can_pre_out(set.as_native()).into()
    }

    /// Compute a `ColoredVertexSet` representing a subset of the given `ColoredVertexSet`
    /// from which any transition stays *within* the original set (i.e. vertex-color pairs
    /// where, from the vertex, all outgoing edges of the associated color lead
    /// to the original set).
    ///
    /// States that have no outgoing transitions are also included.
    pub fn will_post_within(&self, set: &PyGraphColoredVertices) -> PyGraphColoredVertices {
        self.as_native().will_post_within(set.as_native()).into()
    }

    /// Compute a `ColoredVertexSet` representing a subset of the given `ColoredVertexSet`
    /// where all incoming transitions stay *within* the original set (i.e. vertex-color pairs
    /// where, from the vertex, all incoming edges of the associated color start
    /// in the original set).
    ///
    /// States that have no incoming transitions are also included.
    pub fn will_pre_within(&self, set: &PyGraphColoredVertices) -> PyGraphColoredVertices {
        self.as_native().will_pre_within(set.as_native()).into()
    }

    /// Compute a `ColoredVertexSet` representing a subset of the given `ColoredVertexSet`
    /// from which any transition goes *outside* of the original set (i.e. vertex-color pairs
    /// where, from the vertex, all outgoing edges of the associated color lead
    /// to the complement of the original set).
    ///
    /// States that have no outgoing transitions are also included.
    pub fn will_post_out(&self, set: &PyGraphColoredVertices) -> PyGraphColoredVertices {
        self.as_native().will_post_out(set.as_native()).into()
    }

    /// Compute a `ColoredVertexSet` representing a subset of the given `ColoredVertexSet`
    /// where all incoming transitions come from *outside* of the original set (i.e. vertex-color
    /// pairs where, from the vertex, all incoming edges of the associated color start
    /// in the complement of the original set).
    ///
    /// States that have no incoming transitions are also included.
    pub fn will_pre_out(&self, set: &PyGraphColoredVertices) -> PyGraphColoredVertices {
        self.as_native().will_pre_out(set.as_native()).into()
    }

    /// The same as `post`, but only considers transitions of a single variable.
    ///
    /// Variable can be given either as a name, or a `VariableId`.
    pub fn var_post(
        &self,
        variable: &PyAny,
        set: &PyGraphColoredVertices,
    ) -> PyResult<PyGraphColoredVertices> {
        let id = self.resolve_variable(variable)?;
        Ok(self.as_native().var_post(id.into(), set.as_native()).into())
    }

    /// The same as `pre`, but only considers transitions of a single variable.
    ///
    /// Variable can be given either as a name, or a `VariableId`.
    pub fn var_pre(
        &self,
        variable: &PyAny,
        set: &PyGraphColoredVertices,
    ) -> PyResult<PyGraphColoredVertices> {
        let id = self.resolve_variable(variable)?;
        Ok(self.as_native().var_pre(id.into(), set.as_native()).into())
    }

    /// The same as `can_post`, but only considers transitions of a single variable.
    ///
    /// Variable can be given either as a name, or a `VariableId`.
    pub fn var_can_post(
        &self,
        variable: &PyAny,
        set: &PyGraphColoredVertices,
    ) -> PyResult<PyGraphColoredVertices> {
        let id = self.resolve_variable(variable)?;
        Ok(self
            .as_native()
            .var_can_post(id.into(), set.as_native())
            .into())
    }

    /// The same as `can_pre`, but only considers transitions of a single variable.
    ///
    /// Variable can be given either as a name, or a `VariableId`.
    pub fn var_can_pre(
        &self,
        variable: &PyAny,
        set: &PyGraphColoredVertices,
    ) -> PyResult<PyGraphColoredVertices> {
        let id = self.resolve_variable(variable)?;
        Ok(self
            .as_native()
            .var_can_pre(id.into(), set.as_native())
            .into())
    }

    /// The same as `can_pre_within`, but only considers transitions of a single variable.
    ///
    /// Variable can be given either as a name, or a `VariableId`.
    pub fn var_can_pre_within(
        &self,
        variable: &PyAny,
        set: &PyGraphColoredVertices,
    ) -> PyResult<PyGraphColoredVertices> {
        let id = self.resolve_variable(variable)?;
        Ok(self
            .as_native()
            .var_can_pre_within(id.into(), set.as_native())
            .into())
    }

    /// The same as `can_post_within`, but only considers transitions of a single variable.
    ///
    /// Variable can be given either as a name, or a `VariableId`.
    pub fn var_can_post_within(
        &self,
        variable: &PyAny,
        set: &PyGraphColoredVertices,
    ) -> PyResult<PyGraphColoredVertices> {
        let id = self.resolve_variable(variable)?;
        Ok(self
            .as_native()
            .var_can_post_within(id.into(), set.as_native())
            .into())
    }

    /// The same as `can_pre_out`, but only considers transitions of a single variable.
    ///
    /// Variable can be given either as a name, or a `VariableId`.
    pub fn var_can_pre_out(
        &self,
        variable: &PyAny,
        set: &PyGraphColoredVertices,
    ) -> PyResult<PyGraphColoredVertices> {
        let id = self.resolve_variable(variable)?;
        Ok(self
            .as_native()
            .var_can_pre_out(id.into(), set.as_native())
            .into())
    }

    /// The same as `can_post_out`, but only considers transitions of a single variable.
    ///
    /// Variable can be given either as a name, or a `VariableId`.
    pub fn var_can_post_out(
        &self,
        variable: &PyAny,
        set: &PyGraphColoredVertices,
    ) -> PyResult<PyGraphColoredVertices> {
        let id = self.resolve_variable(variable)?;
        Ok(self
            .as_native()
            .var_can_post_out(id.into(), set.as_native())
            .into())
    }

    /// Compute the set of forward-reachable vertices from the given initial set.
    ///
    /// In other words, the result is the smallest forward-closed superset of initial.
    pub fn reach_forward(&self, initial: &PyGraphColoredVertices) -> PyGraphColoredVertices {
        self.as_native().reach_forward(initial.as_native()).into()
    }

    /// Compute the set of backward-reachable vertices from the given `initial` set.
    ///
    /// In other words, the result is the smallest backward-closed superset of `initial`.
    pub fn reach_backward(&self, initial: &PyGraphColoredVertices) -> PyGraphColoredVertices {
        self.as_native().reach_backward(initial.as_native()).into()
    }

    /// Compute the subset of `initial` vertices that can only reach other vertices within
    /// the `initial` set.
    ///
    /// In other words, the result is the largest forward-closed subset of `initial`.
    pub fn trap_forward(&self, initial: &PyGraphColoredVertices) -> PyGraphColoredVertices {
        self.as_native().trap_forward(initial.as_native()).into()
    }

    /// Compute the subset of `initial` vertices that can only be reached from vertices within
    /// the `initial` set.
    ///
    /// In other words, the result is the largest backward-closed subset of `initial`.
    pub fn trap_backward(&self, initial: &PyGraphColoredVertices) -> PyGraphColoredVertices {
        self.as_native().trap_backward(initial.as_native()).into()
    }

    /// Resolve a `VariableId` for a variable given either as a string or as a `VariableId`.
    pub(crate) fn resolve_variable(&self, variable: &PyAny) -> PyResult<PyVariableId> {
        if let Ok(name) = variable.extract::<String>() {
            let var = self
                .as_native()
                .as_network()
                .as_graph()
                .find_variable(name.as_str());
            if let Some(var) = var {
                Ok(var.into())
            } else {
                throw_runtime_error(format!("Unknown variable `{name}`."))
            }
        } else {
            variable.extract::<PyVariableId>()
        }
    }

    /// Resolve a `ParameterId` for a parameter given either as a string or as a `ParameterId`.
    pub(crate) fn resolve_parameter(&self, parameter: &PyAny) -> PyResult<PyParameterId> {
        if let Ok(name) = parameter.extract::<String>() {
            let param = self.as_native().as_network().find_parameter(name.as_str());
            if let Some(param) = param {
                Ok(param.into())
            } else {
                throw_runtime_error(format!("Unknown parameter `{name}`."))
            }
        } else {
            parameter.extract::<PyParameterId>()
        }
    }
}
