use biodivine_lib_param_bn::{Monotonicity, RegulatoryGraph};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use crate::bindings::lib_param_bn::PyVariableId;
use crate::{runtime_error, throw_runtime_error, throw_type_error};
use super::PyRegulatoryGraph;

impl From<PyRegulatoryGraph> for RegulatoryGraph {
    fn from(value: PyRegulatoryGraph) -> Self {
        value.0
    }
}

impl From<RegulatoryGraph> for PyRegulatoryGraph {
    fn from(value: RegulatoryGraph) -> Self {
        PyRegulatoryGraph(value)
    }
}

#[pymethods]
impl PyRegulatoryGraph {
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("RegulatoryGraph(variables = {}, regulations = {})", self.0.num_vars(), self.0.regulations().count()))
    }

    fn __repr__(&self) -> PyResult<String> {
        self.__str__()
    }

    /// Create a new `RegulatoryGraph` using a given list of variable names.
    #[new]
    pub fn new(variables: Vec<String>) -> Self {
        RegulatoryGraph::new(variables).into()
    }

    /// Create a regulatory graph by parsing a list of `.aeon` regulation strings.
    /// The variable names are exactly the ones used by some regulation.
    #[staticmethod]
    pub fn from_regulations(lines: Vec<String>) -> PyResult<Self> {
        let graph = RegulatoryGraph::try_from_string_regulations(lines);
        match graph {
            Ok(graph) => Ok(graph.into()),
            Err(e) => throw_runtime_error(e),
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
                return throw_runtime_error("Missing regulation source.");
            };
            let source = if let Ok(name) = source.extract::<String>() {
                name
            } else {
                let id = source.extract::<PyVariableId>()?;
                self.0.get_variable_name(id.into()).clone()
            };
            let target = dict.get_item("target");
            let target = if let Some(target) = target {
                target
            } else {
                return throw_runtime_error("Missing regulation target.");
            };
            let target = if let Ok(name) = target.extract::<String>() {
                name
            } else {
                let id = target.extract::<PyVariableId>()?;
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
            Err(e) => throw_runtime_error(e),
        }
    }

    /// Find a `VariableId` based on its name.
    pub fn find_variable(&self, variable: &PyAny) -> PyResult<PyVariableId> {
        if let Ok(name) = variable.extract::<String>() {
            let id = self.0.find_variable(name.as_str());
            if let Some(id) = id {
                Ok(id.into())
            } else {
                throw_runtime_error(format!("Variable {} unknown.", name))
            }
        } else if let Ok(id) = variable.extract::<PyVariableId>() {
            Ok(id)
        } else {
            throw_type_error("Expected variable name.")
        }
    }

    /// Get a variable name from its `VariableId`.
    pub fn get_variable_name(&self, id: PyVariableId) -> String {
        self.0.get_variable_name(id.into()).clone()
    }

    /// Set a variable name for the given id.
    pub fn set_variable_name(&mut self, id: PyVariableId, name: &str) -> PyResult<()> {
        self.0
            .set_variable_name(id.0, name)
            .map_err(|error| runtime_error(error))
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
            throw_runtime_error("Unknown regulation.")
        }
    }

    /// Get a list of all regulators that influence the given target (can be a `VariableId` or a
    /// name string).
    pub fn regulators(&self, target: &PyAny) -> PyResult<Vec<PyVariableId>> {
        let target = self.find_variable(target)?;
        let regulators = self.0.regulators(target.into());
        Ok(regulators.into_iter().map(|it| it.into()).collect())
    }

    /// Get a list of all targets that are influenced by the given regulator.
    ///
    /// See also `RegulatoryGraph::regulators`.
    pub fn targets(&self, source: &PyAny) -> PyResult<Vec<PyVariableId>> {
        let source = self.find_variable(source)?;
        let targets = self.0.targets(source.into());
        Ok(targets.into_iter().map(|it| it.into()).collect())
    }

    /// Get a list of all variables which the given `target` depends on, even transitively.
    pub fn transitive_regulators(&self, target: &PyAny) -> PyResult<Vec<PyVariableId>> {
        let id = self.find_variable(target)?;
        let list = self.0.transitive_regulators(id.into());
        Ok(list.into_iter().map(|it| it.into()).collect())
    }

    /// Get a list of all variables which are regulated by the given `target`, even transitively.
    pub fn transitive_targets(&self, source: &PyAny) -> PyResult<Vec<PyVariableId>> {
        let id = self.find_variable(source)?;
        let list = self.0.transitive_targets(id.into());
        Ok(list.into_iter().map(|it| it.into()).collect())
    }

    /// Returns a list of strongly connected components of this graph, where each component is
    /// represented as a list of its variable ids.
    pub fn components(&self) -> Vec<Vec<PyVariableId>> {
        self.0
            .components()
            .into_iter()
            .map(|c| c.into_iter().map(|it| it.into()).collect())
            .collect()
    }

    /// Get a list of all variable ids.
    pub fn variables(&self) -> Vec<PyVariableId> {
        self.0.variables().map(|it| it.into()).collect()
    }

    /// Get a list of all regulations.
    pub fn regulations(&self, py: Python) -> PyResult<Vec<PyObject>> {
        let mut result = Vec::new();
        for reg in self.0.regulations() {
            let dict = PyDict::new(py);
            dict.set_item("source", PyVariableId(reg.get_regulator()).into_py(py))?;
            dict.set_item("target", PyVariableId(reg.get_target()).into_py(py))?;
            if let Some(m) = reg.get_monotonicity() {
                let m = match m {
                    Monotonicity::Activation => "activation",
                    Monotonicity::Inhibition => "inhibition",
                };
                dict.set_item("monotonicity", m)?;
            }
            dict.set_item("observable", reg.is_observable())?;
            result.push(dict.to_object(py));
        }
        Ok(result)
    }
}
