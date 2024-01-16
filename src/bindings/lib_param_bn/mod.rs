use pyo3::prelude::PyModule;
use pyo3::PyResult;

pub mod boolean_network;
pub mod fixed_points;
pub mod model_annotation;
pub mod parameter_id;
pub mod regulatory_graph;
pub mod update_function;
pub mod variable_id;

pub fn register(module: &PyModule) -> PyResult<()> {
    module.add_class::<variable_id::VariableId>()?;
    module.add_class::<parameter_id::ParameterId>()?;
    module.add_class::<regulatory_graph::RegulatoryGraph>()?;
    module.add_class::<boolean_network::BooleanNetwork>()?;
    module.add_class::<update_function::UpdateFunction>()?;
    module.add_class::<model_annotation::ModelAnnotationRoot>()?;
    module.add_class::<model_annotation::ModelAnnotation>()?;
    Ok(())
}
