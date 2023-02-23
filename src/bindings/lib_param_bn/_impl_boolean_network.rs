use crate::bindings::lib_param_bn::_impl_regulatory_graph::regulation_from_python;
use crate::bindings::lib_param_bn::{
    PyBooleanNetwork, PyParameterId, PyRegulatoryGraph, PyVariableId,
};
use crate::{throw_runtime_error, throw_type_error, AsNative};
use biodivine_lib_param_bn::BooleanNetwork;
use pyo3::prelude::*;
use pyo3::types::PyDict;

impl PyBooleanNetwork {
    /// Export a `PyBooleanNetwork` to something PyO3 will accept because it respects
    /// the class inheritance hierarchy.
    pub fn export_to_python(self, py: Python) -> PyResult<Py<PyBooleanNetwork>> {
        let graph = self.as_native().as_graph().clone();
        let tuple = (self, PyRegulatoryGraph::from(graph));
        Py::new(py, tuple)
    }
}

#[pymethods]
impl PyBooleanNetwork {
    /*
       These methods override the methods in `RegulatoryGraph` to ensure consistency between
       the "internal" regulatory graph in the Boolean network and the "external" graph managed
       through inheritance in PyO3.
    */

    pub fn add_regulation(mut self_: PyRefMut<'_, Self>, regulation: &PyAny) -> PyResult<()> {
        let rg: &mut PyRegulatoryGraph = self_.as_mut();
        rg.add_regulation(regulation)?;

        // This is basically a copy of the method in RegulatoryGraph, but applied to
        // the BooleanNetwork.
        if let Ok(string) = regulation.extract::<String>() {
            match self_
                .as_native_mut()
                .as_graph_mut()
                .add_string_regulation(string.as_str())
            {
                Ok(()) => Ok(()),
                Err(e) => throw_runtime_error(e),
            }
        } else if let Ok(dict) = regulation.downcast::<PyDict>() {
            let (source, target, observable, monotonicity) = regulation_from_python(dict)?;
            let source = self_.as_ref().get_variable_name(source)?;
            let target = self_.as_ref().get_variable_name(target)?;

            let result = self_.as_native_mut().as_graph_mut().add_regulation(
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

    pub fn set_variable_name(
        mut self_: PyRefMut<'_, Self>,
        id: PyVariableId,
        name: &str,
    ) -> PyResult<()> {
        if let Err(error) = self_
            .as_native_mut()
            .as_graph_mut()
            .set_variable_name(id.into(), name)
        {
            return throw_runtime_error(error);
        }

        let rg: &mut PyRegulatoryGraph = self_.as_mut();
        rg.set_variable_name(id, name)
    }

    /*
       The rest is the actual "normal" implementation of `BooleanNetwork`.
    */

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

    #[new]
    pub fn new(graph: &PyRegulatoryGraph) -> (PyBooleanNetwork, PyRegulatoryGraph) {
        (
            BooleanNetwork::new(graph.as_native().clone()).into(),
            graph.clone(),
        )
    }

    #[staticmethod]
    pub fn from_sbml(py: Python, model: String) -> PyResult<Py<PyBooleanNetwork>> {
        match BooleanNetwork::try_from_sbml(model.as_str()) {
            Ok((model, _)) => PyBooleanNetwork::from(model).export_to_python(py),
            Err(e) => throw_runtime_error(e),
        }
    }

    #[staticmethod]
    pub fn from_bnet(py: Python, model: String) -> PyResult<Py<PyBooleanNetwork>> {
        match BooleanNetwork::try_from_bnet(model.as_str()) {
            Ok(model) => PyBooleanNetwork::from(model).export_to_python(py),
            Err(e) => throw_runtime_error(e),
        }
    }

    #[staticmethod]
    pub fn from_aeon(py: Python, model: String) -> PyResult<Py<PyBooleanNetwork>> {
        match BooleanNetwork::try_from(model.as_str()) {
            Ok(model) => PyBooleanNetwork::from(model).export_to_python(py),
            Err(e) => throw_runtime_error(e),
        }
    }

    #[staticmethod]
    pub fn from_file(py: Python, path: String) -> PyResult<Py<PyBooleanNetwork>> {
        match BooleanNetwork::try_from_file(path) {
            Ok(model) => PyBooleanNetwork::from(model).export_to_python(py),
            Err(e) => throw_runtime_error(e),
        }
    }

    pub fn to_sbml(&self) -> String {
        self.as_native().to_sbml(None)
    }

    pub fn to_bnet(&self, rename_if_necessary: Option<bool>) -> PyResult<String> {
        let rename_if_necessary = rename_if_necessary.unwrap_or(false);
        match self.as_native().to_bnet(rename_if_necessary) {
            Ok(data) => Ok(data),
            Err(error) => throw_runtime_error(error),
        }
    }

    pub fn to_aeon(&self) -> String {
        self.as_native().to_string()
    }

    pub fn graph(&self) -> PyRegulatoryGraph {
        self.as_native().as_graph().clone().into()
    }

    pub fn set_update_function(
        mut self_: PyRefMut<'_, Self>,
        variable: &PyAny,
        function: Option<String>,
    ) -> PyResult<()> {
        let id = self_.as_ref().find_variable(variable)?.expect("Unknown variable.");

        // Clear previous update function.
        self_
            .as_native_mut()
            .set_update_function(id.into(), None)
            .unwrap();

        if let Some(function) = function {
            let name = self_.as_ref().get_variable_name(variable)?;
            match self_
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

    pub fn num_parameters(&self) -> usize {
        self.as_native().num_parameters()
    }

    pub fn num_implicit_parameters(&self) -> usize {
        self.as_native().num_implicit_parameters()
    }

    pub fn parameters(&self) -> Vec<PyParameterId> {
        self.as_native()
            .parameters()
            .into_iter()
            .map(|it| it.into())
            .collect()
    }

    pub fn implicit_parameters(&self) -> Vec<PyVariableId> {
        self.as_native()
            .implicit_parameters()
            .into_iter()
            .map(PyVariableId::from)
            .collect()
    }

    pub fn parameter_appears_in(&self, parameter: &PyAny) -> PyResult<Vec<PyVariableId>> {
        let parameter = self.find_parameter(parameter)?.expect("Unknown parameter.");
        Ok(self
            .as_native()
            .parameter_appears_in(parameter.into())
            .into_iter()
            .map(PyVariableId::from)
            .collect())
    }

    pub fn get_update_function(self_: PyRef<'_, Self>, variable: &PyAny) -> PyResult<Option<String>> {
        let variable = self_.as_ref().find_variable(variable)?.expect("Unknown variable.");
        let function = self_.as_native().get_update_function(variable.into());
        Ok(function.as_ref().map(|fun| fun.to_string(self_.as_native())))
    }

    pub fn find_parameter(&self, parameter: &PyAny) -> PyResult<Option<PyParameterId>> {
        if let Ok(name) = parameter.extract::<String>() {
            Ok(self
                .as_native()
                .find_parameter(name.as_str())
                .map(|it| it.into()))
        } else if let Ok(id) = parameter.extract::<PyParameterId>() {
            Ok(Some(id))
        } else {
            throw_type_error("Expected parameter name.".to_string())
        }
    }

    pub fn get_parameter_name(&self, name: &PyAny) -> PyResult<String> {
        let id = self.find_parameter(name)?.expect("Unknown parameter.");
        let param = self.as_native().get_parameter(id.into());
        Ok(param.get_name().clone())
    }

    pub fn get_parameter_arity(&self, name: &PyAny) -> PyResult<u32> {
        let id = self.find_parameter(name)?.expect("Unknown parameter.");
        let param = self.as_native().get_parameter(id.into());
        Ok(param.get_arity())
    }

    pub fn get_parameter(&self, py: Python, name: &PyAny) -> PyResult<PyObject> {
        let id = self.find_parameter(name)?.expect("Unknown parameter.");
        let param = self.as_native().get_parameter(id.into());
        let dict = PyDict::new(py);
        dict.set_item("name", param.get_name().clone())?;
        dict.set_item("arity", param.get_arity())?;
        Ok(dict.to_object(py))
    }

    pub fn infer_regulatory_graph(&self, py: Python) -> PyResult<Py<PyBooleanNetwork>> {
        match self.as_native().infer_valid_graph() {
            Ok(bn) => PyBooleanNetwork(bn).export_to_python(py),
            Err(error) => throw_runtime_error(error),
        }
    }

    pub fn inline_inputs(&self, py: Python) -> PyResult<Py<PyBooleanNetwork>> {
        PyBooleanNetwork::from(self.as_native().inline_inputs()).export_to_python(py)
    }
}
