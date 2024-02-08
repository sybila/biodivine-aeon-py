use pyo3::prelude::PyModule;
use pyo3::{PyAny, PyResult};

pub mod algorithms;
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
    module.add_class::<symbolic::symbolic_space_context::SymbolicSpaceContext>()?;
    module.add_class::<symbolic::set_vertex::VertexSet>()?;
    module.add_class::<symbolic::set_vertex::_VertexModelIterator>()?;
    module.add_class::<symbolic::model_vertex::VertexModel>()?;
    module.add_class::<symbolic::set_color::ColorSet>()?;
    module.add_class::<symbolic::set_color::_ColorModelIterator>()?;
    module.add_class::<symbolic::model_color::ColorModel>()?;
    module.add_class::<symbolic::set_spaces::SpaceSet>()?;
    module.add_class::<symbolic::set_spaces::_SpaceModelIterator>()?;
    module.add_class::<symbolic::model_space::SpaceModel>()?;
    module.add_class::<symbolic::set_colored_vertex::ColoredVertexSet>()?;
    module.add_class::<symbolic::set_colored_vertex::_ColorVertexModelIterator>()?;
    module.add_class::<symbolic::set_colored_space::ColoredSpaceSet>()?;
    module.add_class::<symbolic::set_colored_space::_ColorSpaceModelIterator>()?;
    module.add_class::<symbolic::asynchronous_graph::AsynchronousGraph>()?;
    module.add_class::<algorithms::trap_spaces::TrapSpaces>()?;
    module.add_class::<algorithms::fixed_points::FixedPoints>()?;
    module.add_class::<algorithms::attractors::Attractors>()?;
    module.add_class::<algorithms::percolation::Percolation>()?;
    module.add_class::<algorithms::reachability::Reachability>()?;
    module.add_class::<algorithms::regulation_constraint::RegulationConstraint>()?;
    Ok(())
}

/// A trait implemented by types that can resolve a `VariableId` based on its name.
pub trait NetworkVariableContext {
    fn resolve_network_variable(
        &self,
        variable: &PyAny,
    ) -> PyResult<biodivine_lib_param_bn::VariableId>;
    fn get_network_variable_name(&self, variable: biodivine_lib_param_bn::VariableId) -> String;
}
