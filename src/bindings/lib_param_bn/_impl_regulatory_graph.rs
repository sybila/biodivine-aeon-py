use super::PyRegulatoryGraph;
use crate::bindings::lib_param_bn::PyVariableId;
use crate::{throw_runtime_error, throw_type_error, AsNative};
use biodivine_lib_param_bn::{Monotonicity, Regulation, RegulatoryGraph};
use pyo3::prelude::*;
use pyo3::types::PyDict;

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

impl AsNative<RegulatoryGraph> for PyRegulatoryGraph {
    fn as_native(&self) -> &RegulatoryGraph {
        &self.0
    }
    fn as_native_mut(&mut self) -> &mut RegulatoryGraph {
        &mut self.0
    }
}

#[pymethods]
impl PyRegulatoryGraph {
    fn __str__(&self) -> PyResult<String> {
        Ok(format!(
            "RegulatoryGraph(variables = {}, regulations = {})",
            self.as_native().num_vars(),
            self.as_native().regulations().count()
        ))
    }

    fn __repr__(&self) -> PyResult<String> {
        self.__str__()
    }

    /// Create a new `RegulatoryGraph` using a given list of variable names.
    #[new]
    pub fn new(variables: Vec<String>) -> Self {
        RegulatoryGraph::new(variables).into()
    }

    /// Create a `RegulatoryGraph` by parsing a list of `.aeon` regulation strings.
    ///
    /// The variables created in the graph are exactly the ones used by some regulation
    /// (i.e. they do not need to be declared separately).
    #[staticmethod]
    pub fn from_regulations(lines: Vec<String>) -> PyResult<Self> {
        let graph = RegulatoryGraph::try_from_string_regulations(lines);
        match graph {
            Ok(graph) => Ok(graph.into()),
            Err(e) => throw_runtime_error(e),
        }
    }

    /// Create a new regulation in this `RegulatoryGraph` with the given properties.
    ///
    /// The function takes one argument. When this argument is a string, the string is interpreted
    /// as a regulation line in the AEON text format. Alternatively, the argument can be
    /// a dictionary, in which case the function constructs a regulation based on `source`,
    /// `target`, `monotonicity`, and `observable` keys from this dictionary.
    ///
    /// Both `source` and `target` are either names or `VariableId` objects. Furthermore,
    /// the `observable` key is a Boolean, and `monotonicity` is either `activation`, `inhibition`
    /// (i.e. strings), or is missing (i.e. the value is `None`).
    ///
    /// Both `source` and `target` are mandatory, `observability` is optional and defaults to
    /// `True`, and `monotonicity` is also optional, defaulting to non-monotonous.
    pub fn add_regulation(&mut self, regulation: &PyAny) -> PyResult<()> {
        if let Ok(string) = regulation.extract::<String>() {
            match self.as_native_mut().add_string_regulation(string.as_str()) {
                Ok(()) => Ok(()),
                Err(e) => throw_runtime_error(e),
            }
        } else if let Ok(dict) = regulation.cast_as::<PyDict>() {
            let source = if let Some(source) = dict.get_item("source") {
                self.get_variable_name(source)?
            } else {
                return throw_type_error("Missing regulation source variable.");
            };
            let target = if let Some(target) = dict.get_item("target") {
                self.get_variable_name(target)?
            } else {
                return throw_type_error("Missing regulation target variable.");
            };
            let observable = dict
                .get_item("observable")
                .map(|it| it.extract::<bool>())
                .unwrap_or(Ok(true))?;
            let monotonicity = dict
                .get_item("monotonicity")
                .map(|it| it.extract::<String>())
                .transpose()?;
            let monotonicity = monotonicity
                .map(|str| match str.as_str() {
                    "activation" => Ok(Monotonicity::Activation),
                    "inhibition" => Ok(Monotonicity::Inhibition),
                    _ => throw_type_error(format!("Unknown monotonicity: {}", str)),
                })
                .transpose()?;

            let result = self.as_native_mut().add_regulation(
                source.as_str(),
                target.as_str(),
                observable,
                monotonicity,
            );

            match result {
                Ok(()) => Ok(()),
                Err(e) => throw_runtime_error(e),
            }
        } else {
            throw_type_error("Expected string or dictionary as argument.")
        }
    }

    /// Find a `VariableId` based on a variable name.
    ///
    /// For convenience, this function also accepts `VariableId`, in which case it is returned
    /// unmodified.
    pub fn find_variable(&self, variable: &PyAny) -> PyResult<PyVariableId> {
        if let Ok(name) = variable.extract::<String>() {
            let id = self.as_native().find_variable(name.as_str());
            if let Some(id) = id {
                Ok(id.into())
            } else {
                throw_runtime_error(format!("Variable {} unknown.", name))
            }
        } else if let Ok(id) = variable.extract::<PyVariableId>() {
            Ok(id)
        } else {
            throw_type_error("Expected variable name as argument.")
        }
    }

