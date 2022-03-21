use crate::bindings::lib_param_bn::{
    PyBooleanNetwork, PyParameterId, PyRegulatoryGraph, PyVariableId,
};
use crate::{throw_runtime_error, throw_type_error, AsNative};
use biodivine_lib_param_bn::BooleanNetwork;
use pyo3::prelude::*;
use pyo3::types::PyDict;

impl From<PyBooleanNetwork> for BooleanNetwork {
    fn from(value: PyBooleanNetwork) -> Self {
        value.0
    }
}

impl From<BooleanNetwork> for PyBooleanNetwork {
    fn from(value: BooleanNetwork) -> Self {
        PyBooleanNetwork(value)
    }
}

impl AsNative<BooleanNetwork> for PyBooleanNetwork {
    fn as_native(&self) -> &BooleanNetwork {
        &self.0
    }

    fn as_native_mut(&mut self) -> &mut BooleanNetwork {
        &mut self.0
    }
}

#[pymethods]
impl PyBooleanNetwork {
    fn __str__(&self) -> PyResult<String> {
        Ok(format!(
            "BooleanNetwork(variables = {}, parameters = {}, regulations = {})",
            self.as_native().num_vars(),
            self.as_native().num_parameters(),
            self.as_native().as_graph().regulations().count(),
        ))
    }

    fn __repr__(&self) -> PyResult<String> {
        self.__str__()
    }

    /// Create a new `BooleanNetwork` with no update functions based on a `RegulatoryGraph`.
    #[new]
    pub fn new(graph: &PyRegulatoryGraph) -> PyBooleanNetwork {
        BooleanNetwork::new(graph.as_native().clone()).into()
    }

    /// Create a `BooleanNetwork` from an `.sbml` model string.
    #[staticmethod]
    pub fn from_sbml(model: String) -> PyResult<PyBooleanNetwork> {
        match BooleanNetwork::try_from_sbml(model.as_str()) {
            Ok((model, _)) => Ok(model.into()),
            Err(e) => throw_runtime_error(e),
        }
    }

    /// Create a `BooleanNetwork` from a `.bnet` model string.
    #[staticmethod]
    pub fn from_bnet(model: String) -> PyResult<PyBooleanNetwork> {
        match BooleanNetwork::try_from_bnet(model.as_str()) {
            Ok(model) => Ok(model.into()),
            Err(e) => throw_runtime_error(e),
        }
    }

    /// Create a `BooleanNetwork` from an `.aeon` model string.
    #[staticmethod]
    pub fn from_aeon(model: String) -> PyResult<PyBooleanNetwork> {
        match BooleanNetwork::try_from(model.as_str()) {
            Ok(model) => Ok(model.into()),
            Err(e) => throw_runtime_error(e),
        }
    }

    /// Convert this `BooleanNetwork` into an `.sbml` model string.
    pub fn to_sbml(&self) -> String {
        self.as_native().to_sbml(None)
    }

    /// Convert this `BooleanNetwork` into a `.bnet` model string.
    ///
    /// Note that this conversion may fail when the network contains uninterpreted functions
    /// or unsupported variable names.
    pub fn to_bnet(&self) -> PyResult<String> {
        match self.as_native().to_bnet() {
            Ok(data) => Ok(data),
            Err(error) => throw_runtime_error(error),
        }
    }

    /// Convert this `BooleanNetwork` into a `.aeon` model string.
    pub fn to_aeon(&self) -> String {
        self.as_native().to_string()
    }

    /// Obtain a copy of the underlying `RegulatoryGraph` of this `BooleanNetwork`.
    pub fn graph(&self) -> PyRegulatoryGraph {
        self.as_native().as_graph().clone().into()
    }

    /// Set an update function for the given network variable.
    ///
    /// Variable can be given either as a name, or as a `VariableId` object.
    ///
    /// An update function is specified as a string that is parsed and interpreted as
    /// a Boolean expression. To remove an update function, use `None` instead of function string.
    /// Keep in mind that the network must contain all variables and parameters used in the
    /// given string expression.
    pub fn set_update_function(
        &mut self,
        variable: &PyAny,
        function: Option<String>,
    ) -> PyResult<()> {
        let id = self.find_variable(variable)?;

        // Clear previous update function.
        self.as_native_mut()
            .set_update_function(id.into(), None)
            .unwrap();

        if let Some(function) = function {
            let name = self.get_variable_name(variable)?;
            match self
                .as_native_mut()
                .add_string_update_function(name.as_str(), function.as_str())
            {
                Ok(()) => Ok(()),
                Err(e) => throw_runtime_error(e),
            }
        } else {
            Ok(())
        }
    }

