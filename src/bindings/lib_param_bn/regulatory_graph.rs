use crate::bindings::lib_param_bn::argument_types::regulation::RegulationOutput;
use crate::bindings::lib_param_bn::argument_types::regulation_type::RegulationType;
use crate::bindings::lib_param_bn::argument_types::sign_type::SignType;
use crate::bindings::lib_param_bn::argument_types::variable_id_multiple_type::VariableIdMultipleType;
use crate::bindings::lib_param_bn::argument_types::variable_id_type::VariableIdType;
use crate::bindings::lib_param_bn::variable_id::{
    VariableId, VariableIdResolvable, VariableIdResolver,
};
use crate::pyo3_utils::richcmp_eq_by_key;
use crate::{AsNative, global_log_level, runtime_error, throw_runtime_error, throw_type_error};
use biodivine_lib_param_bn::Sign::{Negative, Positive};
use biodivine_lib_param_bn::{Monotonicity, SdGraph};
use macros::Wrapper;
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::types::PyList;
use std::collections::HashSet;

/// A regulatory graph is a directed graph consisting of network *variables* connected using
/// *regulations*. Each regulation can be labeled as *essential* (also known as *observable*),
/// and it can have a specified *sign* (also known as *monotonicity*).
///
/// Currently, the set of variables in a regulatory graph is immutable because changing the
/// variable count would disrupt any `VariableId` references to existing variables. However,
/// there are still multiple properties that can be mutated:
///  1. The variable names can be changed using `RegulatoryGraph.set_variable_name`.
///  2. Regulations can be added or removed arbitrarily using `RegulatoryGraph.add_regulation`,
///     `RegulatoryGraph.ensure_regulation`, and `RegulatoryGraph.remove_regulation`.
///  3. The variable set can be modified using the `RegulatoryGraph.extend`,
///     `RegulatoryGraph.drop`, and `RegulatoryGraph.inline_variable` methods. However, these
///     always create a new copy of the graph with a new set of valid `VariableId` objects.
///
///
#[pyclass(module = "biodivine_aeon", subclass)]
#[derive(Clone, Wrapper)]
pub struct RegulatoryGraph(biodivine_lib_param_bn::RegulatoryGraph);

#[pymethods]
impl RegulatoryGraph {
    /// To construct a `RegulatoryGraph`, you have to provide:
    ///  - A list of variable names. If this list is not given, it is inferred from the list of regulations.
    ///  - A list of regulations. These can be either `NamedRegulation` dictionaries or string objects compatible
    ///    with the `.aeon` format notation.
    ///
    /// If you don't provide any arguments, an "empty" `RegulatoryGraph` is constructed with no variables
    /// and no regulations.
    #[new]
    #[pyo3(signature = (variables = None, regulations = None))]
    pub fn new(
        variables: Option<Vec<String>>,
        regulations: Option<Vec<RegulationType>>,
    ) -> PyResult<RegulatoryGraph> {
        // First, try to extract regulation data if it is provided.
        let (regulations, inferred_variables) = if let Some(regulations) = regulations.as_ref() {
            let mut data = Vec::new();
            for item in regulations.iter() {
                data.push(item.resolve_no_context()?);
            }
            let mut variables = HashSet::new();
            for (s, _, _, t) in &data {
                variables.insert(s.clone());
                variables.insert(t.clone());
            }
            let mut variables = Vec::from_iter(variables);
            variables.sort();
            (data, variables)
        } else {
            (Vec::new(), Vec::new())
        };

        // Then build a regulatory graph using either the given variable names or the inferred variable names
        // (if explicit names are not provided).
        let mut graph = if let Some(variables) = variables {
            biodivine_lib_param_bn::RegulatoryGraph::new(variables)
        } else {
            biodivine_lib_param_bn::RegulatoryGraph::new(inferred_variables)
        };

        for (s, m, o, t) in regulations {
            let m = m.as_ref().map(|it| match it {
                Positive => Monotonicity::Activation,
                Negative => Monotonicity::Inhibition,
            });
            if let Err(e) = graph.add_regulation(s.as_str(), t.as_str(), o, m) {
                return throw_runtime_error(e);
            }
        }

        Ok(graph.into())
    }

    fn __str__(&self) -> String {
        format!(
            "RegulatoryGraph(variables={}, regulations={})",
            self.variable_count(),
            self.regulation_count()
        )
    }

