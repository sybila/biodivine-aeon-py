use super::PyRegulatoryGraph;
use crate::bindings::lib_param_bn::PyVariableId;
use crate::{throw_runtime_error, throw_type_error, AsNative};
use biodivine_lib_param_bn::{
    Monotonicity, Regulation, RegulatoryGraph, SdGraph, Sign, VariableId,
};
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::HashSet;

/*
   WARNING: `BooleanNetwork` inherits from `RegulatoryGraph`, so if you are adding new
   methods that can modify the regulatory graph, you should override them in Boolean network
   and ensure both the regulatory graph managed by PyO3 and the regulatory graph *inside*
   the network receive the same updates.

   For now, this only includes `add_regulation` and `set_variable_name`.
*/

#[pymethods]
impl PyRegulatoryGraph {
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

    #[new]
    pub fn new(variables: Vec<String>) -> Self {
        RegulatoryGraph::new(variables).into()
    }

    #[staticmethod]
    pub fn from_regulations(lines: Vec<String>) -> PyResult<Self> {
        let graph = RegulatoryGraph::try_from_string_regulations(lines);
        match graph {
            Ok(graph) => Ok(graph.into()),
            Err(e) => throw_runtime_error(e),
        }
    }

    pub fn add_regulation(&mut self, regulation: &PyAny) -> PyResult<()> {
        if let Ok(string) = regulation.extract::<String>() {
            match self.as_native_mut().add_string_regulation(string.as_str()) {
                Ok(()) => Ok(()),
                Err(e) => throw_runtime_error(e),
            }
        } else if let Ok(dict) = regulation.downcast::<PyDict>() {
            let (source, target, observable, monotonicity) = regulation_from_python(dict)?;
            let source = self.get_variable_name(source)?;
            let target = self.get_variable_name(target)?;

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

    pub fn find_variable(&self, variable: &PyAny) -> PyResult<Option<PyVariableId>> {
        if let Ok(name) = variable.extract::<String>() {
            Ok(self
                .as_native()
                .find_variable(name.as_str())
                .map(|it| it.into()))
        } else if let Ok(id) = variable.extract::<PyVariableId>() {
            Ok(Some(id))
        } else {
            throw_type_error("Expected variable name as argument.")
        }
    }

    pub fn get_variable_name(&self, id: &PyAny) -> PyResult<String> {
        let id = self.find_variable(id)?.expect("Unknown variable.");
        Ok(self.as_native().get_variable_name(id.into()).clone())
    }

    pub fn set_variable_name(&mut self, id: PyVariableId, name: &str) -> PyResult<()> {
        match self.as_native_mut().set_variable_name(id.into(), name) {
            Ok(()) => Ok(()),
            Err(error) => throw_runtime_error(error),
        }
    }

    pub fn num_vars(&self) -> usize {
        self.as_native().num_vars()
    }

    pub fn find_regulation(
        &self,
        py: Python,
        source: &PyAny,
        target: &PyAny,
    ) -> PyResult<Option<PyObject>> {
        let source = self.find_variable(source)?.expect("Unknown variable.");
        let target = self.find_variable(target)?.expect("Unknown variable.");
        if let Some(reg) = self
            .as_native()
            .find_regulation(source.into(), target.into())
        {
            Ok(Some(regulation_to_python(py, reg)?))
        } else {
            Ok(None)
        }
    }

    pub fn regulators(&self, target: &PyAny) -> PyResult<HashSet<PyVariableId>> {
        let target = self.find_variable(target)?.expect("Unknown variable.");
        let regulators = self.0.regulators(target.into());
        Ok(regulators.into_iter().map(|it| it.into()).collect())
    }

    pub fn targets(&self, source: &PyAny) -> PyResult<HashSet<PyVariableId>> {
        let source = self.find_variable(source)?.expect("Unknown variable.");
        let targets = self.as_native().targets(source.into());
        Ok(targets.into_iter().map(|it| it.into()).collect())
    }

    pub fn regulators_transitive(&self, target: &PyAny) -> PyResult<HashSet<PyVariableId>> {
        let id = self.find_variable(target)?.expect("Unknown variable.");
        let list = self.as_native().transitive_regulators(id.into());
        Ok(list.into_iter().map(|it| it.into()).collect())
    }

    pub fn targets_transitive(&self, source: &PyAny) -> PyResult<HashSet<PyVariableId>> {
        let id = self.find_variable(source)?.expect("Unknown variable.");
        let list = self.as_native().transitive_targets(id.into());
        Ok(list.into_iter().map(|it| it.into()).collect())
    }

    pub fn variables(&self) -> Vec<PyVariableId> {
        self.as_native().variables().map(|it| it.into()).collect()
    }

    pub fn regulations(&self, py: Python) -> PyResult<Vec<PyObject>> {
        let mut result = Vec::new();
        for reg in self.as_native().regulations() {
            result.push(regulation_to_python(py, reg)?);
        }
        Ok(result)
    }

    pub fn to_dot(&self) -> String {
        self.as_native().to_dot()
    }

    #[pyo3(signature = (restriction = None))]
    pub fn strongly_connected_components(
        &self,
        restriction: Option<Vec<&PyAny>>,
    ) -> PyResult<Vec<HashSet<PyVariableId>>> {
        let components = if let Some(restriction) = restriction {
            let restriction = read_restriction(self, restriction)?;
            self.as_native()
                .restricted_strongly_connected_components(&restriction)
        } else {
            self.as_native().strongly_connected_components()
        };
        Ok(components
            .into_iter()
            .map(|c| c.into_iter().map(|it| it.into()).collect())
            .collect())
    }

    #[pyo3(signature = (pivot, parity = None))]
    pub fn shortest_cycle(
        &self,
        pivot: &PyAny,
        parity: Option<&str>,
    ) -> PyResult<Option<Vec<PyVariableId>>> {
        let pivot: VariableId = self
            .find_variable(pivot)?
            .expect("Unknown variable.")
            .into();
        let cycle = if let Some(parity) = parity {
            let parity = parse_parity(parity)?;
            self.as_native().shortest_parity_cycle(pivot, parity)
        } else {
            self.as_native().shortest_cycle(pivot)
        };
        Ok(cycle.map(|c| c.into_iter().map(PyVariableId::from).collect()))
    }

    #[pyo3(signature = (parity = None, restriction = None))]
    pub fn feedback_vertex_set(
        &self,
        parity: Option<&str>,
        restriction: Option<Vec<&PyAny>>,
    ) -> PyResult<HashSet<PyVariableId>> {
        let sd_graph = SdGraph::from(self.as_native());
        let restriction = if let Some(restriction) = restriction {
            read_restriction(self, restriction)?
        } else {
            sd_graph.mk_all_vertices()
        };
        let fvs = if let Some(parity) = parity {
            let parity = parse_parity(parity)?;
            sd_graph.restricted_parity_feedback_vertex_set(&restriction, parity)
        } else {
            sd_graph.restricted_feedback_vertex_set(&restriction)
        };
        Ok(fvs.into_iter().map(PyVariableId::from).collect())
    }

    /// Approximate the maximum set of independent cycles of this graph. When optional `parity`
    /// is specified, the method only considers `positive` / `negative` cycles.
    #[pyo3(signature = (parity = None, restriction = None))]
    pub fn independent_cycles(
        &self,
        parity: Option<&str>,
        restriction: Option<Vec<&PyAny>>,
    ) -> PyResult<Vec<Vec<PyVariableId>>> {
        let sd_graph = SdGraph::from(self.as_native());
        let restriction = if let Some(restriction) = restriction {
            read_restriction(self, restriction)?
        } else {
            sd_graph.mk_all_vertices()
        };
        let cycles = if let Some(parity) = parity {
            let parity = parse_parity(parity)?;
            sd_graph.restricted_independent_parity_cycles(&restriction, parity)
        } else {
            sd_graph.restricted_independent_cycles(&restriction)
        };
        let cycles = cycles
            .into_iter()
            .map(|cycle| {
                cycle
                    .into_iter()
                    .map(PyVariableId::from)
                    .collect::<Vec<_>>()
            })
            .collect();
        Ok(cycles)
    }
}

fn parse_parity(parity: &str) -> PyResult<Sign> {
    match parity {
        "positive" | "+" => Ok(Sign::Positive),
        "negative" | "-" => Ok(Sign::Negative),
        _ => throw_runtime_error(format!(
            "Unknown parity. Expected `positive`/`negative`, got {parity}."
        )),
    }
}

fn read_restriction(
    graph: &PyRegulatoryGraph,
    restriction: Vec<&PyAny>,
) -> PyResult<HashSet<VariableId>> {
    let mut set: HashSet<VariableId> = HashSet::new();
    for var in restriction {
        set.insert(graph.find_variable(var)?.expect("Unknown variable.").into());
    }
    Ok(set)
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

pub(crate) fn regulation_from_python(
    dict: &PyDict,
) -> PyResult<(&PyAny, &PyAny, bool, Option<Monotonicity>)> {
    let Some(source) = dict.get_item("source") else {
        return throw_type_error("Missing regulation source variable.");
    };
    let Some(target) = dict.get_item("target") else {
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
            _ => throw_type_error(format!("Unknown monotonicity: {str}")),
        })
        .transpose()?;

    Ok((source, target, observable, monotonicity))
}
