use crate::internal::classification::classify::classify;
use crate::internal::classification::write_output::build_classification_archive;
use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
use biodivine_lib_param_bn::{BooleanNetwork, ModelAnnotation};
use std::collections::HashMap;

use crate::bindings::lib_param_bn::{
    PyGraphColors, PyGraphVertices, PyModelAnnotation, PySymbolicAsyncGraph,
};

use crate::AsNative;
use crate::{throw_runtime_error, throw_type_error};

use crate::internal::classification::load_inputs::{
    load_classification_archive, read_model_assertions, read_model_properties,
};
use crate::internal::scc::algo_interleaved_transition_guided_reduction::interleaved_transition_guided_reduction;
use crate::internal::scc::algo_xie_beerel::xie_beerel_attractors;
use crate::internal::scc::{Classifier, ClassifierPhenotype};
use pyo3::prelude::*;
use pyo3::PyResult;

pub(crate) fn register(module: &PyModule) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(run_hctl_classification, module)?)?;
    module.add_function(wrap_pyfunction!(run_attractor_classification, module)?)?;
    module.add_function(wrap_pyfunction!(save_class_archive, module)?)?;
    module.add_function(wrap_pyfunction!(load_class_archive, module)?)?;
    module.add_function(wrap_pyfunction!(get_model_assertions, module)?)?;
    module.add_function(wrap_pyfunction!(get_model_properties, module)?)?;
    module.add_function(wrap_pyfunction!(
        run_phenotype_attractor_classification,
        module
    )?)?;
    Ok(())
}

#[pyfunction]
/// Classify the color set of a partially specified Boolean network based on HCTL properties
/// given as model annotations.
///
/// The method takes a path to an annotated AEON model (input) and a path where a classification
/// result archive should be saved (output). The input can be annotated with assertions
/// (see [get_model_assertions]) which restrict the set of admissible colors. The input must
/// be also annotated with properties (see [get_model_properties]) which define the classification
/// classes.
///
/// First, colors satisfying all assertions are computed, and then the set of remaining colors is
/// decomposed into classes based on the satisfied properties. One class maps to a color set
/// where the same HCTL properties are satisfied (universally).
///
/// The output archive is compatible with [load_class_archive] and [save_class_archive].
pub fn run_hctl_classification(model_path: String, output_zip: String) -> PyResult<()> {
    match classify(model_path.as_str(), output_zip.as_str()) {
        Ok(()) => Ok(()),
        Err(error) => throw_runtime_error(error),
    }
}

/// Classify the attractors of the model from the given `model_path` based on their long-term
/// behaviour (i.e. stable, oscillating and disordered). The output is a classification archive
/// written into the `output_zip` path,
#[pyfunction]
pub fn run_attractor_classification(model_path: String, output_zip: String) -> PyResult<()> {
    let model = match BooleanNetwork::try_from_file(model_path) {
        Ok(model) => model,
        Err(error) => return throw_runtime_error(error),
    };
    let graph = match SymbolicAsyncGraph::new(&model) {
        Ok(graph) => graph,
        Err(error) => return throw_runtime_error(error),
    };
    let (states, transitions) =
        interleaved_transition_guided_reduction(&graph, graph.mk_unit_colored_vertices());
    let result = xie_beerel_attractors(&graph, &states, &transitions);
    let scc_classifier = Classifier::new(&graph);
    for attr in result {
        scc_classifier.add_component(attr, &graph);
    }
    let classification = scc_classifier.export_result();
    let classification: HashMap<String, PyGraphColors> = classification
        .into_iter()
        .map(|(a, b)| (a.to_string(), b.into()))
        .collect();

    save_class_archive(output_zip, classification, model.to_string())
}

