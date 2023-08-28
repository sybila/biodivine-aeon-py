use crate::bindings::lib_param_bn::{PyBooleanNetwork, PyGraphColoredVertices, PySymbolicAsyncGraph};
use biodivine_hctl_model_checker::analysis::{analyse_formula, analyse_formulae};
use biodivine_hctl_model_checker::mc_utils::get_extended_symbolic_graph;
use biodivine_hctl_model_checker::model_checking::{
    model_check_formula, model_check_multiple_formulae, model_check_tree, model_check_trees
};
use biodivine_hctl_model_checker::preprocessing::node::HctlTreeNode;
use biodivine_hctl_model_checker::result_print::PrintOptions;

use crate::{AsNative, throw_runtime_error};

use macros::Wrapper;
use pyo3::PyResult;
use pyo3::prelude::*;

mod _impl_hctl_tree_node;

pub(crate) fn register(module: &PyModule) -> PyResult<()> {
    module.add_class::<PyHctlTreeNode>()?;

    module.add_function(wrap_pyfunction!(get_extended_stg, module)?)?;
    module.add_function(wrap_pyfunction!(model_check, module)?)?;
    module.add_function(wrap_pyfunction!(model_check_multiple, module)?)?;
    module.add_function(wrap_pyfunction!(model_check_hctl_tree, module)?)?;
    module.add_function(wrap_pyfunction!(model_check_multiple_hctl_trees, module)?)?;
    module.add_function(wrap_pyfunction!(mc_analysis, module)?)?;
    module.add_function(wrap_pyfunction!(mc_analysis_multiple, module)?)?;
    Ok(())
}

/// Structure for a HCTL formula syntax tree.
#[pyclass(name = "HctlTreeNode")]
#[derive(Clone, Debug, Eq, Hash, PartialEq, Wrapper)]
pub struct PyHctlTreeNode(HctlTreeNode);

#[pyfunction]
/// Create an extended symbolic transition graph that supports the number of needed HCTL variables.
pub fn get_extended_stg(
    bn: PyBooleanNetwork,
    num_hctl_vars: u16,
) -> PyResult<PySymbolicAsyncGraph> {
    match get_extended_symbolic_graph(&bn.as_native().clone(), num_hctl_vars) {
        Ok(result) => Ok(result.into()),
        Err(error) => throw_runtime_error(error)
    }
}

#[pyfunction]
/// Run the model checking algorithm on a HCTL formula.
/// Returns a satisfying color-state relation.
pub fn model_check(
    formula: String,
    stg: &PySymbolicAsyncGraph,
) -> PyResult<PyGraphColoredVertices> {
    match model_check_formula(formula, stg.as_native()) {
        Ok(result) => Ok(result.into()),
        Err(error) => throw_runtime_error(error)
    }
}

#[pyfunction]
/// Run the model checking algorithm on a list of HCTL formulae.
/// Returns a list of satisfying color-state relations, one for each formula.
pub fn model_check_multiple(
    formulae: Vec<String>,
    stg: &PySymbolicAsyncGraph,
) -> PyResult<Vec<PyGraphColoredVertices>> {
    match model_check_multiple_formulae(formulae, stg.as_native()) {
        Ok(results) => {
            Ok(results.iter().map(|r| r.clone().into()).collect())
        },
        Err(error) => throw_runtime_error(error)
    }
}

#[pyfunction]
/// Run the model checking algorithm on a HCTL formula given by its syntactic tree.
/// Returns a satisfying color-state relation.
pub fn model_check_hctl_tree(
    tree: PyHctlTreeNode,
    stg: &PySymbolicAsyncGraph,
) -> PyResult<PyGraphColoredVertices> {
    match model_check_tree(tree.as_native().clone(), stg.as_native()) {
        Ok(result) => Ok(result.into()),
        Err(error) => throw_runtime_error(error)
    }
}

#[pyfunction]
/// Run the model checking algorithm on a list of HCTL formulae given by their syntactic trees.
/// Returns a list of satisfying color-state relations, one for each formula.
pub fn model_check_multiple_hctl_trees(
    trees: Vec<PyHctlTreeNode>,
    stg: &PySymbolicAsyncGraph,
) -> PyResult<Vec<PyGraphColoredVertices>> {
    let native_trees = trees.iter().map(|r| r.as_native().clone()).collect();
    match model_check_trees(native_trees, stg.as_native()) {
        Ok(results) => {
            Ok(results.iter().map(|r| r.clone().into()).collect())
        },
        Err(error) => throw_runtime_error(error)
    }
}


#[pyfunction]
/// Run the whole model checking analysis pipeline on a single formula.
pub fn mc_analysis(
    bn: PyBooleanNetwork,
    formula: String,
) -> PyResult<()> {
    let result = analyse_formula(&bn.as_native().clone(), formula, PrintOptions::MediumPrint);
    match result {
        Ok(()) => Ok(()),
        Err(e) => throw_runtime_error(e),
    }
}

#[pyfunction]
/// Run the whole model checking analysis pipeline on a list of several (individual) formulae.
pub fn mc_analysis_multiple(
    bn: PyBooleanNetwork,
    formulae: Vec<String>,
) -> PyResult<()> {
    let result = analyse_formulae(&bn.as_native().clone(), formulae, PrintOptions::MediumPrint);
    match result {
        Ok(()) => Ok(()),
        Err(e) => throw_runtime_error(e),
    }
}
