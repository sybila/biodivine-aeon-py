use pyo3::prelude::*;

pub mod network_regulation;
pub mod network_variable;
pub mod regulatory_graph;

const REFERENCE: &str = "reference";
pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<regulatory_graph::RegulatoryGraphAnnotation>()?;
    module.add_class::<network_variable::NetworkVariableAnnotation>()?;
    module.add_class::<network_variable::VariableIdsAnnotation>()?;
    module.add_class::<network_variable::VariableLayoutAnnotation>()?;
    module.add_class::<network_regulation::NetworkRegulationAnnotation>()?;
    Ok(())
}
