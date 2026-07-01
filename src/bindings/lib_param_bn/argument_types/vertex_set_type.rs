use crate::bindings::lib_param_bn::argument_types::bool_type::BoolType;
use crate::bindings::lib_param_bn::argument_types::subspace_valuation_type::SubspaceValuationType;
use crate::bindings::lib_param_bn::argument_types::variable_id_type::VariableIdType;
use crate::bindings::lib_param_bn::symbolic::asynchronous_graph::AsynchronousGraph;
use crate::bindings::lib_param_bn::symbolic::model_space::SpaceModel;
use crate::bindings::lib_param_bn::symbolic::model_vertex::VertexModel;
use crate::bindings::lib_param_bn::symbolic::set_colored_vertex::ColoredVertexSet;
use crate::bindings::lib_param_bn::symbolic::set_vertex::VertexSet;
use pyo3::{FromPyObject, PyResult, Python};
use std::collections::HashMap;

#[derive(FromPyObject, Clone)]
pub enum VertexSetType {
    Set(VertexSet),
    ColoredSet(ColoredVertexSet),
    Model(VertexModel),
    SpaceModel(SpaceModel),
    Mapping(HashMap<VariableIdType, BoolType>),
}

impl VertexSetType {
    pub fn resolve(self, graph: &AsynchronousGraph, py: Python) -> PyResult<VertexSet> {
        match self {
            VertexSetType::Set(set) => Ok(set),
            VertexSetType::ColoredSet(set) => Ok(set.vertices()),
            VertexSetType::Model(model) => Ok(model.to_symbolic()),
            VertexSetType::SpaceModel(model) => {
                model.to_symbolic().to_vertices(model.__ctx__(), py)
            }
            VertexSetType::Mapping(mapping) => {
                graph.mk_subspace_vertices(SubspaceValuationType::Mapping(mapping))
            }
        }
    }
}