    fn __richcmp__(&self, py: Python, other: &Self, op: CompareOp) -> PyResult<Py<PyAny>> {
        richcmp_eq_by_key(py, op, &self, &other, |x| x.as_native())
    }

    fn __repr__(&self) -> String {
        let (names, regulations) = self.__getnewargs__();
        format!("RegulatoryGraph({names:?}, {regulations:?})")
    }

    pub fn __getnewargs__(&self) -> (Vec<String>, Vec<String>) {
        let names = self.variable_names();
        let regulations = self
            .as_native()
            .regulations()
            .map(|it| it.to_string(self.as_native()))
            .collect();
        (names, regulations)
    }

    fn __copy__(&self) -> RegulatoryGraph {
        self.clone()
    }

    fn __deepcopy__(&self, _memo: &Bound<'_, PyAny>) -> RegulatoryGraph {
        self.__copy__()
    }

    /// Try to read the structure of a `RegulatoryGraph` from an `.aeon` file at the specified path.
    #[staticmethod]
    fn from_file(file_path: &str) -> PyResult<RegulatoryGraph> {
        match std::fs::read_to_string(file_path) {
            Err(e) => throw_runtime_error(format!("Cannot read file {file_path}: `{e}`.")),
            Ok(contents) => Self::from_aeon(contents.as_str()),
        }
    }

    /// Try to read the structure of a `RegulatoryGraph` from a string representing the contents of an `.aeon` file.
    #[staticmethod]
    fn from_aeon(file_content: &str) -> PyResult<RegulatoryGraph> {
        biodivine_lib_param_bn::RegulatoryGraph::try_from(file_content)
            .map(RegulatoryGraph)
            .map_err(runtime_error)
    }

    /// Convert this `RegulatoryGraph` to a string representation of a valid `.aeon` file.
    fn to_aeon(&self) -> String {
        let (_, regulations) = self.__getnewargs__();
        regulations.join("\n")
    }

    /// Produce a `graphviz`-compatible `.dot` representation of the underlying graph.
    ///
    /// You can use this in Jupyter notebooks to visualize the `RegulatoryGraph`:
    /// ```python
    /// graph = ...
    ///
    /// import graphviz
    /// graphviz.Source(graph.to_dot())
    /// ```
    fn to_dot(&self) -> String {
        self.as_native().to_dot()
    }

    /// The number of network variables that are represented in this `RegulatoryGraph`.
    pub fn variable_count(&self) -> usize {
        self.as_native().num_vars()
    }

    /// Return the list of all names for all variables managed by this `RegulatoryGraph`.
    ///
    /// The ordering should match the standard ordering of `VariableId` identifiers.
    pub fn variable_names(&self) -> Vec<String> {
        self.as_native().variable_names()
    }

    /// Return the list of all `BddVariable` identifiers valid in this `RegulatoryGraph`.
    pub fn variables(&self) -> Vec<VariableId> {
        self.as_native().variables().map(|it| it.into()).collect()
    }

    /// Return a `VariableId` identifier of the requested `variable`, or `None` if the variable
    /// does not exist in this `RegulatoryGraph`.
    pub fn find_variable(&self, variable: &Bound<'_, PyAny>) -> PyResult<Option<VariableId>> {
        if let Ok(id) = variable.extract::<VariableId>() {
            return if id.__index__() < self.variable_count() {
                Ok(Some(id))
            } else {
                Ok(None)
            };
        }
        if let Ok(name) = variable.extract::<String>() {
            return Ok(self.0.find_variable(name.as_str()).map(Into::into));
        }
        throw_type_error("Expected `VariableId` or `str`.")
    }

    /// Return the string name of the requested `variable`, or throw `RuntimeError` if
    /// such a variable does not exist.
    pub fn get_variable_name(&self, variable: VariableIdType) -> PyResult<String> {
        let var = variable.resolve(self.as_native())?;
        Ok(self.0.get_variable_name(var).clone())
    }

    /// Update the variable name of the provided `variable`. This does not change the
    /// corresponding `VariableId`.
    pub fn set_variable_name(&mut self, variable: VariableIdType, name: &str) -> PyResult<()> {
        let var = variable.resolve(self.as_native())?;
        self.0.set_variable_name(var, name).map_err(runtime_error)
    }

    /// The number of regulations currently managed by this `RegulatoryGraph`.
    pub fn regulation_count(&self) -> usize {
        self.as_native().regulations().count()
    }

