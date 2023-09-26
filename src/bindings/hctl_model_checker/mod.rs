use crate::bindings::lib_param_bn::{
    PyBooleanNetwork, PyGraphColoredVertices, PySymbolicAsyncGraph,
};
use biodivine_hctl_model_checker::analysis::{analyse_formula, analyse_formulae};
use biodivine_hctl_model_checker::mc_utils::get_extended_symbolic_graph;
use biodivine_hctl_model_checker::model_checking::{
    model_check_extended_formula, model_check_multiple_extended_formulae,
    model_check_multiple_trees, model_check_multiple_trees_dirty, model_check_tree,
    model_check_tree_dirty,
};
use biodivine_hctl_model_checker::preprocessing::node::HctlTreeNode;
use biodivine_hctl_model_checker::preprocessing::parser::parse_and_minimize_hctl_formula;
use biodivine_hctl_model_checker::result_print::PrintOptions;
use biodivine_lib_param_bn::BooleanNetwork;
use std::collections::HashMap;

use crate::{throw_runtime_error, throw_type_error, AsNative};

use macros::Wrapper;
use pyo3::prelude::*;
use pyo3::types::PyList;
use pyo3::PyResult;

mod _impl_hctl_tree_node;

pub(crate) fn register(module: &PyModule) -> PyResult<()> {
    module.add_class::<PyHctlTreeNode>()?;

    module.add_function(wrap_pyfunction!(get_extended_stg, module)?)?;
    module.add_function(wrap_pyfunction!(model_check, module)?)?;
    module.add_function(wrap_pyfunction!(model_check_multiple, module)?)?;
    module.add_function(wrap_pyfunction!(model_check_extended, module)?)?;
    module.add_function(wrap_pyfunction!(model_check_multiple_extended, module)?)?;
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
        Err(error) => throw_runtime_error(error),
    }
}

impl PyHctlTreeNode {
    /// Try to read a HCTL tree node from a dynamic Python type. This can be either:
    ///
    ///  - `PyHctlTreeNode` itself;
    ///  - A string that will be parsed as a HCTL formula.
    pub(crate) fn from_python(any: &PyAny, network: &BooleanNetwork) -> PyResult<PyHctlTreeNode> {
        if let Ok(val) = any.extract::<PyHctlTreeNode>() {
            Ok(val)
        } else if let Ok(string) = any.extract::<String>() {
            let parsed = parse_and_minimize_hctl_formula(network, string.as_str());
            match parsed {
                Err(e) => throw_runtime_error(e),
                Ok(tree) => Ok(PyHctlTreeNode::from(tree)),
            }
        } else {
            throw_type_error("Expected a HCTL formula or a HCTL tree node.")
        }
    }
}

#[pyfunction]
#[pyo3(signature = (formula, stg, sanitize=true))]
/// Run the model checking algorithm on a HCTL `formula` ([String] or [PyHctlTreeNode]).
///
/// Argument `sanitize` determines whether the extra symbolic variables required for HCTL model
/// checking should be removed from the result (default: `true`). In general, you should use
/// sanitized results with any other form of post-processing using the standard
/// `SymbolicAsyncGraph`, while non-sanitized results can be used for further model checking.
///
/// Returns a satisfying color-state relation.
pub fn model_check(
    formula: &PyAny,
    stg: &PySymbolicAsyncGraph,
    sanitize: bool,
) -> PyResult<PyGraphColoredVertices> {
    let stg = stg.as_native();
    let formula = PyHctlTreeNode::from_python(formula, stg.as_network())?;

    let result = if sanitize {
        model_check_tree(formula.into(), stg)
    } else {
        model_check_tree_dirty(formula.into(), stg)
    };

    match result {
        Ok(colored_states) => Ok(colored_states.into()),
        Err(error) => throw_runtime_error(error),
    }
}

