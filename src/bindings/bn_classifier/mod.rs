use crate::internal::classification::classify::classify;
use crate::internal::classification::write_output::build_classification_archive;
use std::collections::HashMap;

use crate::bindings::lib_param_bn::{PyGraphColors, PyModelAnnotation};

use crate::throw_runtime_error;
use crate::AsNative;

use crate::internal::classification::load_inputs::{
    load_classification_archive, read_model_assertions, read_model_properties,
};
use pyo3::prelude::*;
use pyo3::PyResult;

pub(crate) fn register(module: &PyModule) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(run_classification, module)?)?;
    module.add_function(wrap_pyfunction!(build_class_archive, module)?)?;
    module.add_function(wrap_pyfunction!(load_class_archive, module)?)?;
    module.add_function(wrap_pyfunction!(get_model_assertions, module)?)?;
    module.add_function(wrap_pyfunction!(get_model_properties, module)?)?;
    Ok(())
}

#[pyfunction]
/// Perform the classification of Boolean networks based on given properties.
/// Takes a path to a file in annotated `AEON` format containing a partially defined BN model
/// and 2 sets of HCTL formulae. `Assertions` are formulae that must be satisfied by all models,
/// and `properties` are formulae used for classification.
///
/// First, colors satisfying all assertions are computed, and then the set of remaining colors is
/// decomposed into categories based on satisfied properties. One class = colors where the same set
/// of properties is satisfied (universally).
///
/// Report and BDDs representing resulting classes are generated into `output_zip` archive.
pub fn run_classification(model_path: String, output_zip: String) -> PyResult<()> {
    match classify(model_path.as_str(), output_zip.as_str()) {
        Ok(()) => Ok(()),
        Err(error) => throw_runtime_error(error),
    }
}

#[pyfunction]
/// Generate `classification` archive for an arbitrary "map" of `string -> color set` (one item
/// represents one class).
/// Argument `archive_path` sets the path for results bundle, `model_aeon_str` is the original
/// model in aeon format (which needs to be included in results to later load the classes).
///
/// Note that empty classes are ignored.
pub fn build_class_archive(
    class_mapping: HashMap<String, PyGraphColors>,
    archive_path: String,
    model_aeon_str: String,
) -> PyResult<()> {
    let mapping_native = class_mapping
        .into_iter()
        .map(|(s, c)| (s, c.into()))
        .collect();
    match build_classification_archive(
        mapping_native,
        archive_path.as_str(),
        model_aeon_str.as_str(),
    ) {
        Ok(()) => Ok(()),
        Err(error) => throw_runtime_error(format!("{error:?}")),
    }
}

#[pyfunction]
/// Load the archive containing results produced by the classifier.
/// This function can also be used to load any `classification archives` of the same format (e.g.,
/// those produced by the `build_class_archive` function).
///
/// Return mapping `category name -> color set` and whole model string in aeon format.
/// Category names are simply taken from BDD-file names in the archive.
pub fn load_class_archive(
    archive_path: String,
) -> PyResult<(HashMap<String, PyGraphColors>, String)> {
    match load_classification_archive(archive_path) {
        Ok((native_mapping, model_str)) => {
            let mapping = native_mapping
                .into_iter()
                .map(|(s, c)| (s, c.into()))
                .collect();
            Ok((mapping, model_str))
        }
        Err(error) => throw_runtime_error(format!("{error:?}")),
    }
}

#[pyfunction]
/// Read the list of assertions from an `aeon model annotation` object.
/// The assertions are expected to appear as `#! dynamic_assertion: FORMULA` in model string.
pub fn get_model_assertions(annotations: &PyModelAnnotation) -> Vec<String> {
    read_model_assertions(annotations.as_native())
}

#[pyfunction]
/// Read the list of named properties from an `aeon model annotation` object.
///
/// The properties are expected to appear as `#! dynamic_property: NAME: FORMULA` in model string.
/// They are returned in alphabetic order w.r.t. the property name.
pub fn get_model_properties(annotations: &PyModelAnnotation) -> PyResult<Vec<(String, String)>> {
    match read_model_properties(annotations.as_native()) {
        Ok(named_properties) => Ok(named_properties),
        Err(error) => throw_runtime_error(format!("{error:?}")),
    }
}
