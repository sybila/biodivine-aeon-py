extern crate biodivine_bdd;
extern crate biodivine_lib_param_bn;

use biodivine_bdd::{Bdd, BddVariable, BddVariableSet, BddVariableSetBuilder, BooleanExpression};
use biodivine_lib_param_bn::biodivine_std::bitvector::{ArrayBitVector, BitVector};
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{
    GraphColoredVertices, GraphColors, GraphVertices,
};
use biodivine_lib_param_bn::Monotonicity;
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

#[pyclass]
#[derive(Clone)]
pub struct VertexSet(biodivine_lib_param_bn::symbolic_async_graph::GraphVertices);

impl From<VertexSet> for biodivine_lib_param_bn::symbolic_async_graph::GraphVertices {
    fn from(value: VertexSet) -> Self {
        value.0
    }
}

impl From<biodivine_lib_param_bn::symbolic_async_graph::GraphVertices> for VertexSet {
    fn from(value: GraphVertices) -> Self {
        VertexSet(value)
    }
}

#[pymethods]
impl VertexSet {
    /// Convert this set to a raw Bdd.
    pub fn to_bdd(&self) -> Bdd {
        self.0.as_bdd().clone().into()
    }

    /// Populate a new set with a raw Bdd.
    pub fn copy_with(&self, bdd: Bdd) -> Self {
        self.0.copy(bdd.into()).into()
    }

    /// Populate a new set with a raw Bdd in a string.
    pub fn copy_with_raw_string(&self, bdd: String) -> PyResult<Self> {
        Ok(self
            .0
            .copy(Bdd::from_raw_string(bdd.as_str()).into())
            .into())
    }

    /// Get an approximate count of elements in this set.
    pub fn cardinality(&self) -> f64 {
        self.0.approx_cardinality()
    }

    /// Get an approximate memory consumption of this symbolic set in bytes.
    ///
    /// (real value may be different due to OS and allocation specifics)
    pub fn symbolic_size(&self) -> usize {
        self.0.symbolic_size() * 10
    }

    /// Compute a `.dot` string representing the underlying BDD graph.
    ///
    /// Needs a reference to the underlying symbolic graph to resolve variable names
    pub fn to_dot(&self, graph: &SymbolicAsyncGraph) -> String {
        self.0.to_dot_string(graph.0.symbolic_context())
    }

    /// Compute a union of two sets.
    pub fn union(&self, other: &Self) -> Self {
        self.0.union(&other.0).into()
    }

    /// Compute an intersection of two sets.
    pub fn intersect(&self, other: &Self) -> Self {
        self.0.intersect(&other.0).into()
    }

    /// Compute a difference of two sets.
    pub fn minus(&self, other: &Self) -> Self {
        self.0.minus(&other.0).into()
    }

    /// Returns true if this set is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns true if this set is a subset.
    pub fn is_subset(&self, other: &Self) -> bool {
        self.0.is_subset(&other.0)
    }

    /// Pick a single vertex from this set, and returns it as a singleton set.
    ///
    /// If the set is empty, also returns an empty set.
    pub fn pick_singleton(&self) -> Self {
        self.0.pick_singleton().into()
    }

    /// Turn this symbolic set into an explicit list of vertices (each represented as a Boolean
    /// vector). Note that if this set is large, this may exhaust system memory easily.
    pub fn vertices(&self) -> Vec<Vec<bool>> {
        self.0
            .materialize()
            .iter()
            .map(|bv| bv.values())
            .collect::<Vec<_>>()
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("VertexSet({})", self.0.approx_cardinality()))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("VertexSet({})", self.0.approx_cardinality()))
    }
}

#[pyclass]
#[derive(Clone)]
pub struct ColoredVertexSet(pub biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices);

impl From<ColoredVertexSet> for biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices {
    fn from(value: ColoredVertexSet) -> Self {
        value.0
    }
}

impl From<biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices> for ColoredVertexSet {
    fn from(value: GraphColoredVertices) -> Self {
        ColoredVertexSet(value)
    }
}

#[pymethods]
impl ColoredVertexSet {
    /// Convert this set to a raw Bdd.
    pub fn to_bdd(&self) -> Bdd {
        self.0.as_bdd().clone().into()
    }

    /// Populate a new set with a raw Bdd.
    pub fn copy_with(&self, bdd: Bdd) -> Self {
        self.0.copy(bdd.into()).into()
    }