    /// Get a variable given its `VariableId`.
    ///
    /// For convenience, the function also accepts a variable name and returns the same name.
    pub fn get_variable_name(&self, id: &PyAny) -> PyResult<String> {
        let id = self.find_variable(id)?;
        Ok(self.as_native().get_variable_name(id.into()).clone())
    }

    /// Set the variable name of a given `VariableId`.
    ///
    /// Throws an exception if the name is invalid or already in use.
    pub fn set_variable_name(&mut self, id: PyVariableId, name: &str) -> PyResult<()> {
        match self.as_native_mut().set_variable_name(id.into(), name) {
            Ok(()) => Ok(()),
            Err(error) => throw_runtime_error(error),
        }
    }

    /// Get the number of variables in this regulatory graph.
    pub fn num_vars(&self) -> usize {
        self.as_native().num_vars()
    }

    /// Find information about a regulation in the graph if it exists. If the regulation is not
    /// found, throws a runtime exception.
    ///
    /// The regulation is specified by its `source` and `target`, both of which are either names
    /// or `VariableId` objects. The result is a dictionary with the `source` and `target` key
    /// giving the `VariableId` of the respective entities. Furthermore, `observable` key states
    /// the observability of the regulation (value is Boolean), and `monotonicity` key states
    /// whether the regulation is an `activation` or an `inhibition`. If the regulation is
    /// not monotonous, the key is undefined.
    pub fn find_regulation(
        &self,
        py: Python,
        source: &PyAny,
        target: &PyAny,
    ) -> PyResult<PyObject> {
        let source = self.find_variable(source)?;
        let target = self.find_variable(target)?;
        if let Some(reg) = self
            .as_native()
            .find_regulation(source.into(), target.into())
        {
            Ok(regulation_to_python(py, reg)?)
        } else {
            throw_runtime_error("Unknown regulation.")
        }
    }

    /// Get a list of all regulators that influence the given `target`.
    ///
    /// Target can be a variable name or `VariableId`. Result is always a list of `VariableId`
    /// objects. Note that the entities may be returned in arbitrary order.
    pub fn regulators(&self, target: &PyAny) -> PyResult<Vec<PyVariableId>> {
        let target = self.find_variable(target)?;
        let regulators = self.0.regulators(target.into());
        Ok(regulators.into_iter().map(|it| it.into()).collect())
    }

    /// Get a list of all targets that are influenced by the given `source` regulator.
    ///
    /// Source can be a variable name or `VariableId`. Result is always a list of `VariableId`
    /// objects. Note that the entities may be returned in arbitrary order.
    pub fn targets(&self, source: &PyAny) -> PyResult<Vec<PyVariableId>> {
        let source = self.find_variable(source)?;
        let targets = self.as_native().targets(source.into());
        Ok(targets.into_iter().map(|it| it.into()).collect())
    }

    /// Get a list of all variables that the given `target` depends on, even transitively.
    ///
    /// Target can be a variable name or `VariableId`. Result is always a list of `VariableId`
    /// objects. Note that the entities may be returned in arbitrary order.
    pub fn transitive_regulators(&self, target: &PyAny) -> PyResult<Vec<PyVariableId>> {
        let id = self.find_variable(target)?;
        let list = self.0.transitive_regulators(id.into());
        Ok(list.into_iter().map(|it| it.into()).collect())
    }

    /// Get a list of all variables that are regulated by the given `source`, even transitively.
    ///
    /// Source can be a variable name or `VariableId`. Result is always a list of `VariableId`
    /// objects. Note that the entities may be returned in arbitrary order.
    pub fn transitive_targets(&self, source: &PyAny) -> PyResult<Vec<PyVariableId>> {
        let id = self.find_variable(source)?;
        let list = self.as_native().transitive_targets(id.into());
        Ok(list.into_iter().map(|it| it.into()).collect())
    }

    /// Returns a list of strongly connected components of this `RegulatoryGraph`, where each
    /// component is represented as a list of its nodes (`VariableId` objects).
    pub fn components(&self) -> Vec<Vec<PyVariableId>> {
        self.as_native()
            .components()
            .into_iter()
            .map(|c| c.into_iter().map(|it| it.into()).collect())
            .collect()
    }

    /// Get a list of all variables represented by their `VariableId` objects.
    pub fn variables(&self) -> Vec<PyVariableId> {
        self.as_native().variables().map(|it| it.into()).collect()
    }

    /// Get a list of all regulations in this `RegulatoryGraph`.
    ///
    /// Each member of the list is a dictionary with the `source` and `target` key giving
    /// the `VariableId` of the respective entities. Furthermore, `observable` key states
    /// the observability of the regulation (value is Boolean), and `monotonicity` key states
    /// whether the regulation is an `activation` or an `inhibition`. If the regulation is
    /// not monotonous, the key is undefined.
    pub fn regulations(&self, py: Python) -> PyResult<Vec<PyObject>> {
        let mut result = Vec::new();
        for reg in self.as_native().regulations() {
            result.push(regulation_to_python(py, reg)?);
        }
        Ok(result)
    }
}

fn regulation_to_python(py: Python, reg: &Regulation) -> PyResult<PyObject> {
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
    Ok(dict.to_object(py))
}