    /// Create a new parameter from a dictionary containing the parameter `arity` and `name`.
    ///
    /// The function can fail if some of the values are missing/invalid, or when the parameter
    /// already exists.
    pub fn add_parameter(&mut self, parameter: &PyDict) -> PyResult<PyParameterId> {
        let name = parameter.get_item("name");
        let name = if let Some(name) = name {
            name.extract::<String>()?
        } else {
            return throw_type_error("Expected string name.");
        };

        let arity = parameter.get_item("arity");
        let arity = if let Some(arity) = arity {
            arity.extract::<u32>()?
        } else {
            return throw_type_error("Expected integer arity.");
        };

        match self.as_native_mut().add_parameter(name.as_str(), arity) {
            Ok(id) => Ok(id.into()),
            Err(e) => throw_runtime_error(e),
        }
    }

    /// Get the number of variables in this `BooleanNetwork`.
    pub fn num_vars(&self) -> usize {
        self.as_native().num_vars()
    }

    /// Get the number of parameters in this `BooleanNetwork`.
    pub fn num_parameters(&self) -> usize {
        self.as_native().num_parameters()
    }

    /// Get a list of `VariableId` objects representing the variables of this `BooleanNetwork`.
    pub fn variables(&self) -> Vec<PyVariableId> {
        self.as_native()
            .variables()
            .into_iter()
            .map(|it| it.into())
            .collect()
    }

    /// Get a list of `ParameterId` objects representing uninterpreted functions
    /// in this `BooleanNetwork`.
    pub fn parameters(&self) -> Vec<PyParameterId> {
        self.as_native()
            .parameters()
            .into_iter()
            .map(|it| it.into())
            .collect()
    }

    /// Obtain a string expression representing the update function of the given variable.
    ///
    /// Variable can be given as a name, or as `VariableId`. The result is `None` when
    /// the variable has no update function specified.
    pub fn get_update_function(&self, variable: &PyAny) -> PyResult<Option<String>> {
        let variable = self.find_variable(variable)?;
        let function = self.as_native().get_update_function(variable.into());
        Ok(function.as_ref().map(|fun| fun.to_string(self.as_native())))
    }

    /// Find a `ParameterId` for a parameter of the given name.
    ///
    /// For convenience, this function also accepts `ParameterId`, in which case it is returned
    /// unmodified.
    pub fn find_parameter(&self, parameter: &PyAny) -> PyResult<PyParameterId> {
        if let Ok(name) = parameter.extract::<String>() {
            let id = self.as_native().find_parameter(name.as_str());
            if let Some(id) = id {
                Ok(id.into())
            } else {
                throw_runtime_error(format!("Parameter {} unknown.", name))
            }
        } else if let Ok(id) = parameter.extract::<PyParameterId>() {
            Ok(id)
        } else {
            throw_type_error(format!("Expected parameter name."))
        }
    }

    /// Get a parameter name given its `ParameterId`.
    ///
    /// For convenience, the function also accepts a parameter name, in which case it returns it.
    pub fn get_parameter_name(&self, name: &PyAny) -> PyResult<String> {
        let id = self.find_parameter(name)?;
        let param = self.as_native().get_parameter(id.into());
        Ok(param.get_name().clone())
    }

    /// Get a parameter arity given its `ParameterId` or name.
    pub fn get_parameter_arity(&self, name: &PyAny) -> PyResult<u32> {
        let id = self.find_parameter(name)?;
        let param = self.as_native().get_parameter(id.into());
        Ok(param.get_arity())
    }

    /// Get a dictionary representing the properties (`name` and `arity`) of a particular
    /// network parameter.
    ///
    /// The parameter can be given either as a `ParameterId` or using its name.
    pub fn get_parameter(&self, py: Python, name: &PyAny) -> PyResult<PyObject> {
        let id = self.find_parameter(name)?;
        let param = self.as_native().get_parameter(id.into());
        let dict = PyDict::new(py);
        dict.set_item("name", param.get_name().clone())?;
        dict.set_item("arity", param.get_arity())?;
        Ok(dict.to_object(py))
    }

    /// Find a `VariableId` based on a variable name.
    ///
    /// For convenience, this function also accepts `VariableId`, in which case it is returned
    /// unmodified.
    pub fn find_variable(&self, variable: &PyAny) -> PyResult<PyVariableId> {
        if let Ok(name) = variable.extract::<String>() {
            let id = self.as_native().as_graph().find_variable(name.as_str());
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
        match self
            .as_native_mut()
            .as_graph_mut()
            .set_variable_name(id.into(), name)
        {
            Ok(()) => Ok(()),
            Err(error) => throw_runtime_error(error),
        }
    }
}