    /// Populate a new set with a raw Bdd in a string.
    pub fn copy_with_raw_string(&self, bdd: String) -> PyResult<Self> {
        Ok(self
            .0
            .copy(Bdd::from_raw_string(bdd.as_str()).into())
            .into())
    }

    /// Get an approximate count of elements in this set.
    pub fn cardinality(&self) -> f64 {
        self.0.approx_cardinality()
    }

    /// Get an approximate memory consumption of this symbolic set in bytes.
    ///
    /// (real value may be different due to OS and allocation specifics)
    pub fn symbolic_size(&self) -> usize {
        self.0.symbolic_size() * 10
    }

    /// Compute a `.dot` string representing the underlying BDD graph.
    ///
    /// Needs a reference to the underlying symbolic graph to resolve variable names
    pub fn to_dot(&self, graph: &SymbolicAsyncGraph) -> String {
        self.0.to_dot_string(graph.0.symbolic_context())
    }

    /// Compute a union of two sets.
    pub fn union(&self, other: &Self) -> Self {
        self.0.union(&other.0).into()
    }

    /// Compute an intersection of two sets.
    pub fn intersect(&self, other: &Self) -> Self {
        self.0.intersect(&other.0).into()
    }

    /// Compute a difference of two sets.
    pub fn minus(&self, other: &Self) -> Self {
        self.0.minus(&other.0).into()
    }

    /// Returns true if this set is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns true if this set is a subset.
    pub fn is_subset(&self, other: &Self) -> bool {
        self.0.is_subset(&other.0)
    }

    /// Return only vertices in this set.
    pub fn vertices(&self) -> VertexSet {
        self.0.vertices().into()
    }

    /// Return only colors in this set.
    pub fn colors(&self) -> ColorSet {
        self.0.colors().into()
    }

    /// Pick a single color-vertex pair from this set.
    pub fn pick_singleton(&self) -> Self {
        self.0.pick_singleton().into()
    }

    /// For every vertex in this set, pick exactly one color.
    pub fn pick_color(&self) -> Self {
        self.0.pick_color().into()
    }

    /// For every color in this set, pick exactly one vertex.
    pub fn pick_vertex(&self) -> Self {
        self.0.pick_vertex().into()
    }

    /// Remove given color from this set for all vertices.
    pub fn minus_colors(&self, other: &ColorSet) -> Self {
        self.0.minus_colors(&other.0).into()
    }

    /// Keep only colours in the given set for all vertices.
    pub fn intersect_colors(&self, other: &ColorSet) -> Self {
        self.0.intersect_colors(&other.0).into()
    }

    /// Remove vertices from the given set for any color.
    pub fn minus_vertices(&self, other: &VertexSet) -> Self {
        self.0.minus_vertices(&other.0).into()
    }

    /// Retain only vertices from the given set, for any color.
    pub fn intersect_vertices(&self, other: &VertexSet) -> Self {
        self.0.intersect_vertices(&other.0).into()
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!(
            "ColoredVertexSet({}, {}x{})",
            self.0.approx_cardinality(),
            self.0.vertices().approx_cardinality(),
            self.0.colors().approx_cardinality()
        ))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "ColoredVertexSet({}, {}x{})",
            self.0.approx_cardinality(),
            self.0.vertices().approx_cardinality(),
            self.0.colors().approx_cardinality()
        ))
    }
}

#[pyclass]
#[derive(Clone)]
pub struct SymbolicAsyncGraph(pub biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph);

impl From<SymbolicAsyncGraph> for biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph {
    fn from(value: SymbolicAsyncGraph) -> Self {
        value.0
    }
}

impl From<biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph> for SymbolicAsyncGraph {
    fn from(value: biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph) -> Self {
        SymbolicAsyncGraph(value)
    }
}

#[pymethods]
impl SymbolicAsyncGraph {
    /// Create a new symbolic async graph from a Boolean network.
    #[new]
    pub fn new(network: BooleanNetwork) -> PyResult<SymbolicAsyncGraph> {
        let result =
            biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph::new(network.into());
        match result {
            Ok(graph) => Ok(graph.into()),
            Err(e) => Err(PyTypeError::new_err(e)),
        }
    }

    /// Get the underlying Boolean network of this graph.
    pub fn network(&self) -> BooleanNetwork {
        self.0.as_network().clone().into()
    }