    /// Return the list of all regulations (represented as `IdRegulation` dictionaries) that are currently
    /// managed by this `RegulatoryGraph`.
    pub fn regulations<'a>(&self, py: Python<'a>) -> PyResult<Bound<'a, PyList>> {
        let result = PyList::empty(py);
        for reg in self.as_native().regulations() {
            result.append(RegulationOutput::from(reg))?;
        }
        Ok(result)
    }

    /// Return the list of regulations encoded as strings that would appear in the `.aeon` file format.
    pub fn regulation_strings(&self) -> Vec<String> {
        self.as_native()
            .regulations()
            .map(|it| it.to_string(self.as_native()))
            .collect()
    }

    /// Find an `IdRegulation` dictionary that represents the regulation between the two variables, or `None`
    /// if such regulation does not exist.
    pub fn find_regulation(
        &self,
        source: VariableIdType,
        target: VariableIdType,
    ) -> PyResult<Option<RegulationOutput>> {
        let source = source.resolve(self.as_native())?;
        let target = target.resolve(self.as_native())?;
        if let Some(regulation) = self.as_native().find_regulation(source, target) {
            Ok(Some(RegulationOutput::from(regulation)))
        } else {
            Ok(None)
        }
    }

    /// Add a new regulation to the `RegulatoryGraph`, either using a `NamedRegulation`, `IdRegulation`, or
    /// a string representation compatible with the `.aeon` format.
    pub fn add_regulation(&mut self, regulation: RegulationType) -> PyResult<()> {
        let (s, m, o, t) = regulation.resolve_named(self.as_native())?;
        let m = m.map(|it| SignType::from(it).monotonicity());
        self.as_native_mut()
            .add_regulation(s.as_str(), t.as_str(), o, m)
            .map_err(runtime_error)
    }

    /// Remove a regulation that is currently present in this `RegulatoryGraph`. Returns the `IdRegulation`
    /// dictionary that represents the removed regulation, or throws a `RuntimeError` if the regulation
    /// does not exist.
    pub fn remove_regulation(
        &mut self,
        source: VariableIdType,
        target: VariableIdType,
    ) -> PyResult<RegulationOutput> {
        let source = source.resolve(self.as_native())?;
        let target = target.resolve(self.as_native())?;
        let removed = self
            .as_native_mut()
            .remove_regulation(source, target)
            .map_err(runtime_error)?;
        Ok(RegulationOutput::from(&removed))
    }

    /// Update the `sign` and `essential` flags of a regulation in this `RegulatoryGraph`.
    /// If the regulation does not exist, it is created.
    ///
    /// Returns the previous state of the regulation as an `IdRegulation` dictionary, assuming the regulation
    /// already existed.
    pub fn ensure_regulation(
        &mut self,
        regulation: RegulationType,
    ) -> PyResult<Option<RegulationOutput>> {
        // This is a bit inefficient but should be good enough for now.
        let (s, m, o, t) = regulation.resolve_id(self.as_native())?;
        let old = self.as_native_mut().remove_regulation(s, t).ok();
        let m = m.map(|it| SignType::from(it).monotonicity());
        self.as_native_mut()
            .add_raw_regulation(biodivine_lib_param_bn::Regulation {
                regulator: s,
                target: t,
                observable: o,
                monotonicity: m,
            })
            .map_err(runtime_error)?;
        Ok(old.map(|it| RegulationOutput::from(&it)))
    }

    /// Create a copy of this `RegulatoryGraph` that is extended with the given list of `variables`.
    ///
    /// The new variables are added *after* the existing variables, so any previously used `VariableId` references
    /// are still valid. However, the added names must still be unique within the new graph.
    pub fn extend(&self, mut variables: Vec<String>) -> PyResult<RegulatoryGraph> {
        let (mut names, regulations) = self.__getnewargs__();
        names.append(&mut variables);
        let mut result = Self::new(Some(names), None)?;
        for reg in regulations {
            result
                .as_native_mut()
                .add_string_regulation(reg.as_str())
                .map_err(runtime_error)?;
        }
        Ok(result)
    }

    /// Create a copy of this `RegulatoryGraph` with all the specified variables (and their associated regulations)
    /// removed.
    ///
    /// Throws `RuntimeError` if one of the variables does not exist.
    ///
    /// The new graph follows the variable ordering of the old graph, but since there are now variables that are
    /// missing in the new graph, the `VariableId` objects are not compatible with the original graph.
    pub fn drop(&self, variables: VariableIdMultipleType) -> PyResult<RegulatoryGraph> {
        let to_remove: Vec<VariableIdType> = variables.into();
        let to_remove: Vec<biodivine_lib_param_bn::VariableId> =
            VariableIdType::resolve_collection(to_remove, self.as_native())?;
        let to_remove = to_remove
            .into_iter()
            .map(|it| VariableIdResolver::get_name(self.as_native(), it))
            .collect::<HashSet<_>>();
        let names = self
            .variable_names()
            .into_iter()
            .filter(|it| !to_remove.contains(it))
            .collect::<Vec<_>>();
        let mut result = Self::new(Some(names), None)?;
        for reg in self.as_native().regulations() {
            let source = self.as_native().get_variable_name(reg.get_regulator());
            let target = self.as_native().get_variable_name(reg.get_target());
            if to_remove.contains(source) || to_remove.contains(target) {
                continue;
            }
            result
                .as_native_mut()
                .add_regulation(
                    source.as_str(),
                    target.as_str(),
                    reg.is_observable(),
                    reg.get_monotonicity(),
                )
                .map_err(runtime_error)?;
        }

        Ok(result)
    }

    /// Inline a variable into its downstream targets. This also "merges" the essential and sign flags of
    /// the associated regulations in a way that makes sense for the existing constraints (e.g. `+` and `-` becomes
    /// `-`, `-` and `-` becomes `+`; a regulation is essential if both "partial" regulations are essential, etc.).
    ///
    /// Raises a `RuntimeError` if the inlined variable has a self-regulation. This is because inlining
    /// a self-regulated variable potentially "erases" a feedback loop in the graph, which can fundamentally
    /// change its behavior. And as opposed to `RegulatoryGraph.drop`, the intention of this method is to produce
    /// a result that is functionally compatible with the original regulatory graph. Of course, you can use
    /// `RegulatoryGraph.remove_regulation` to explicitly remove the self-loop before inlining the variable.
    pub fn inline_variable(&self, variable: VariableIdType) -> PyResult<RegulatoryGraph> {
        let variable = variable.resolve(self.as_native())?;
        let bn = biodivine_lib_param_bn::BooleanNetwork::new(self.as_native().clone());
        let Some(bn) = bn.inline_variable(variable, false) else {
            return throw_runtime_error("Variable has a self-regulation.");
        };
        Ok(RegulatoryGraph(bn.as_graph().clone()))
    }

    /// Make a copy of this `RegulatoryGraph` with all constraints on the regulations removed.
    /// In particular, every regulation is set to non-essential with an unknown sign.
    pub fn remove_regulation_constraints(&self) -> PyResult<RegulatoryGraph> {
        let native = self.as_native();
        let bn = biodivine_lib_param_bn::BooleanNetwork::new(native.clone());
        let bn = bn.remove_static_constraints();
        Ok(RegulatoryGraph(bn.as_graph().clone()))
    }

    /// Compute the `set` of all predecessors (regulators) of a specific variable.
    pub fn predecessors(&self, variable: VariableIdType) -> PyResult<HashSet<VariableId>> {
        let variable = variable.resolve(self.as_native())?;
        Ok(self
            .as_native()
            .regulators(variable)
            .into_iter()
            .map(VariableId::from)
            .collect())
    }

    /// Compute the `set` of all successors (targets) of a specific variable.
    pub fn successors(&self, variable: VariableIdType) -> PyResult<HashSet<VariableId>> {
        let variable = variable.resolve(self.as_native())?;
        Ok(self
            .as_native()
            .targets(variable)
            .into_iter()
            .map(VariableId::from)
            .collect())
    }

    /// The set of all variables that transitively regulate the given variable, or a set of variables.
    ///
    /// If `subgraph` is specified, the search is limited to a subgraph induced by the given collection of variables.
    #[pyo3(signature = (pivots, subgraph = None))]
    pub fn backward_reachable(
        &self,
        pivots: VariableIdMultipleType,
        subgraph: Option<Vec<VariableIdType>>,
    ) -> PyResult<HashSet<VariableId>> {
        let pivots: Vec<VariableIdType> = pivots.into();
        let pivots = VariableIdType::resolve_collection(pivots, self.as_native())?;
        let subgraph = self.resolve_subgraph(subgraph)?;
        let sd_graph = SdGraph::from(self.as_native());
        Ok(sd_graph
            .restricted_backward_reachable(&subgraph, pivots)
            .into_iter()
            .map(VariableId::from)
            .collect())
    }

    /// The set of all variables that are transitively regulated by the given variable, or a set of variables.
    ///
    /// If `subgraph` is specified, the search is limited to a subgraph induced by the given collection of variables.
    #[pyo3(signature = (pivots, subgraph = None))]
    pub fn forward_reachable(
        &self,
        pivots: VariableIdMultipleType,
        subgraph: Option<Vec<VariableIdType>>,
    ) -> PyResult<HashSet<VariableId>> {
        let pivots: Vec<VariableIdType> = pivots.into();
        let pivots = VariableIdType::resolve_collection(pivots, self.as_native())?;
        let subgraph = self.resolve_subgraph(subgraph)?;
        let sd_graph = SdGraph::from(self.as_native());
        Ok(sd_graph
            .restricted_forward_reachable(&subgraph, pivots)
            .into_iter()
            .map(VariableId::from)
            .collect())
    }

    /// Heuristically computes an approximation of a minimal feedback vertex set of this `RegulatoryGraph`.
    ///
    /// A feedback vertex set (FVS) is a set of variables that once removed cause the graph to become acyclic.
    /// The set is minimal if there is no smaller set that is also an FVS (in terms of cardinality).
    ///
    /// You can specify a `subgraph` restriction, in which case the algorithm operates only on the subgraph
    /// induced by the provided variables. Similarly, you can specify `parity`, which causes the algorithm to
    /// only consider positive or negative cycles when evaluating the validity of an FVS.
    ///
    /// Finally, note that the algorithm is not exact in the sense that it can result in a non-minimal FVS,
    /// but the FVS is always *correct* in the context of this `RegulatoryGraph` (or the specified `subgraph`).
    /// The algorithm is deterministic.
    #[pyo3(signature = (parity = None, subgraph = None))]
    pub fn feedback_vertex_set(
        &self,
        py: Python,
        parity: Option<SignType>,
        subgraph: Option<Vec<VariableIdType>>,
    ) -> PyResult<HashSet<VariableId>> {
        cancel_this::on_python(|| {
            let log_level = global_log_level(py)?;
            let sd_graph = SdGraph::from(self.as_native());
            let restriction = self.resolve_subgraph(subgraph)?;
            let fvs = if let Some(parity) = parity {
                sd_graph._restricted_parity_feedback_vertex_set(
                    &restriction,
                    parity.sign(),
                    log_level,
                )?
            } else {
                sd_graph._restricted_feedback_vertex_set(&restriction, log_level)?
            };
            Ok(fvs.into_iter().map(VariableId::from).collect())
        })
    }

    /// Heuristically computes an approximation of a maximal set of independent cycles of this `RegulatoryGraph`.
    ///
    /// Two cycles are independent if they do not intersect. A set of independent cycles (IC set) is maximal if
    /// it has the largest possible cardinality with all cycles being pair-wise disjoint.
    ///
    /// You can specify a `subgraph` restriction, in which case the algorithm operates only on the subgraph
    /// induced by the provided variables. Similarly, you can specify `parity`, which causes the algorithm to
    /// only consider positive or negative cycles when evaluating the validity of an IC set.
    ///
    /// Finally, note that the algorithm is not exact in the sense that it can result in a non-maximal IC set,
    /// but the set is always *correct* in the context of this `RegulatoryGraph` (or the specified `subgraph`).
    /// The algorithm is deterministic and the results are sorted from shortest to longest.
    #[pyo3(signature = (parity = None, subgraph = None))]
    pub fn independent_cycles(
        &self,
        py: Python,
        parity: Option<SignType>,
        subgraph: Option<Vec<VariableIdType>>,
    ) -> PyResult<Vec<Vec<VariableId>>> {
        cancel_this::on_python(|| {
            let log_level = global_log_level(py)?;
            let sd_graph = SdGraph::from(self.as_native());
            let restriction = self.resolve_subgraph(subgraph)?;
            let cycles = if let Some(parity) = parity {
                sd_graph._restricted_independent_parity_cycles(
                    &restriction,
                    parity.sign(),
                    log_level,
                )?
            } else {
                sd_graph._restricted_independent_cycles(&restriction, log_level)?
            };
            let cycles = cycles
                .into_iter()
                .map(|cycle| cycle.into_iter().map(VariableId::from).collect::<Vec<_>>())
                .collect();
            Ok(cycles)
        })
    }

    /// Compute the set of *non-trivial* strongly connected components of this `RegulatoryGraph`.
    ///
    /// If the `subgraph` option is specified, only operates on the subgraph induced by these variables.
    ///
    /// Note that a single variable with a self-regulation is considered a non-trivial SCC, even if it is not
    /// a member of a larger component.
    #[pyo3(signature = (subgraph = None))]
    pub fn strongly_connected_components(
        &self,
        py: Python,
        subgraph: Option<Vec<VariableIdType>>,
    ) -> PyResult<Vec<HashSet<VariableId>>> {
        cancel_this::on_python(|| {
            let subgraph = self.resolve_subgraph(subgraph)?;
            let sd_graph = SdGraph::from(self.as_native());
            let components = sd_graph
                ._restricted_strongly_connected_components(&subgraph, global_log_level(py)?)?;
            Ok(components
                .into_iter()
                .map(|c| c.into_iter().map(|it| it.into()).collect())
                .collect())
        })
    }

    /// Compute the set of weakly connected components of this `RegulatoryGraph`. Note that typical regulatory graphs
    /// represent a single weakly connected component.
    ///
    /// If the `subgraph` option is specified, only operates on the subgraph induced by these variables.
    #[pyo3(signature = (subgraph = None))]
    pub fn weakly_connected_components(
        &self,
        py: Python,
        subgraph: Option<Vec<VariableIdType>>,
    ) -> PyResult<Vec<HashSet<VariableId>>> {
        cancel_this::on_python(|| {
            let subgraph = self.resolve_subgraph(subgraph)?;
            let sd_graph = SdGraph::from(self.as_native());
            let components = sd_graph
                ._restricted_weakly_connected_components(&subgraph, global_log_level(py)?)?;
            Ok(components
                .into_iter()
                .map(|c| c.into_iter().map(|it| it.into()).collect())
                .collect())
        })
    }

    /// Find the shortest cycle in this `RegulatoryGraph` that contains the `pivot` variable, or `None` if no such
    /// cycle exists.
    ///
    /// You can further restrict the algorithm using:
    ///  - `parity`: restricts the search to only positive/negative cycles.
    ///  - `subgraph`: only considers cycles that fully belong to the specified induced subgraph.
    ///  - `length`: only return cycles which are shorter or equal to the provided length.
    ///
    /// The length of a cycle is counted in terms of edges, and a self-loop is thus a cycle of length one. If there
    /// are multiple shortest cycles, the algorithm always deterministically picks one such cycle, but
    /// the exact criterion is not documented. The result is ordered such that the first variable in the list
    /// is always the pivot vertex.
    #[pyo3(signature = (pivot, parity = None, subgraph = None, length = None))]
    pub fn shortest_cycle(
        &self,
        pivot: VariableIdType,
        parity: Option<SignType>,
        subgraph: Option<Vec<VariableIdType>>,
        length: Option<usize>,
    ) -> PyResult<Option<Vec<VariableId>>> {
        let pivot = pivot.resolve(self.as_native())?;
        let subgraph = self.resolve_subgraph(subgraph)?;
        let length = length.unwrap_or(usize::MAX);
        let sd_graph = SdGraph::from(self.as_native());

        let cycle = if let Some(parity) = parity {
            sd_graph.shortest_parity_cycle(&subgraph, pivot, parity.sign(), length)
        } else {
            sd_graph.shortest_cycle(&subgraph, pivot, length)
        };

        Ok(cycle.map(|c| c.into_iter().map(VariableId::from).collect()))
    }
}

impl RegulatoryGraph {
    /// Convert an optional value into a set of graph variables. These typically represent an induced subgraph
    /// to which an operation should be applied.
    pub fn resolve_subgraph(
        &self,
        variables: Option<Vec<VariableIdType>>,
    ) -> PyResult<HashSet<biodivine_lib_param_bn::VariableId>> {
        let Some(variables) = variables else {
            // If no value is given, we consider the full sub-graph always.
            return Ok(HashSet::from_iter(self.as_native().variables()));
        };

        VariableIdType::resolve_collection(variables, self.as_native())
    }
}
