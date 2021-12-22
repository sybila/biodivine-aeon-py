extern crate biodivine_bdd;
extern crate biodivine_lib_param_bn;

use biodivine_lib_param_bn::Monotonicity;
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

#[pyclass]
#[derive(Clone, Copy)]
pub struct VariableId(biodivine_lib_param_bn::VariableId);

impl From<VariableId> for biodivine_lib_param_bn::VariableId {
    fn from(value: VariableId) -> Self {
        value.0
    }
}

impl From<biodivine_lib_param_bn::VariableId> for VariableId {
    fn from(value: biodivine_lib_param_bn::VariableId) -> Self {
        VariableId(value)
    }
}

#[pymethods]
impl VariableId {
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{}", self.0))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{}", self.0))
    }
}

#[pyclass]
#[derive(Clone)]
pub struct RegulatoryGraph(biodivine_lib_param_bn::RegulatoryGraph);

impl From<RegulatoryGraph> for biodivine_lib_param_bn::RegulatoryGraph {
    fn from(value: RegulatoryGraph) -> Self {
        value.0
    }
}

impl From<biodivine_lib_param_bn::RegulatoryGraph> for RegulatoryGraph {
    fn from(value: biodivine_lib_param_bn::RegulatoryGraph) -> Self {
        RegulatoryGraph(value)
    }
}

#[pymethods]
impl RegulatoryGraph {
    /// Create a new `RegulatoryGraph` using a given list of variable names.
    #[new]
    pub fn new(variables: Vec<String>) -> Self {
        biodivine_lib_param_bn::RegulatoryGraph::new(variables).into()
    }

    /// Create a regulatory graph by parsing a list of `.aeon` regulation strings.
    /// The variable names are exactly the ones used by some regulation.
    #[staticmethod]
    pub fn from_regulations(lines: Vec<String>) -> PyResult<Self> {
        let graph = biodivine_lib_param_bn::RegulatoryGraph::try_from_string_regulations(lines);
        match graph {
            Ok(graph) => Ok(graph.into()),
            Err(e) => Err(PyTypeError::new_err(e)),
        }
    }

    /// Create a new regulation in this graph with the given parameters.
    ///
    /// The parameter can be either a string, in which case it is parsed as one regulation
    /// line in the .aeon format, or it can be a dictionary, in which case the function will
    /// read `source`, `target`, `monotonicity` and `observable` from the dictionary.
    ///
    /// Values `source` and `target` are mandatory and can be either names (strings) or `VariableId`
    /// objects. Monotonicity is optional, and can be either `'activation'` or `'inhibition'` (defaults
    /// to none). Observability is boolean, but is also optional (in which case it defaults to
    /// true).
    pub fn add_regulation(&mut self, regulation: &PyAny) -> PyResult<()> {
        let error = if let Ok(string) = regulation.extract::<String>() {
            self.0.add_string_regulation(string.as_str())
        } else {
            let dict = regulation.cast_as::<PyDict>()?;
            let source = dict.get_item("source");
            let source = if let Some(source) = source {
                source
            } else {
                return Err(PyTypeError::new_err("Missing regulation source."));
            };
            let source = if let Ok(name) = source.extract::<String>() {
                name
            } else {
                let id = source.extract::<VariableId>()?;
                self.0.get_variable_name(id.into()).clone()
            };
            let target = dict.get_item("target");
            let target = if let Some(target) = target {
                target
            } else {
                return Err(PyTypeError::new_err("Missing regulation target."));
            };
            let target = if let Ok(name) = target.extract::<String>() {
                name
            } else {
                let id = target.extract::<VariableId>()?;
                self.0.get_variable_name(id.into()).clone()
            };
            let monotonicity = dict.get_item("monotonicity");
            let monotonicity = if let Some(monotonicity) = monotonicity {
                let string = monotonicity.extract::<String>()?;
                if string == "activation" {
                    Some(Monotonicity::Activation)
                } else if string == "inhibition" {
                    Some(Monotonicity::Inhibition)
                } else {
                    None
                }
            } else {
                None
            };
            let observable = dict.get_item("observable");
            let observable = if let Some(observable) = observable {
                observable.extract::<bool>()?
            } else {
                false
            };
            self.0
                .add_regulation(source.as_str(), target.as_str(), observable, monotonicity)
        };

        match error {
            Ok(()) => Ok(()),
            Err(e) => Err(PyTypeError::new_err(e)),
        }
    }