    /// Get the variable set that is used for symbolic encoding.
    pub fn bdd_variables(&self) -> BddVariableSet {
        self.0.symbolic_context().bdd_variable_set().clone().into()
    }

    /// Create a set which contains every color-vertex pair with a specified variable fixed
    /// to the specified constant.
    pub fn fix_variable(&self, variable: &PyAny, value: bool) -> PyResult<ColoredVertexSet> {
        let id = self.resolve_variable(variable)?;
        Ok(self.0.fix_network_variable(id.into(), value).into())
    }

    /// Given a graph vertex (as a boolean vector), create a set of color-vertex pairs
    /// which contains exactly this one vertex with all possible graph colors.
    pub fn fix_vertex(&self, vertex: Vec<bool>) -> ColoredVertexSet {
        self.0
            .vertex(&ArrayBitVector::from_bool_vector(vertex))
            .into()
    }

    /// Create a set of colors in which a particular logical parameter is fixed to a specific
    /// value. A logical parameter is any parameter of arity 0.
    pub fn fix_parameter(&self, parameter: &PyAny, value: bool) -> PyResult<ColorSet> {
        // Find parameter and validate.
        let id = self.resolve_parameter(parameter)?;
        let param = self.0.as_network().get_parameter(id.into());
        if param.get_arity() != 0 {
            return Err(PyTypeError::new_err(format!(
                "Parameter {} has non-zero arity.",
                param.get_name()
            )));
        }
        // Make a BDD using symbolic context.
        let ctx = self.0.symbolic_context();
        let bdd = ctx.mk_uninterpreted_function_is_true(id.into(), &[]);
        // Apply desired value.
        let bdd = if value { bdd } else { bdd.not() };
        // Export as colour set.
        let unit_bdd = self.0.unit_colors().as_bdd();
        Ok(self.0.empty_colors().copy(bdd.and(unit_bdd)).into())
    }

    /// Create a subset of the unit color set which fixes the value of exactly one row in an
    /// explicit uninterpreted function.
    ///
    /// That is, given a (here binary) function `f`, input vector `[True, False]` and
    /// a Boolean `value`, the resulting set will only have colours `f(True, False) = value`
    /// which are otherwise valid.
    pub fn fix_explicit_function(
        &self,
        parameter: &PyAny,
        inputs: Vec<bool>,
        value: bool,
    ) -> PyResult<ColorSet> {
        let id = self.resolve_parameter(parameter)?;
        let param = self.0.as_network().get_parameter(id.into());
        if (param.get_arity() as usize) != inputs.len() {
            return Err(PyTypeError::new_err(format!(
                "Artiy mismatch for parameter {}.",
                param.get_name()
            )));
        }
        let ctx = self.0.symbolic_context();
        let table = ctx.get_explicit_function_table(id.into());
        let mut bdd = ctx.mk_constant(false);
        for (row, bdd_var) in table {
            if row == inputs {
                bdd = ctx.bdd_variable_set().mk_literal(bdd_var, value);
            }
        }
        let unit_bdd = self.0.unit_colors().as_bdd();
        Ok(self.0.empty_colors().copy(bdd.and(unit_bdd)).into())
    }

    /// Create a subset of the unit color set which fixed the output value of exactly one row in
    /// an implicit update function.
    ///
    /// That is, assume a network variable `x` which has an implicit update function and two
    /// regulators. Now, given function row `[True, False]` and a Boolean `value`, the result
    /// will be a subset of the unit color set where `f_x(True, False) = value`.
    pub fn fix_implicit_function(
        &self,
        variable: &PyAny,
        inputs: Vec<bool>,
        value: bool,
    ) -> PyResult<ColorSet> {
        let id = self.resolve_variable(variable)?;
        if self.0.as_network().regulators(id.into()).len() != inputs.len() {
            let name = self.0.as_network().get_variable_name(id.into());
            return Err(PyTypeError::new_err(format!(
                "Artiy mismatch for variable {}.",
                name
            )));
        }
        let ctx = self.0.symbolic_context();
        let table = ctx.get_implicit_function_table(id.into());
        let mut bdd = ctx.mk_constant(false);
        for (row, bdd_var) in table {
            if row == inputs {
                bdd = ctx.bdd_variable_set().mk_literal(bdd_var, value);
            }
        }
        let unit_bdd = self.0.unit_colors().as_bdd();
        Ok(self.0.empty_colors().copy(bdd.and(unit_bdd)).into())
    }

