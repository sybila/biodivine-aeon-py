use crate::bindings::annotations::regulatory_graph::RegulatoryGraphAnnotation;
use crate::bindings::lib_param_bn::model_annotation::{ModelAnnotation, ModelAnnotationRoot};
use crate::bindings::lib_param_bn::variable_id::VariableId;
use crate::bindings::lib_param_bn::NetworkVariableContext;
use crate::pyo3_utils::{richcmp_eq_by_key, BoolLikeValue, SignValue};
use crate::{
    global_log_level, runtime_error, throw_index_error, throw_runtime_error, throw_type_error,
    AsNative,
};
use biodivine_lib_param_bn::Sign::{Negative, Positive};
use biodivine_lib_param_bn::{Monotonicity, SdGraph, Sign};
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PySet};
use std::collections::HashSet;

/// A regulatory graph is a directed graph consisting of network *variables* connected using
/// *regulations*. Each regulation can be labeled as *essential* (also known as *observable*)
/// and it can have a specified *sign* (also known as *monotonicity*).
///
/// Currently, the set of variables in a regulatory graph is immutable, because changing the
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
#[derive(Clone)]
pub struct RegulatoryGraph {
    native: biodivine_lib_param_bn::RegulatoryGraph,
    // Annotation metadata that is associated with this model.
    pub annotations: Option<Py<ModelAnnotationRoot>>,
}

impl From<biodivine_lib_param_bn::RegulatoryGraph> for RegulatoryGraph {
    fn from(value: biodivine_lib_param_bn::RegulatoryGraph) -> Self {
        RegulatoryGraph {
            native: value,
            annotations: None,
        }
    }
}

impl From<RegulatoryGraph> for biodivine_lib_param_bn::RegulatoryGraph {
    fn from(value: RegulatoryGraph) -> Self {
        value.native
    }
}

impl AsNative<biodivine_lib_param_bn::RegulatoryGraph> for RegulatoryGraph {
    fn as_native(&self) -> &biodivine_lib_param_bn::RegulatoryGraph {
        &self.native
    }

    fn as_native_mut(&mut self) -> &mut biodivine_lib_param_bn::RegulatoryGraph {
        &mut self.native
    }
}

impl NetworkVariableContext for RegulatoryGraph {
    fn resolve_network_variable(
        &self,
        variable: &Bound<'_, PyAny>,
    ) -> PyResult<biodivine_lib_param_bn::VariableId> {
        if let Ok(id) = variable.extract::<VariableId>() {
            return if id.__index__() < self.variable_count() {
                Ok(*id.as_native())
            } else {
                throw_index_error(format!("Unknown variable ID `{}`.", id.__index__()))
            };
        }
        if let Ok(name) = variable.extract::<String>() {
            return if let Some(var) = self.as_native().find_variable(name.as_str()) {
                Ok(var)
            } else {
                throw_index_error(format!("Unknown variable name `{}`.", name))
            };
        }
        throw_type_error("Expected `VariableId` or `str`.")
    }

    fn get_network_variable_name(&self, variable: biodivine_lib_param_bn::VariableId) -> String {
        self.as_native().get_variable_name(variable).to_string()
    }
}