    /// Find a `VariableId` based on its name.
    pub fn find_variable(&self, variable: &PyAny) -> PyResult<VariableId> {
        if let Ok(name) = variable.extract::<String>() {
            let id = self.0.find_variable(name.as_str());
            if let Some(id) = id {
                Ok(id.into())
            } else {
                Err(PyTypeError::new_err(format!("Variable {} unknown.", name)))
            }
        } else if let Ok(id) = variable.extract::<VariableId>() {
            Ok(id)
        } else {
            Err(PyTypeError::new_err(format!("Expected variable name.")))
        }
    }

    /// Get a variable name from its `VariableId`.
    pub fn get_variable_name(&self, id: VariableId) -> String {
        self.0.get_variable_name(id.into()).clone()
    }

    /// Get the number of variables in this regulatory graph.
    pub fn num_vars(&self) -> usize {
        self.0.num_vars()
    }

    /// Find information about a regulation in the graph if it exists (throws an
    /// exception otherwise)
    ///
    /// You have to specify a source and a target, both of which can be either a variable id,
    /// or a string name.
    pub fn find_regulation(
        &self,
        py: Python,
        source: &PyAny,
        target: &PyAny,
    ) -> PyResult<PyObject> {
        let source = self.find_variable(source)?;
        let target = self.find_variable(target)?;
        if let Some(reg) = self.0.find_regulation(source.into(), target.into()) {
            let dict = PyDict::new(py);
            dict.set_item("source", source.into_py(py))?;
            dict.set_item("target", target.into_py(py))?;
            if let Some(m) = reg.get_monotonicity() {
                let m = match m {
                    Monotonicity::Activation => "activation",
                    Monotonicity::Inhibition => "inhibition",
                };
                dict.set_item("monotonicity", m)?;
            }
            dict.set_item("observable", reg.is_observable())?;
            Ok(dict.to_object(py))
        } else {
            Err(PyTypeError::new_err("Unknown regulation."))
        }
    }

    /// Get a list of all regulators that influence the given target (can be a `VariableId` or a
    /// name string).
    pub fn regulators(&self, target: &PyAny) -> PyResult<Vec<VariableId>> {
        let target = self.find_variable(target)?;
        let regulators = self.0.regulators(target.into());
        Ok(regulators.into_iter().map(|it| it.into()).collect())
    }

    /// Get a list of all targets that are influenced by the given regulator.
    ///
    /// See also `RegulatoryGraph::regulators`.
    pub fn targets(&self, source: &PyAny) -> PyResult<Vec<VariableId>> {
        let source = self.find_variable(source)?;
        let targets = self.0.targets(source.into());
        Ok(targets.into_iter().map(|it| it.into()).collect())
    }

    /// Get a list of all variables which the given `target` depends on, even transitively.
    pub fn transitive_regulators(&self, target: &PyAny) -> PyResult<Vec<VariableId>> {
        let id = self.find_variable(target)?;
        let list = self.0.transitive_regulators(id.into());
        Ok(list.into_iter().map(|it| it.into()).collect())
    }

    /// Get a list of all variables which are regulated by the given `target`, even transitively.
    pub fn transitive_targets(&self, source: &PyAny) -> PyResult<Vec<VariableId>> {
        let id = self.find_variable(source)?;
        let list = self.0.transitive_targets(id.into());
        Ok(list.into_iter().map(|it| it.into()).collect())
    }

    /// Returns a list of strongly connected components of this graph, where each component is
    /// represented as a list of its variable ids.
    pub fn components(&self) -> Vec<Vec<VariableId>> {
        self.0
            .components()
            .into_iter()
            .map(|c| c.into_iter().map(|it| it.into()).collect())
            .collect()
    }

    /// Get a list of all variable ids.
    pub fn variables(&self) -> Vec<VariableId> {
        self.0.variables().map(|it| it.into()).collect()
    }
}

#[pyclass]
#[derive(Clone)]
pub struct BooleanNetwork(biodivine_lib_param_bn::BooleanNetwork);

#[pyclass]
#[derive(Clone)]
pub struct SymbolicAsyncGraph(biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph);

/// A Python module implemented in Rust.
#[pymodule]
fn biodivine_boolean_networks(_py: Python, module: &PyModule) -> PyResult<()> {
    module.add_class::<RegulatoryGraph>()?;
    module.add_class::<BooleanNetwork>()?;
    module.add_class::<SymbolicAsyncGraph>()?;
    Ok(())
}