    /// Create a Boolean network which matches this graph, but its parameters are fully specified,
    /// and the specification is picked from the given color set.
    pub fn pick_witness(&self, colors: &ColorSet) -> BooleanNetwork {
        self.0.pick_witness(&colors.0).into()
    }

    /// Make an empty `ColorSet`.
    pub fn empty_colors(&self) -> ColorSet {
        self.0.mk_empty_colors().into()
    }

    /// Make an empty `ColoredVertexSet`.
    pub fn empty_colored_vertices(&self) -> ColoredVertexSet {
        self.0.mk_empty_vertices().into()
    }

    /// Return all colors valid in this graph.
    pub fn unit_colors(&self) -> ColorSet {
        self.0.mk_unit_colors().into()
    }

    /// Return all color-vertex pairs valid in this graph.
    pub fn unit_colored_vertices(&self) -> ColoredVertexSet {
        self.0.mk_unit_colored_vertices().into()
    }

    /// Compute all successors of the provided color-vertex pairs.
    pub fn post(&self, set: &ColoredVertexSet) -> ColoredVertexSet {
        self.0.post(&set.0).into()
    }

    /// Compute all predecessors of the provided color-vertex pairs.
    pub fn pre(&self, set: &ColoredVertexSet) -> ColoredVertexSet {
        self.0.pre(&set.0).into()
    }

    /// Compute a subset of the given color-vertex pairs that can perform a transition.
    pub fn can_post(&self, set: &ColoredVertexSet) -> ColoredVertexSet {
        self.0.can_post(&set.0).into()
    }

    /// Compute a subset of the given color-vertex pairs that can be reached by a transition.
    pub fn can_pre(&self, set: &ColoredVertexSet) -> ColoredVertexSet {
        self.0.can_pre(&set.0).into()
    }

    /// Compute the subset of the given set that can reach a state *within* the same
    /// set using *some* transition.
    pub fn can_post_within(&self, set: &ColoredVertexSet) -> ColoredVertexSet {
        self.0.can_post_within(&set.0).into()
    }

    /// Compute the subset of the given set that can be reached from a state *within* the same
    /// set using *some* transition.
    pub fn can_pre_within(&self, set: &ColoredVertexSet) -> ColoredVertexSet {
        self.0.can_pre_within(&set.0).into()
    }

    /// Compute the subset of the given set that can reach a state *outside* the same
    /// set using *some* transition.
    pub fn can_post_out(&self, set: &ColoredVertexSet) -> ColoredVertexSet {
        self.0.can_post_out(&set.0).into()
    }

    /// Compute the subset of the given set that can be reached from a state *outside* the same
    /// set using *some* transition.
    pub fn can_pre_out(&self, set: &ColoredVertexSet) -> ColoredVertexSet {
        self.0.can_pre_out(&set.0).into()
    }

    /// Compute the subset of the given set that will reach a state *within* the same
    /// set using *every* admissible transition.
    ///
    /// States which have no outgoing transitions are also included.
    pub fn will_post_within(&self, set: &ColoredVertexSet) -> ColoredVertexSet {
        self.0.will_post_within(&set.0).into()
    }

    /// Compute the subset of the given set that will be reached from a state *within* the same
    /// set using *every* admissible transition.
    ///
    /// States which have no incoming transitions are also included.
    pub fn will_pre_within(&self, set: &ColoredVertexSet) -> ColoredVertexSet {
        self.0.will_pre_within(&set.0).into()
    }

    /// Compute the subset of the given set that will reach a state *outside* the same
    /// set using *every* admissible transition.
    ///
    /// States which have no outgoing transitions are also included.
    pub fn will_post_out(&self, set: &ColoredVertexSet) -> ColoredVertexSet {
        self.0.will_post_out(&set.0).into()
    }

    /// Compute the subset of the given set that will be reached from a state *outside* the same
    /// set using *every* admissible transition.
    ///
    /// States which have no incoming transitions are also included.
    pub fn will_pre_out(&self, set: &ColoredVertexSet) -> ColoredVertexSet {
        self.0.will_pre_out(&set.0).into()
    }

    /// The same as `post`, but only for transitions under one variable.
    pub fn var_post(&self, variable: &PyAny, set: &ColoredVertexSet) -> PyResult<ColoredVertexSet> {
        let id = self.resolve_variable(variable)?;
        Ok(self.0.var_post(id.into(), &set.0).into())
    }

