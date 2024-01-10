use pyo3::prelude::PyModule;
use pyo3::PyResult;

pub mod fixed_points;
pub mod parameter_id;
pub mod regulatory_graph;
pub mod variable_id;

pub fn register(module: &PyModule) -> PyResult<()> {
    module.add_class::<variable_id::VariableId>()?;
    module.add_class::<parameter_id::ParameterId>()?;
    module.add_class::<regulatory_graph::RegulatoryGraph>()?;
    Ok(())
}