#[pyfunction]
#[pyo3(signature = (formulae, stg, sanitize=true))]
/// Run the model checking algorithm on a list of HCTL formulae (each formula can be a [String]
/// or a [HctlTreeNode]).
///
/// Argument `sanitize` determines whether the results are sanitized (i.e., the underlying BDDs
/// are `cleaned` of redundant symbolic variables).
///
/// Returns a list of satisfying color-state relations, one for each formula.
pub fn model_check_multiple(
    formulae: &PyList,
    stg: &PySymbolicAsyncGraph,
    sanitize: bool,
) -> PyResult<Vec<PyGraphColoredVertices>> {
    let stg = stg.as_native();
    let mut list: Vec<HctlTreeNode> = Vec::new();
    for formula in formulae {
        list.push(PyHctlTreeNode::from_python(formula, stg.as_network())?.into());
    }

    let result = if sanitize {
        model_check_multiple_trees(list, stg)
    } else {
        model_check_multiple_trees_dirty(list, stg)
    };

    match result {
        Ok(list_colored_states) => Ok(list_colored_states.into_iter().map(|r| r.into()).collect()),
        Err(error) => throw_runtime_error(error),
    }
}

#[pyfunction]
/// Run the model checking algorithm on an `extended HCTL formula` ([String]).
/// The `substitution context` is a mapping determining how `wild-card propositions` are evaluated.
///
/// Returns a satisfying color-state relation.
pub fn model_check_extended(
    formula: String,
    stg: &PySymbolicAsyncGraph,
    substitution_context: HashMap<String, PyGraphColoredVertices>,
) -> PyResult<PyGraphColoredVertices> {
    let stg_native = stg.as_native();
    let context_native = substitution_context
        .into_iter()
        .map(|(s, c)| (s, c.into()))
        .collect();
    match model_check_extended_formula(formula, stg_native, context_native) {
        Ok(result) => Ok(result.into()),
        Err(error) => throw_runtime_error(error),
    }
}

#[pyfunction]
/// Run the model checking algorithm on a list of HCTL formulae (each being a [String]).
/// The `substitution context` is a mapping determining how `wild-card propositions` are evaluated.
///
/// Returns a list of satisfying color-state relations, one for each formula.
pub fn model_check_multiple_extended(
    formulae: Vec<String>,
    stg: &PySymbolicAsyncGraph,
    substitution_context: HashMap<String, PyGraphColoredVertices>,
) -> PyResult<Vec<PyGraphColoredVertices>> {
    let stg_native = stg.as_native();
    let context_native = substitution_context
        .into_iter()
        .map(|(s, c)| (s, c.into()))
        .collect();

    match model_check_multiple_extended_formulae(formulae, stg_native, context_native) {
        Ok(results) => Ok(results.iter().map(|r| r.clone().into()).collect()),
        Err(error) => throw_runtime_error(error),
    }
}

#[pyfunction]
#[pyo3(signature = (bn, formula, print_progress=true))]
/// Run the whole model checking analysis pipeline on a single formula.
/// Argument `print_progress` determines the amount of progress printed. If false (default), only
/// the results summary is printed. Otherwise also some progress details are given.
pub fn mc_analysis(bn: PyBooleanNetwork, formula: String, print_progress: bool) -> PyResult<()> {
    let result = if print_progress {
        analyse_formula(bn.as_native(), formula, PrintOptions::WithProgress)
    } else {
        analyse_formula(bn.as_native(), formula, PrintOptions::JustSummary)
    };

    match result {
        Ok(()) => Ok(()),
        Err(e) => throw_runtime_error(e),
    }
}

#[pyfunction]
#[pyo3(signature = (bn, formulae, print_progress=true))]
/// Run the whole model checking analysis pipeline on a list of several (individual) formulae.
/// Argument `print_progress` determines the amount of progress printed. If false (default), only
/// the results summary is printed. Otherwise also some progress details are given.
pub fn mc_analysis_multiple(
    bn: PyBooleanNetwork,
    formulae: Vec<String>,
    print_progress: bool,
) -> PyResult<()> {
    let result = if print_progress {
        analyse_formulae(bn.as_native(), formulae, PrintOptions::WithProgress)
    } else {
        analyse_formulae(bn.as_native(), formulae, PrintOptions::JustSummary)
    };

    match result {
        Ok(()) => Ok(()),
        Err(e) => throw_runtime_error(e),
    }
}
