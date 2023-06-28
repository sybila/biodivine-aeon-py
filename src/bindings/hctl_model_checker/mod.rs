use crate::bindings::lib_param_bn::{PyBooleanNetwork, PyGraphColoredVertices, PySymbolicAsyncGraph};
use biodivine_hctl_model_checker::analysis::{analyse_formula, analyse_formulae};
use biodivine_hctl_model_checker::model_checking::{
    get_extended_symbolic_graph, model_check_multiple_formulae, model_check_formula
};
use biodivine_hctl_model_checker::result_print::PrintOptions;

use crate::{AsNative, throw_runtime_error};

use pyo3::prelude::*;
use pyo3::PyResult;

pub(crate) fn register(module: &PyModule) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(model_check, module)?)?;
    module.add_function(wrap_pyfunction!(model_check_multiple, module)?)?;
    module.add_function(wrap_pyfunction!(mc_analysis, module)?)?;
    module.add_function(wrap_pyfunction!(mc_analysis_multiple, module)?)?;
    module.add_function(wrap_pyfunction!(get_extended_stg, module)?)?;
    Ok(())
}

#[pyfunction]
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
pub fn mc_analysis(
    bn: PyBooleanNetwork,
    formula: String,
) -> PyResult<()> {
    let result = analyse_formula(&bn.as_native().clone(), formula, PrintOptions::ShortPrint);
    match result {
        Ok(()) => Ok(()),
        Err(e) => throw_runtime_error(e),
    }
}

#[pyfunction]
pub fn mc_analysis_multiple(
    bn: PyBooleanNetwork,
    formulae: Vec<String>,
) -> PyResult<()> {
    let result = analyse_formulae(&bn.as_native().clone(), formulae, PrintOptions::ShortPrint);
    match result {
        Ok(()) => Ok(()),
        Err(e) => throw_runtime_error(e),
    }
}

#[pyfunction]
pub fn get_extended_stg(
    bn: PyBooleanNetwork,
    num_hctl_vars: u16,
) -> PySymbolicAsyncGraph {
    get_extended_symbolic_graph(&bn.as_native().clone(), num_hctl_vars).into()
}

