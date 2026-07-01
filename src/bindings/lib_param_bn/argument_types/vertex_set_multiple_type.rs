use crate::bindings::lib_param_bn::argument_types::vertex_set_type::VertexSetType;
use crate::bindings::lib_param_bn::symbolic::asynchronous_graph::AsynchronousGraph;
use crate::bindings::lib_param_bn::symbolic::set_vertex::VertexSet;
use pyo3::{FromPyObject, PyResult, Python};

#[derive(FromPyObject, Clone)]
pub enum VertexSetMultipleType {
    One(VertexSetType),
    Many(Vec<VertexSetType>),
}

impl From<VertexSetMultipleType> for Vec<VertexSetType> {
    fn from(value: VertexSetMultipleType) -> Self {
        match value {
            VertexSetMultipleType::One(one) => vec![one],
            VertexSetMultipleType::Many(many) => many,
        }
    }
}

impl VertexSetMultipleType {
    pub fn resolve_union(self, graph: &AsynchronousGraph, py: Python) -> PyResult<VertexSet> {
        match self {
            VertexSetMultipleType::One(one) => one.resolve(graph, py),
            VertexSetMultipleType::Many(many) => {
                let mut result = graph.mk_empty_vertices();
                for set in many {
                    result = result.union(&set.resolve(graph, py)?);
                }
                Ok(result)
            }
        }
    }
}
