use crate::bindings::lib_param_bn::_impl_regulatory_graph::regulation_from_python;
use crate::bindings::lib_param_bn::{
    PyBooleanNetwork, PyFnUpdate, PyParameterId, PyRegulatoryGraph, PyVariableId,
};
use crate::{throw_runtime_error, throw_type_error, AsNative};
use biodivine_lib_param_bn::{BooleanNetwork, RegulatoryGraph};
use pyo3::basic::CompareOp;
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

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Lt => throw_runtime_error("Unsupported operation."),
            CompareOp::Le => throw_runtime_error("Unsupported operation."),
            CompareOp::Eq => Ok(self.as_native() == other.as_native()),
            CompareOp::Ne => Ok(self.as_native() != other.as_native()),
            CompareOp::Gt => throw_runtime_error("Unsupported operation."),
            CompareOp::Ge => throw_runtime_error("Unsupported operation."),
        }
    }

    fn __getstate__(&self) -> String {
        self.to_aeon()
    }

    fn __setstate__(mut self_: PyRefMut<'_, Self>, state: &str) -> PyResult<()> {
        let Ok(model) = BooleanNetwork::try_from(state) else {
            return throw_runtime_error("Invalid serialized network state.");
        };
        let rg: &mut PyRegulatoryGraph = self_.as_mut();
        rg.0 = model.as_graph().clone();
        self_.0 = model;
        Ok(())
    }

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
    #[pyo3(signature = (graph = None))]
    pub fn new(graph: Option<&PyRegulatoryGraph>) -> (PyBooleanNetwork, PyRegulatoryGraph) {
        if let Some(graph) = graph {
            (
                BooleanNetwork::new(graph.as_native().clone()).into(),
                graph.clone(),
            )
        } else {
            (
                BooleanNetwork::new(RegulatoryGraph::new(Vec::default())).into(),
                RegulatoryGraph::new(Vec::default()).into(),
            )
        }
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
        function: Option<&PyAny>,
    ) -> PyResult<()> {
        let id = self_
            .as_ref()
            .find_variable(variable)?
            .expect("Unknown variable.");

        // Clear previous update function.
        self_
            .as_native_mut()
            .set_update_function(id.into(), None)
            .unwrap();

        if let Some(function) = function {
            let result = if let Ok(function) = function.extract::<PyFnUpdate>() {
                self_
                    .as_native_mut()
                    .set_update_function(id.into(), Some(function.into()))
            } else if let Ok(function_str) = function.extract::<String>() {
                let name = self_.as_ref().get_variable_name(variable)?;
                self_
                    .as_native_mut()
                    .add_string_update_function(name.as_str(), function_str.as_str())
            } else {
                return throw_type_error("Expected `FnUpdate` or string.");
            };
            match result {
                Ok(()) => Ok(()),
                Err(e) => throw_runtime_error(e),
            }
        } else {
            Ok(())
        }
    }

    pub fn add_parameter(&mut self, parameter: &PyDict) -> PyResult<PyParameterId> {
        let name = parameter.get_item("name")?;
        let name = if let Some(name) = name {
            name.extract::<String>()?
        } else {
            return throw_type_error("Expected string name.");
        };

        let arity = parameter.get_item("arity")?;
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
        self.as_native().parameters().map(|it| it.into()).collect()
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

    pub fn get_update_function(
        self_: PyRef<'_, Self>,
        variable: &PyAny,
    ) -> PyResult<Option<PyFnUpdate>> {
        let variable = self_
            .as_ref()
            .find_variable(variable)?
            .expect("Unknown variable.");
        let function = self_.as_native().get_update_function(variable.into());
        Ok(function.clone().map(|x| x.into()))
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

    #[pyo3(signature = (infer_inputs = true, repair_graph = true))]
    pub fn inline_inputs(
        &self,
        py: Python,
        infer_inputs: bool,
        repair_graph: bool,
    ) -> PyResult<Py<PyBooleanNetwork>> {
        PyBooleanNetwork::from(self.as_native().inline_inputs(infer_inputs, repair_graph))
            .export_to_python(py)
    }

    /// Eliminate a network variable by inlining its update function into its downstream targets.
    ///
    /// Currently, this method returns `None` if:
    ///
    ///  - The variable has a self-regulation.
    ///  - The function cannot be safely inlined due to the presence of uninterpreted functions.
    ///
    /// Check the Rust documentation for more information about the method.
    #[pyo3(signature = (var, repair_graph = true))]
    pub fn inline_variable(
        self_: PyRefMut<'_, Self>,
        py: Python,
        var: &PyAny,
        repair_graph: bool,
    ) -> PyResult<Option<Py<PyBooleanNetwork>>> {
        let id = self_
            .as_ref()
            .find_variable(var)?
            .expect("Unknown variable.");

        self_
            .as_native()
            .inline_variable(id.into(), repair_graph)
            .map(|it| PyBooleanNetwork::from(it).export_to_python(py))
            .transpose()
    }
}