#[pymethods]
impl RegulatoryGraph {
    /// To construct a `RegulatoryGraph`, you have to provide:
    ///  - A list of variable names. If this list is not given, it is inferred from the list of regulations.
    ///  - A list of regulations. These can be either `NamedRegulation` dictionaries, or string objects compatible
    ///    with the `.aeon` format notation.
    ///
    /// If you don't provide any arguments, an "empty" `RegulatoryGraph` is constructed with no variables
    /// and no regulations.
    ///
    /// Optionally, you can also provide model annotations as either string, or `ModelAnnotation` object.
    #[new]
    #[pyo3(signature = (variables = None, regulations = None, annotations = None))]
    pub fn new(
        variables: Option<Vec<String>>,
        regulations: Option<&Bound<'_, PyList>>,
        annotations: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<RegulatoryGraph> {
        // First, try to extract regulation data if it is provided.
        let (regulations, inferred_variables) = if let Some(regulations) = regulations.as_ref() {
            let mut data = Vec::new();
            for item in regulations.iter() {
                data.push(Self::resolve_regulation::<RegulatoryGraph>(None, &item)?);
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

        // Then build a regulatory graph using either the given variable names, or the inferred variable names
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

        let ann_data = annotations
            .map(|it| RegulatoryGraph::resolve_annotations(it))
            .transpose()?;

        Ok(RegulatoryGraph {
            native: graph,
            annotations: ann_data,
        })
    }

    fn __str__(&self) -> String {
        format!(
            "RegulatoryGraph(variables={}, regulations={})",
            self.variable_count(),
            self.regulation_count()
        )
    }

    fn __richcmp__(&self, py: Python, other: &Self, op: CompareOp) -> Py<PyAny> {
        richcmp_eq_by_key(py, op, &self, &other, |x| x.as_native())
    }

    fn __repr__(&self, py: Python) -> String {
        let (names, regulations, annotations) = self.__getnewargs__(py);
        // Rust prints Option values differently compared to Python. This way we make sure the value is either a valid string, or `None`.
        let ann_string = annotations
            .map(|it| format!("{:?}", it))
            .unwrap_or_else(|| "None".to_string());
        format!(
            "RegulatoryGraph({:?}, {:?}, {})",
            names, regulations, ann_string
        )
    }

    pub fn __getnewargs__(&self, py: Python) -> (Vec<String>, Vec<String>, Option<String>) {
        let names = self.variable_names();
        let regulations = self
            .as_native()
            .regulations()
            .map(|it| it.to_string(self.as_native()))
            .collect();
        let ann_string = self
            .annotations
            .as_ref()
            .map(|it| it.borrow(py).as_native().to_string());
        (names, regulations, ann_string)
    }

    fn __copy__(&self, py: Python) -> PyResult<RegulatoryGraph> {
        // Note that we have to make a new "deep copy" of the model annotations.
        let ann_copy = self
            .annotations
            .as_ref()
            .map(|it| it.borrow(py).py_copy(py))
            .transpose()?;
        Ok(RegulatoryGraph {
            native: self.as_native().clone(),
            annotations: ann_copy,
        })
    }

    fn __deepcopy__(&self, py: Python, _memo: &Bound<'_, PyAny>) -> PyResult<RegulatoryGraph> {
        self.__copy__(py)
    }

    /// Try to read the structure of a `RegulatoryGraph` from an `.aeon` file at the specified path.
    #[staticmethod]
    fn from_file(py: Python, file_path: &str) -> PyResult<RegulatoryGraph> {
        match std::fs::read_to_string(file_path) {
            Err(e) => throw_runtime_error(format!("Cannot read file {}: `{}`.", file_path, e)),
            Ok(contents) => Self::from_aeon(py, contents.as_str()),
        }
    }

    /// Try to read the structure of a `RegulatoryGraph` from a string representing the contents of an `.aeon` file.
    ///
    /// If the `.aeon` file contains some annotation data, these are stored in `RegulatoryGraph.annotations()`.
    #[staticmethod]
    fn from_aeon(py: Python, file_content: &str) -> PyResult<RegulatoryGraph> {
        let native_graph = biodivine_lib_param_bn::RegulatoryGraph::try_from(file_content)
            .map_err(runtime_error)?;
        let native_annotations =
            biodivine_lib_param_bn::ModelAnnotation::from_model_string(file_content);

        Ok(RegulatoryGraph {
            native: native_graph,
            annotations: Some(Py::new(py, ModelAnnotationRoot::from(native_annotations))?),
        })
    }

    /// Convert this `RegulatoryGraph` to a string representation of a valid `.aeon` file.
    ///
    /// If any annotation data are present (see `RegulatoryGraph.annotations()`), the `.aeon`
    /// file will also contain respective annotation comments.
    fn to_aeon(&self, py: Python) -> String {
        let (_, regulations, _) = self.__getnewargs__(py);
        if let Some(annotations) = self.annotations.as_ref() {
            let ann_string = annotations.borrow(py).as_native().to_string();
            // If annotations are present, print them, leave two lines empty, and then print
            // the regulations.
            format!("{}\n\n{}", ann_string, regulations.join("\n"))
        } else {
            regulations.join("\n")
        }
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
            return Ok(self
                .as_native()
                .find_variable(name.as_str())
                .map(Into::into));
        }
        throw_type_error("Expected `VariableId` or `str`.")
    }

    /// Return the string name of the requested `variable`, or throw `RuntimeError` if
    /// such variable does not exist.
    pub fn get_variable_name(&self, variable: &Bound<'_, PyAny>) -> PyResult<String> {
        let var = self.resolve_network_variable(variable)?;
        Ok(self.as_native().get_variable_name(var).clone())
    }

    /// Update the variable name of the provided `variable`. This does not change the
    /// corresponding `VariableId`.
    ///
    /// The variable is also renamed in the associated `ModelAnnotations`, but any existing
    /// annotation objects (i.e. `ModelAnnotation` or `NetworkVariableAnnotation`) referencing
    /// the renamed annotation become invalid.
    pub fn set_variable_name(
        &mut self,
        py: Python,
        variable: &Bound<'_, PyAny>,
        name: &str,
    ) -> PyResult<()> {
        let var = self.resolve_network_variable(variable)?;

        if let Some(ann) = self.annotations.as_ref() {
            let old_name = self.as_native().get_variable_name(var);
            let mut ann = ann.borrow_mut(py);
            ann.rename_variable(old_name.as_str(), name)?;
        }

        self.as_native_mut()
            .set_variable_name(var, name)
            .map_err(runtime_error)
    }

    /// The number of regulations currently managed by this `RegulatoryGraph`.
    pub fn regulation_count(&self) -> usize {
        self.as_native().regulations().count()
    }

    /// Return the list of all regulations (represented as `IdRegulation` dictionaries) that are currently
    /// managed by this `RegulatoryGraph`.
    pub fn regulations<'a>(&self, py: Python<'a>) -> PyResult<Bound<'a, PyList>> {
        let result = PyList::empty_bound(py);
        for reg in self.as_native().regulations() {
            let reg = Self::encode_regulation(py, reg)?;
            result.append(reg)?;
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
    pub fn find_regulation<'a>(
        &self,
        py: Python<'a>,
        source: &Bound<'_, PyAny>,
        target: &Bound<'_, PyAny>,
    ) -> PyResult<Option<Bound<'a, PyDict>>> {
        let source = self.resolve_network_variable(source)?;
        let target = self.resolve_network_variable(target)?;
        if let Some(regulation) = self.as_native().find_regulation(source, target) {
            Self::encode_regulation(py, regulation).map(Some)
        } else {
            Ok(None)
        }
    }

