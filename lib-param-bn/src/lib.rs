extern crate biodivine_bdd;
extern crate biodivine_lib_param_bn;

use biodivine_bdd::BooleanExpression;
use biodivine_lib_param_bn::{FnUpdate, Monotonicity};
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

    /// Get a list of all regulations.
    pub fn regulations(&self, py: Python) -> PyResult<Vec<PyObject>> {
        let mut result = Vec::new();
        for reg in self.0.regulations() {
            let dict = PyDict::new(py);
            dict.set_item("source", VariableId(reg.get_regulator()).into_py(py))?;
            dict.set_item("target", VariableId(reg.get_target()).into_py(py))?;
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

#[pyclass]
#[derive(Clone, Copy)]
pub struct ParameterId(biodivine_lib_param_bn::ParameterId);

impl From<ParameterId> for biodivine_lib_param_bn::ParameterId {
    fn from(value: ParameterId) -> Self {
        value.0
    }
}

impl From<biodivine_lib_param_bn::ParameterId> for ParameterId {
    fn from(value: biodivine_lib_param_bn::ParameterId) -> Self {
        ParameterId(value)
    }
}

#[pymethods]
impl ParameterId {
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self.0))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self.0))
    }
}

#[pyclass]
#[derive(Clone)]
pub struct BooleanNetwork(biodivine_lib_param_bn::BooleanNetwork);

impl From<BooleanNetwork> for biodivine_lib_param_bn::BooleanNetwork {
    fn from(value: BooleanNetwork) -> Self {
        value.0
    }
}

impl From<biodivine_lib_param_bn::BooleanNetwork> for BooleanNetwork {
    fn from(value: biodivine_lib_param_bn::BooleanNetwork) -> Self {
        BooleanNetwork(value)
    }
}

#[pymethods]
impl BooleanNetwork {
    /// Create a new Boolean network with no functions using a regulatory graph.
    #[new]
    pub fn new(graph: RegulatoryGraph) -> BooleanNetwork {
        BooleanNetwork(biodivine_lib_param_bn::BooleanNetwork::new(graph.into()))
    }

    /// Create a Boolean network from an `.sbml` string.
    #[staticmethod]
    pub fn from_sbml(model: String) -> PyResult<BooleanNetwork> {
        let model = biodivine_lib_param_bn::BooleanNetwork::try_from_sbml(model.as_str());
        match model {
            Ok(model) => Ok(model.0.into()),
            Err(e) => Err(PyTypeError::new_err(e)),
        }
    }

    /// Create a Boolean network from a `.bnet` string.
    #[staticmethod]
    pub fn from_bnet(model: String) -> PyResult<BooleanNetwork> {
        let model = biodivine_lib_param_bn::BooleanNetwork::try_from_bnet(model.as_str());
        match model {
            Ok(model) => Ok(model.into()),
            Err(e) => Err(PyTypeError::new_err(e)),
        }
    }

    /// Create a Boolean network from an `.aeon` string.
    #[staticmethod]
    pub fn from_aeon(model: String) -> PyResult<BooleanNetwork> {
        let model: Result<biodivine_lib_param_bn::BooleanNetwork, String> =
            std::convert::TryFrom::try_from(model.as_str());
        match model {
            Ok(model) => Ok(model.into()),
            Err(e) => Err(PyTypeError::new_err(e)),
        }
    }

    /// Write this network into an `.sbml` string.
    pub fn to_sbml(&self) -> String {
        self.0.to_sbml(None)
    }

    /// Write this network into a `.bnet` string.
    pub fn to_bnet(&self) -> PyResult<String> {
        match self.0.to_bnet() {
            Ok(data) => Ok(data),
            Err(error) => Err(PyTypeError::new_err(error)),
        }
    }

    /// Write this network into an `.aeon` string.
    pub fn to_aeon(&self) -> String {
        self.0.to_string()
    }

    /// Get the underlying regulatory graph of this Boolean network.
    pub fn graph(&self) -> RegulatoryGraph {
        self.0.as_graph().clone().into()
    }

    /// Set an update function for the given variable.
    ///
    /// Use `None` to remove update function, otherwise use a string that will be automatically
    /// parsed. Note that the network must contain all used variables and parameters.
    pub fn set_update_function(
        &mut self,
        variable: &PyAny,
        function: Option<String>,
    ) -> PyResult<()> {
        let graph = self.graph();
        let variable = self.graph().find_variable(variable)?;
        let result = if let Some(function) = function {
            let expression = BooleanExpression::parse(function.as_str())?;
            let function = FnUpdate::try_from_expression(expression.into(), &graph.into());
            if let Some(function) = function {
                self.0.set_update_function(variable.into(), Some(function))
            } else {
                Err("Function expression uses unknown variables.".to_string())
            }
        } else {
            self.0.set_update_function(variable.into(), None)
        };

        match result {
            Ok(()) => Ok(()),
            Err(e) => Err(PyTypeError::new_err(e)),
        }
    }

    /// Create a new parameter from a dictionary containing the parameter arity and name.
    pub fn add_parameter(&mut self, parameter: &PyDict) -> PyResult<ParameterId> {
        let name = parameter.get_item("name");
        let name = if let Some(name) = name {
            name.extract::<String>()?
        } else {
            return Err(PyTypeError::new_err("Expected string name."));
        };

        let arity = parameter.get_item("arity");
        let arity = if let Some(arity) = arity {
            arity.extract::<u32>()?
        } else {
            return Err(PyTypeError::new_err("Expected integer arity."));
        };

        let result = self.0.add_parameter(name.as_str(), arity);
        match result {
            Ok(id) => Ok(id.into()),
            Err(e) => Err(PyTypeError::new_err(e)),
        }
    }

    /// Get the number of variables in this network.
    pub fn num_vars(&self) -> usize {
        self.0.num_vars()
    }

    /// Get the number of parameters in this network.
    pub fn num_parameters(&self) -> usize {
        self.0.num_parameters()
    }

    /// Get a list of variable ids in this network.
    pub fn variables(&self) -> Vec<VariableId> {
        self.0.variables().into_iter().map(|it| it.into()).collect()
    }

    /// Get a list of parameter ids in this network.
    pub fn parameters(&self) -> Vec<ParameterId> {
        self.0
            .parameters()
            .into_iter()
            .map(|it| it.into())
            .collect()
    }

    /// Get a string expression representing the update function of the given variable.
    pub fn get_update_function(&self, variable: &PyAny) -> PyResult<Option<String>> {
        let graph = self.graph();
        let variable = graph.find_variable(variable)?;
        let function = self.0.get_update_function(variable.into());
        Ok(function.as_ref().map(|fun| fun.to_string(&self.0)))
    }

    /// Find a `ParameterId` for a parameter of the given name.
    pub fn find_parameter(&self, parameter: &PyAny) -> PyResult<ParameterId> {
        if let Ok(name) = parameter.extract::<String>() {
            let id = self.0.find_parameter(name.as_str());
            if let Some(id) = id {
                Ok(id.into())
            } else {
                Err(PyTypeError::new_err(format!("Parameter {} unknown.", name)))
            }
        } else if let Ok(id) = parameter.extract::<ParameterId>() {
            Ok(id)
        } else {
            Err(PyTypeError::new_err(format!("Expected parameter name.")))
        }
    }
}

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
