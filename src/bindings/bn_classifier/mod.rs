use pyo3::prelude::*;
use pyo3::PyResult;

mod class;
mod classification;

pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<class::Class>()?;
    module.add_class::<classification::Classification>()?;
    Ok(())
}

/*
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

*/
