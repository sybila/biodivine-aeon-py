use crate::bindings::lib_bdd::PyBdd;
use crate::bindings::lib_param_bn::{
    PyFnUpdate, PyGraphColoredVertices, PyGraphColors, PyGraphVertices, PySymbolicAsyncGraph,
    PySymbolicProjection, PyVariableId,
};
use crate::{throw_runtime_error, throw_type_error, AsNative};
use biodivine_lib_bdd::Bdd;
use biodivine_lib_param_bn::symbolic_async_graph::projected_iteration::MixedProjection;
use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
use biodivine_lib_param_bn::VariableId;
use pyo3::prelude::*;

type StateData = Vec<(PyVariableId, bool)>;
type FunctionData = Vec<(PyVariableId, PyFnUpdate)>;

#[pymethods]
impl PySymbolicProjection {
    #[new]
    #[pyo3(signature = (graph, symbolic_set, retained_variables = None, retained_functions = None))]
    pub fn new(
        graph: &PySymbolicAsyncGraph,
        symbolic_set: &PyAny,
        retained_variables: Option<Vec<&PyAny>>,
        retained_functions: Option<Vec<&PyAny>>,
    ) -> PyResult<PySymbolicProjection> {
        let retained_variables = retained_variables.unwrap_or_default();

        let mut variables: Vec<VariableId> = Vec::new();
        for retained in retained_variables {
            variables.push(graph.resolve_variable(retained)?.into());
        }

        let retained_functions = retained_functions.unwrap_or_default();

        let mut functions: Vec<VariableId> = Vec::new();
        for retained in retained_functions {
            functions.push(graph.resolve_variable(retained)?.into());
        }

        let symbolic_set: Bdd = if let Ok(set) = symbolic_set.extract::<PyGraphColoredVertices>() {
            set.as_native().as_bdd().clone()
        } else if let Ok(set) = symbolic_set.extract::<PyGraphVertices>() {
            if !functions.is_empty() {
                return throw_runtime_error("Cannot project to functions in a vertex set.");
            }
            set.as_native().as_bdd().clone()
        } else if let Ok(set) = symbolic_set.extract::<PyGraphColors>() {
            if !variables.is_empty() {
                return throw_runtime_error("Cannot project to variables in a color set.");
            }
            set.as_native().as_bdd().clone()
        } else if let Ok(set) = symbolic_set.extract::<PyBdd>() {
            set.as_native().clone()
        } else {
            return throw_type_error("Expected symbolic set.");
        };

        let boxed_set = Box::new(symbolic_set);
        let boxed_graph = Box::new(graph.as_native().clone());

        let set_ref = unsafe { (boxed_set.as_ref() as *const Bdd).as_ref().unwrap() };

        let graph_ref = unsafe {
            (boxed_graph.as_ref() as *const SymbolicAsyncGraph)
                .as_ref()
                .unwrap()
        };

        let projection = Box::new(MixedProjection::new(
            variables, functions, graph_ref, set_ref,
        ));

        let projection_ref = unsafe {
            (projection.as_ref() as *const MixedProjection)
                .as_ref()
                .unwrap()
        };

        Ok(PySymbolicProjection(
            boxed_graph,
            boxed_set,
            projection,
            projection_ref.iter(),
        ))
    }

    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<(StateData, FunctionData)> {
        slf.3.next().map(|(a, b)| {
            let a: Vec<(PyVariableId, bool)> = a.into_iter().map(|(x, y)| (x.into(), y)).collect();
            let b: Vec<(PyVariableId, PyFnUpdate)> =
                b.into_iter().map(|(x, y)| (x.into(), y.into())).collect();
            (a, b)
        })
    }
}
