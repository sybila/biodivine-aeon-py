use pyo3::prelude::PyModule;
use pyo3::PyResult;

pub mod boolean_network;
pub mod fixed_points;
pub mod model_annotation;
pub mod parameter_id;
pub mod regulatory_graph;
pub mod symbolic;
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
    module.add_class::<symbolic::symbolic_context::SymbolicContext>()?;
    module.add_class::<symbolic::set_vertex::VertexSet>()?;
    module.add_class::<symbolic::set_vertex::_VertexModelIterator>()?;
    module.add_class::<symbolic::model_vertex::VertexModel>()?;
    module.add_class::<symbolic::set_color::ColorSet>()?;
    module.add_class::<symbolic::set_color::_ColorModelIterator>()?;
    module.add_class::<symbolic::model_color::ColorModel>()?;
    module.add_class::<symbolic::set_colored_vertex::ColoredVertexSet>()?;
    module.add_class::<symbolic::set_colored_vertex::_ColorVertexModelIterator>()?;
    module.add_class::<symbolic::asynchronous_graph::AsynchronousGraph>()?;
    Ok(())
}
