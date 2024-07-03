use pyo3::prelude::{PyModule, PyModuleMethods};
use pyo3::{Bound, PyResult};

pub mod hctl_formula;

pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<hctl_formula::HctlFormula>()?;
    Ok(())
}

/*
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
    pub(crate) fn from_python(any: &PyAny, ctx: &SymbolicContext) -> PyResult<PyHctlTreeNode> {
        if let Ok(val) = any.extract::<PyHctlTreeNode>() {
            Ok(val)
        } else if let Ok(string) = any.extract::<String>() {
            let parsed = parse_and_minimize_hctl_formula(ctx, string.as_str());
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
    let formula = PyHctlTreeNode::from_python(formula, stg.symbolic_context())?;

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
        list.push(PyHctlTreeNode::from_python(formula, stg.symbolic_context())?.into());
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


 */