#[pyfunction]
pub fn run_phenotype_attractor_classification(
    graph: PySymbolicAsyncGraph,
    eligible_phenotypes: Vec<(String, PyGraphVertices)>,
) -> HashMap<String, PyGraphColors> {
    let mut eligible_phenotypes_native = Vec::new();
    for (name, phenotype) in eligible_phenotypes {
        eligible_phenotypes_native.push((name, phenotype.as_native().clone()));
    }

    let stg = graph.as_native();
    let (states, transitions) =
        interleaved_transition_guided_reduction(stg, stg.mk_unit_colored_vertices());
    let result = xie_beerel_attractors(stg, &states, &transitions);
    let classes =
        ClassifierPhenotype::classify_all_components(result, stg, &eligible_phenotypes_native);

    let mut result = HashMap::new();

    for (key, value) in classes {
        result.insert(key, value.into());
    }

    result
}

#[pyfunction]
/// Save an archive with the results of some classification process.
///
/// This archive is compatible with the
/// [BN classifier](https://github.com/sybila/biodivine-bn-classifier) application in
/// which it can be interactively explored. The result should be also compatible with
/// the output of the [run_hctl_classification] function.
///
/// The first argument is the file path where the archive will be saved.
///
/// The second argument is the `class -> colored set` mapping representing the classification
/// data. Finally, the third argument is the string representation of the associated AEON model.
///
/// The process automatically skips any empty class.
pub fn save_class_archive(
    archive_path: String,
    class_mapping: HashMap<String, PyGraphColors>,
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
/// Load an archive containing results produced by a classification process, such as
/// [run_hctl_classification].
///
/// The result is:
///  - A mapping `class name -> color set` for individual "classes" in the archive.
///  - The whole (possibly annotated) model string in the AEON format.
///
/// The names of the classes are simply taken from the BDD-file names in the archive.
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
        Err(error) => throw_runtime_error(error),
    }
}

#[pyfunction]
/// Read the list of assertions from an annotated AEON model.
///
/// If the argument is a single-line string, it is interpreted as a path to an `.aeon` file.
/// If the argument is a multi-line string, it is interpreted as a model file string.
/// Finally, the argument can be a [PyModelAnnotation] which has been already extracted from
/// an annotated AEON model.
///
/// The assertions are expected to appear as `#! dynamic_assertion: FORMULA` in the model string.
/// However, the function only extracts the assertions as strings. They are not "parsed" into any
/// particular format.
pub fn get_model_assertions(annotations: &PyAny) -> PyResult<Vec<String>> {
    let annotations = extract_annotations(annotations)?;
    Ok(read_model_assertions(&annotations))
}

#[pyfunction]
/// Read the list of named properties from an AEON model.
///
/// If the argument is a single-line string, it is interpreted as a path to an `.aeon` file.
/// If the argument is a multi-line string, it is interpreted as a model file string.
/// Finally, the argument can be a [PyModelAnnotation] which has been already extracted from
/// an annotated AEON model.
///
/// The properties are expected to appear as `#! dynamic_property: NAME: FORMULA` in
/// the model string. They are returned in alphabetic order w.r.t. the property name.
/// However, they are returned as raw strings, i.e. not "parsed" into any particular format.
pub fn get_model_properties(annotations: &PyAny) -> PyResult<Vec<(String, String)>> {
    let annotations = extract_annotations(annotations)?;
    match read_model_properties(&annotations) {
        Ok(named_properties) => Ok(named_properties),
        Err(error) => throw_runtime_error(error),
    }
}

/// Extract an annotation object from a Python object.
///
/// An annotation object can be either:
///  - A single line referencing a file path.
///  - Multiple lines representing the contents of an annotated file.
///  - The [PyModelAnnotation] itself.
fn extract_annotations(annotations: &PyAny) -> PyResult<ModelAnnotation> {
    if let Ok(string) = annotations.extract::<String>() {
        if string.contains('\n') {
            // This is a model string.
            Ok(ModelAnnotation::from_model_string(string.as_str()))
        } else {
            // This is a model path.
            match std::fs::read_to_string(string.as_str()) {
                Ok(contents) => Ok(ModelAnnotation::from_model_string(contents.as_str())),
                Err(e) => throw_runtime_error(format!("Cannot read path `{string}`: {e:?}.")),
            }
        }
    } else if let Ok(annotations) = annotations.extract::<PyModelAnnotation>() {
        Ok(annotations.as_native().clone())
    } else {
        throw_type_error("Expected annotation object, model string or path.")
    }
}
