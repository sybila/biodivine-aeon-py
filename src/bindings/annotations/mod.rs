use pyo3::prelude::*;

pub mod network_regulation;
pub mod network_variable;
pub mod regulatory_graph;

const REFERENCE: &str = "reference";
/// Registers Rust annotation classes as Python classes within the given Python module.
///
/// This function exposes several Rust annotation types to Python by adding them as classes to the provided module. It is intended for use with PyO3-based Python bindings.
///
/// # Examples
///
/// ```
/// use pyo3::prelude::*;
/// use my_crate::bindings::annotations::register;
///
/// Python::with_gil(|py| {
///     let module = PyModule::new(py, "my_module").unwrap();
///     register(&module).unwrap();
/// });
/// ```
pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<regulatory_graph::RegulatoryGraphAnnotation>()?;
    module.add_class::<network_variable::NetworkVariableAnnotation>()?;
    module.add_class::<network_variable::VariableIdsAnnotation>()?;
    module.add_class::<network_variable::VariableLayoutAnnotation>()?;
    module.add_class::<network_regulation::NetworkRegulationAnnotation>()?;
    Ok(())
}