    /// Add a new regulation to the `RegulatoryGraph`, either using a `NamedRegulation`, `IdRegulation`, or
    /// a string representation compatible with the `.aeon` format.
    ///
    /// Model annotations are not affected by this operation.
    pub fn add_regulation(&mut self, regulation: &Bound<'_, PyAny>) -> PyResult<()> {
        let (s, m, o, t) = Self::resolve_regulation(Some(self), regulation)?;
        let m = m.as_ref().map(|it| match it {
            Positive => Monotonicity::Activation,
            Negative => Monotonicity::Inhibition,
        });
        self.as_native_mut()
            .add_regulation(s.as_str(), t.as_str(), o, m)
            .map_err(runtime_error)
    }

    /// Remove a regulation that is currently present in this `RegulatoryGraph`. Returns the `IdRegulation`
    /// dictionary that represents the removed regulation, or throws a `RuntimeError` if the regulation
    /// does not exist.
    ///
    /// Also removes any annotation data that is associated with the regulation (however, this
    /// data is not returned).
    pub fn remove_regulation<'a>(
        &mut self,
        py: Python<'a>,
        source: &Bound<'a, PyAny>,
        target: &Bound<'a, PyAny>,
    ) -> PyResult<Bound<'a, PyDict>> {
        let source = self.resolve_network_variable(source)?;
        let target = self.resolve_network_variable(target)?;

        // Update annotations.
        if let Some(ann) = self.annotations.as_ref() {
            let source_name = self.as_native().get_variable_name(source);
            let target_name = self.as_native().get_variable_name(target);
            let mut ann = ann.borrow_mut(py);
            ann.remove_regulation(source_name.as_str(), target_name.as_str())?;
        }

        // Remove regulation.
        let removed = self
            .as_native_mut()
            .remove_regulation(source, target)
            .map_err(runtime_error)?;
        Self::encode_regulation(py, &removed)
    }

    /// Update the `sign` and `essential` flags of a regulation in this `RegulatoryGraph`.
    /// If the regulation does not exist, it is created.
    ///
    /// Returns the previous state of the regulation as an `IdRegulation` dictionary, assuming the regulation
    /// already existed.
    ///
    /// Model annotations are not affected by this operation.
    pub fn ensure_regulation<'a>(
        &mut self,
        py: Python<'a>,
        regulation: &Bound<'a, PyAny>,
    ) -> PyResult<Option<Bound<'a, PyDict>>> {
        // This is a bit inefficient, but should be good enough for now.
        let (s, m, o, t) = Self::resolve_regulation(Some(self), regulation)?;
        let source = self.as_native().find_variable(s.as_str()).unwrap();
        let target = self.as_native().find_variable(t.as_str()).unwrap();
        let old = self.as_native_mut().remove_regulation(source, target).ok();
        let m = m.as_ref().map(|it| match it {
            Positive => Monotonicity::Activation,
            Negative => Monotonicity::Inhibition,
        });
        self.as_native_mut()
            .add_regulation(s.as_str(), t.as_str(), o, m)
            .map_err(runtime_error)?;
        old.map(|it| Self::encode_regulation(py, &it)).transpose()
    }

    /// Create a copy of this `RegulatoryGraph` that is extended with the given list of `variables`.
    ///
    /// The new variables are added *after* the existing variables, so any previously used `VariableId` references
    /// are still valid. However, the added names must still be unique within the new graph.
    ///
    /// Model annotations are not affected by this operation (current annotations are copied
    /// into the new network).
    pub fn extend(&self, py: Python, mut variables: Vec<String>) -> PyResult<RegulatoryGraph> {
        let (mut names, regulations, _) = self.__getnewargs__(py);
        names.append(&mut variables);
        let mut result = Self::new(Some(names), None, None)?;
        for reg in regulations {
            result
                .as_native_mut()
                .add_string_regulation(reg.as_str())
                .map_err(runtime_error)?;
        }

        result.annotations = self
            .annotations
            .as_ref()
            .map(|it| it.borrow(py).py_copy(py))
            .transpose()?;
        Ok(result)
    }

    /// Create a copy of this `RegulatoryGraph` with all the specified variables (and their associated regulations)
    /// removed.
    ///
    /// Throws `RuntimeError` if one of the variables does not exist.
    ///
    /// The new graph follows the variable ordering of the old graph, but since there are now variables that are
    /// missing in the new graph, the `VariableId` objects are not compatible with the original graph.
    ///
    /// Annotations for unaffected variables are preserved.
    pub fn drop(&self, py: Python, variables: &Bound<'_, PyAny>) -> PyResult<RegulatoryGraph> {
        let to_remove = self
            .resolve_variables(variables)?
            .into_iter()
            .map(|it| self.as_native().get_variable_name(it).to_string())
            .collect::<HashSet<String>>();
        let names_to_keep = self
            .variable_names()
            .into_iter()
            .filter(|it| !to_remove.contains(it))
            .collect::<Vec<_>>();
        let names_to_drop = self
            .variable_names()
            .into_iter()
            .filter(|it| to_remove.contains(it))
            .collect::<Vec<_>>();
        let ann_copy = self
            .annotations
            .as_ref()
            .map(|it| it.borrow(py).drop_variables(&names_to_drop, py))
            .transpose()?;
        let mut result = Self::new(Some(names_to_keep), None, None)?;
        result.annotations = ann_copy;
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
    /// change its behaviour. And as opposed to `RegulatoryGraph.drop`, the intention of this method is to produce
    /// a result that is functionally compatible with the original regulatory graph. Of course, you can use
    /// `RegulatoryGraph.remove_regulation` to explicitly remove the self-loop before inlining the variable.
    ///
    /// This attempts to merge the annotations of the inlined variable and associated regulations with the
    pub fn inline_variable(
        &self,
        py: Python,
        variable: &Bound<'_, PyAny>,
    ) -> PyResult<RegulatoryGraph> {
        let variable = self.resolve_network_variable(variable)?;
        let old_bn = biodivine_lib_param_bn::BooleanNetwork::new(self.as_native().clone());
        let Some(bn) = old_bn.inline_variable(variable, false) else {
            return throw_runtime_error("Variable has a self-regulation.");
        };
        let name = self.as_native().get_variable_name(variable);
        let ann_copy = self
            .annotations
            .as_ref()
            .map(|it| {
                it.borrow(py)
                    .inline_variable(name.as_str(), old_bn.as_graph(), py)
            })
            .transpose()?;
        Ok(RegulatoryGraph {
            native: bn.as_graph().clone(),
            annotations: ann_copy,
        })
    }

    /// Make a copy of this `RegulatoryGraph` with all constraints on the regulations removed.
    /// In particular, every regulation is set to non-essential with an unknown sign.
    ///
    ///
    /// This does not affect model annotations (if present).
    pub fn remove_regulation_constraints(&self, py: Python) -> PyResult<RegulatoryGraph> {
        let native = self.as_native();
        let bn = biodivine_lib_param_bn::BooleanNetwork::new(native.clone());
        let bn = bn.remove_static_constraints();
        let ann_copy = self
            .annotations
            .as_ref()
            .map(|it| it.borrow(py).py_copy(py))
            .transpose()?;
        Ok(RegulatoryGraph {
            native: bn.as_graph().clone(),
            annotations: ann_copy,
        })
    }

    /// Compute the `set` of all predecessors (regulators) of a specific variable.
    pub fn predecessors(&self, variable: &Bound<'_, PyAny>) -> PyResult<HashSet<VariableId>> {
        let variable = self.resolve_network_variable(variable)?;
        Ok(self
            .as_native()
            .regulators(variable)
            .into_iter()
            .map(VariableId::from)
            .collect())
    }

    /// Compute the `set` of all successors (targets) of a specific variable.
    pub fn successors(&self, variable: &Bound<'_, PyAny>) -> PyResult<HashSet<VariableId>> {
        let variable = self.resolve_network_variable(variable)?;
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
        pivots: &Bound<'_, PyAny>,
        subgraph: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<HashSet<VariableId>> {
        let pivots = self.resolve_variables(pivots)?;
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
        pivots: &Bound<'_, PyAny>,
        subgraph: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<HashSet<VariableId>> {
        let pivots = self.resolve_variables(pivots)?;
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
    /// A feedback vertex set (FVS) is a set of variables which once removed cause the graph to become acyclic.
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
        parity: Option<SignValue>,
        subgraph: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<HashSet<VariableId>> {
        let sd_graph = SdGraph::from(self.as_native());
        let restriction = self.resolve_subgraph(subgraph)?;
        let fvs = if let Some(parity) = parity {
            sd_graph._restricted_parity_feedback_vertex_set(
                &restriction,
                parity.sign(),
                global_log_level(py)?,
                &|| py.check_signals(),
            )?
        } else {
            sd_graph._restricted_feedback_vertex_set(
                &restriction,
                global_log_level(py)?,
                &|| py.check_signals(),
            )?
        };
        Ok(fvs.into_iter().map(VariableId::from).collect())
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
        parity: Option<SignValue>,
        subgraph: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Vec<Vec<VariableId>>> {
        let sd_graph = SdGraph::from(self.as_native());
        let restriction = self.resolve_subgraph(subgraph)?;
        let cycles = if let Some(parity) = parity {
            sd_graph._restricted_independent_parity_cycles(
                &restriction,
                parity.sign(),
                global_log_level(py)?,
                &|| py.check_signals(),
            )?
        } else {
            sd_graph._restricted_independent_cycles(&restriction, global_log_level(py)?, &|| {
                py.check_signals()
            })?
        };
        let cycles = cycles
            .into_iter()
            .map(|cycle| cycle.into_iter().map(VariableId::from).collect::<Vec<_>>())
            .collect();
        Ok(cycles)
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
        subgraph: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Vec<HashSet<VariableId>>> {
        let subgraph = self.resolve_subgraph(subgraph)?;
        let sd_graph = SdGraph::from(self.as_native());
        let components = sd_graph._restricted_strongly_connected_components(
            &subgraph,
            global_log_level(py)?,
            &|| py.check_signals(),
        )?;
        Ok(components
            .into_iter()
            .map(|c| c.into_iter().map(|it| it.into()).collect())
            .collect())
    }

    /// Compute the set of weakly connected components of this `RegulatoryGraph`. Note that typical regulatory graphs
    /// represent a single weakly connected component.
    ///
    /// If the `subgraph` option is specified, only operates on the subgraph induced by these variables.
    #[pyo3(signature = (subgraph = None))]
    pub fn weakly_connected_components(
        &self,
        py: Python,
        subgraph: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Vec<HashSet<VariableId>>> {
        let subgraph = self.resolve_subgraph(subgraph)?;
        let sd_graph = SdGraph::from(self.as_native());
        let components = sd_graph._restricted_weakly_connected_components(
            &subgraph,
            global_log_level(py)?,
            &|| py.check_signals(),
        )?;
        Ok(components
            .into_iter()
            .map(|c| c.into_iter().map(|it| it.into()).collect())
            .collect())
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
        pivot: &Bound<'_, PyAny>,
        parity: Option<SignValue>,
        subgraph: Option<&Bound<'_, PyAny>>,
        length: Option<usize>,
    ) -> PyResult<Option<Vec<VariableId>>> {
        let pivot = self.resolve_network_variable(pivot)?;
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

    /// Get a reference to the underlying `ModelAnnotation` object. If the object does not exist,
    /// it is created. These annotations are preserved when the network is serialized using
    /// the `.aeon` format.
    pub fn raw_annotation(&mut self, py: Python) -> PyResult<ModelAnnotation> {
        if let Some(ann) = self.annotations.as_ref() {
            Ok(ModelAnnotation::from(ann.clone()))
        } else {
            let ann = ModelAnnotation::new(py, None)?;
            self.annotations = Some(ann.to_root());
            Ok(ann)
        }
    }

    /// Get a "typed" `RegulatoryGraphAnnotation` object containing all annotation data officially
    /// supported by AEON.
    pub fn annotation(
        self_: Py<RegulatoryGraph>,
        py: Python,
    ) -> PyResult<Py<RegulatoryGraphAnnotation>> {
        let annotation = self_.borrow_mut(py).raw_annotation(py)?;
        let tuple = (RegulatoryGraphAnnotation::from(self_), annotation);
        Py::new(py, tuple)
    }
}

impl RegulatoryGraph {
    /// Convert an optional value into a set of graph variables. These typically represent an induced subgraph
    /// to which an operation should be applied.
    pub fn resolve_subgraph(
        &self,
        variables: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<HashSet<biodivine_lib_param_bn::VariableId>> {
        let Some(variables) = variables else {
            // If no value is given, we consider the full sub-graph always.
            return Ok(HashSet::from_iter(self.as_native().variables()));
        };

        let mut result = HashSet::new();
        if let Ok(list) = variables.downcast::<PyList>() {
            for item in list {
                result.insert(self.resolve_network_variable(&item)?);
            }
        } else if let Ok(set) = variables.downcast::<PySet>() {
            for item in set {
                result.insert(self.resolve_network_variable(&item)?);
            }
        } else {
            return throw_type_error("Expected `set` or `list`.");
        }

        Ok(result)
    }

    /// A combination of `resolve_variable` and `resolve_subgraph`: Can accept a single variable or a collection
    /// of variables.
    pub fn resolve_variables(
        &self,
        data: &Bound<'_, PyAny>,
    ) -> PyResult<HashSet<biodivine_lib_param_bn::VariableId>> {
        let result = if let Ok(variable) = self.resolve_network_variable(data) {
            HashSet::from_iter([variable])
        } else if let Ok(variables) = self.resolve_subgraph(Some(data)) {
            variables
        } else {
            return throw_type_error("Expected a single variable or a collection of variables.");
        };
        Ok(result)
    }

    /// Extract regulation data from a dynamic `regulation` object. The regulation object can be either a string
    /// (using internal `.aeon` representation), or a dictionary with mandatory keys `source` and `target`, plus
    /// optional keys `essential` and `sign`. For backwards compatibility, the dictionary can also use `observable`
    /// instead of `essential` and `monotonicity` instead of `sign`.
    ///
    /// The function has two modes: If `ctx` is `None`, the function can only read regulation objects where `source`
    /// and `target` are represented using string names. If `ctx` is specified, then these can also be `VariableId`
    /// objects that are resolved to variable names using `ctx`.
    pub fn resolve_regulation<T: NetworkVariableContext>(
        ctx: Option<&T>,
        regulation: &Bound<'_, PyAny>,
    ) -> PyResult<(String, Option<Sign>, bool, String)> {
        if let Ok(item) = regulation.extract::<String>() {
            let Some((source, monotonicity, observable, target)) =
                biodivine_lib_param_bn::Regulation::try_from_string(item.as_str())
            else {
                return throw_runtime_error(format!("Invalid regulation string: `{}`.", item));
            };
            let monotonicity = match monotonicity {
                None => None,
                Some(Monotonicity::Activation) => Some(Positive),
                Some(Monotonicity::Inhibition) => Some(Negative),
            };
            Ok((source, monotonicity, observable, target))
        } else if let Ok(item) = regulation.downcast::<PyDict>() {
            for key in item.keys() {
                let error = match key.extract::<String>() {
                    Ok(name) => match name.as_str() {
                        "source" | "target" | "essential" | "observable" | "sign"
                        | "monotonicity" => continue,
                        _ => name,
                    },
                    Err(_) => key.to_string(),
                };
                return throw_type_error(format!(
                    "Unknown key in the regulation dictionary: {:?}",
                    error
                ));
            }

            let Some(source) = item.get_item("source")? else {
                return throw_type_error("Missing regulation `source` variable.");
            };
            let Some(target) = item.get_item("target")? else {
                return throw_type_error("Missing regulation `target` variable.");
            };

            let (source, target) = if let Some(ctx) = ctx {
                let source = ctx.resolve_network_variable(&source)?;
                let target = ctx.resolve_network_variable(&target)?;
                (
                    ctx.get_network_variable_name(source).clone(),
                    ctx.get_network_variable_name(target).clone(),
                )
            } else {
                let Ok(source) = source.extract::<String>() else {
                    return throw_type_error("Expected string `source` variable.");
                };
                let Ok(target) = target.extract::<String>() else {
                    return throw_type_error("Expected string `target` variable.");
                };
                (source, target)
            };

            let observable = item
                .get_item("essential")?
                .or(item.get_item("observable")?) // backwards compatibility
                .map(|it| it.extract::<BoolLikeValue>().map(bool::from))
                .unwrap_or(Ok(true))?;
            let monotonicity = item
                .get_item("sign")?
                .or(item.get_item("monotonicity")?) // backwards compatibility
                .and_then(|it| if it.is_none() { None } else { Some(it) })
                .map(|it| it.extract::<SignValue>().map(Sign::from))
                .transpose()?;

            Ok((source, monotonicity, observable, target))
        } else {
            throw_type_error("Expected regulation string or regulation dictionary.")
        }
    }

    pub fn resolve_annotations(data: &Bound<'_, PyAny>) -> PyResult<Py<ModelAnnotationRoot>> {
        let py = data.py();
        if let Ok(data) = data.extract::<String>() {
            let native = biodivine_lib_param_bn::ModelAnnotation::from_model_string(data.as_str());
            Py::new(py, ModelAnnotationRoot::from(native))
        } else if let Ok(root) = data.downcast::<ModelAnnotationRoot>() {
            Ok(root.clone().unbind())
        } else if let Ok(ann) = data.downcast::<ModelAnnotation>() {
            Ok(ann.borrow().to_root())
        } else {
            throw_type_error("Expecting annotations string, or instance of ModelAnnotation")
        }
    }

    /// Encode the data from a single regulation into an `IdRegulation`-compatible dictionary.
    pub fn encode_regulation<'a>(
        py: Python<'a>,
        regulation: &biodivine_lib_param_bn::Regulation,
    ) -> PyResult<Bound<'a, PyDict>> {
        let result = PyDict::new_bound(py);
        let source = VariableId::from(regulation.get_regulator());
        let target = VariableId::from(regulation.get_target());
        result.set_item("source", source.into_py(py))?;
        result.set_item("target", target.into_py(py))?;
        result.set_item("essential", regulation.is_observable().into_py(py))?;
        match regulation.get_monotonicity() {
            None => result.set_item("sign", Option::<&str>::None.into_py(py))?,
            Some(Monotonicity::Activation) => result.set_item("sign", "+")?,
            Some(Monotonicity::Inhibition) => result.set_item("sign", "-")?,
        }
        Ok(result)
    }
}