    /// The same as `pre`, but only for transitions under one variable.
    pub fn var_pre(&self, variable: &PyAny, set: &ColoredVertexSet) -> PyResult<ColoredVertexSet> {
        let id = self.resolve_variable(variable)?;
        Ok(self.0.var_pre(id.into(), &set.0).into())
    }

    /// The same as `can_post`, but only for transitions under one variable.
    pub fn var_can_post(
        &self,
        variable: &PyAny,
        set: &ColoredVertexSet,
    ) -> PyResult<ColoredVertexSet> {
        let id = self.resolve_variable(variable)?;
        Ok(self.0.var_can_post(id.into(), &set.0).into())
    }

    /// The same as `can_pre`, but only for transitions under one variable.
    pub fn var_can_pre(
        &self,
        variable: &PyAny,
        set: &ColoredVertexSet,
    ) -> PyResult<ColoredVertexSet> {
        let id = self.resolve_variable(variable)?;
        Ok(self.0.var_can_pre(id.into(), &set.0).into())
    }

    /// Compute the subset of `set` that has predecessors *within* `set` using only trasition
    /// under the given `variable`.
    pub fn var_can_pre_within(
        &self,
        variable: &PyAny,
        set: &ColoredVertexSet,
    ) -> PyResult<ColoredVertexSet> {
        let id = self.resolve_variable(variable)?;
        Ok(self.0.var_can_pre_within(id.into(), &set.0).into())
    }

    /// Compute the subset of `set` that has successors *within* `set` using only trasition
    /// under the given `variable`.
    pub fn var_can_post_within(
        &self,
        variable: &PyAny,
        set: &ColoredVertexSet,
    ) -> PyResult<ColoredVertexSet> {
        let id = self.resolve_variable(variable)?;
        Ok(self.0.var_can_post_within(id.into(), &set.0).into())
    }

    /// Compute the subset of `set` that has predecessors *outside* `set` using only trasitions
    /// under the given `variable`.
    pub fn var_can_pre_out(
        &self,
        variable: &PyAny,
        set: &ColoredVertexSet,
    ) -> PyResult<ColoredVertexSet> {
        let id = self.resolve_variable(variable)?;
        Ok(self.0.var_can_pre_out(id.into(), &set.0).into())
    }

    /// Compute the subset of `set` that has successors *outside* `set` using only trasitions
    /// under the given `variable`.
    pub fn var_can_post_out(
        &self,
        variable: &PyAny,
        set: &ColoredVertexSet,
    ) -> PyResult<ColoredVertexSet> {
        let id = self.resolve_variable(variable)?;
        Ok(self.0.var_can_post_out(id.into(), &set.0).into())
    }

    /// Resolve a variable that is either a string or a numeric id
    fn resolve_variable(&self, variable: &PyAny) -> PyResult<VariableId> {
        if let Ok(name) = variable.extract::<String>() {
            let var = self.0.as_network().as_graph().find_variable(name.as_str());
            var.map(|var| var.into())
                .ok_or_else(|| PyTypeError::new_err(format!("Unknown variable `{}`.", name)))
        } else {
            variable.extract::<VariableId>()
        }
    }

    /// Resolve a parameter that is either a string or a numeric id
    fn resolve_parameter(&self, parameter: &PyAny) -> PyResult<ParameterId> {
        if let Ok(name) = parameter.extract::<String>() {
            let param = self.0.as_network().find_parameter(name.as_str());
            param
                .map(|param| param.into())
                .ok_or_else(|| PyTypeError::new_err(format!("Unknown parameter `{}`.", name)))
        } else {
            parameter.extract::<ParameterId>()
        }
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn biodivine_boolean_networks(_py: Python, module: &PyModule) -> PyResult<()> {
    module.add_class::<VariableId>()?;
    module.add_class::<ParameterId>()?;
    module.add_class::<RegulatoryGraph>()?;
    module.add_class::<BooleanNetwork>()?;
    module.add_class::<SymbolicAsyncGraph>()?;
    module.add_class::<ColorSet>()?;
    module.add_class::<VertexSet>()?;
    module.add_class::<ColoredVertexSet>()?;
    // Re-export everything here as well, because the types are incompatible in Python :/
    module.add_class::<Bdd>()?;
    module.add_class::<BddVariable>()?;
    module.add_class::<BddVariableSet>()?;
    module.add_class::<BddVariableSetBuilder>()?;
    module.add_class::<BooleanExpression>()?;
    Ok(())
}
