use crate::bindings::lib_param_bn::argument_types::bool_type::BoolType;
use crate::bindings::lib_param_bn::argument_types::variable_id_type::VariableIdType;
use crate::bindings::lib_param_bn::symbolic::model_vertex::VertexModel;
use crate::bindings::lib_param_bn::symbolic::set_vertex::VertexSet;
use crate::bindings::lib_param_bn::variable_id::{VariableIdResolvable, VariableIdResolver};
use crate::{AsNative, throw_runtime_error};
use biodivine_lib_param_bn::biodivine_std::bitvector::BitVector;
use pyo3::{FromPyObject, PyResult};
use std::collections::HashMap;

#[derive(FromPyObject, Clone)]
pub enum SubspaceValuationType {
    Mapping(HashMap<VariableIdType, BoolType>),
    Model(VertexModel),
    Set(VertexSet),
}

impl SubspaceValuationType {
    pub fn resolve<T: VariableIdResolver>(
        &self,
        resolver: &T,
    ) -> PyResult<Vec<(biodivine_lib_param_bn::VariableId, bool)>> {
        match self {
            SubspaceValuationType::Mapping(mapping) => {
                let mut result = Vec::new();
                for (k, v) in mapping {
                    let k = k.resolve(resolver)?;
                    result.push((k, v.bool()));
                }
                Ok(result)
            }
            SubspaceValuationType::Model(model) => Ok(model
                .items()
                .into_iter()
                .map(|(a, b)| (a.into(), b))
                .collect()),
            SubspaceValuationType::Set(set) => {
                let ctx = set.context();
                if !set.is_singleton() {
                    return throw_runtime_error("The state set must be a singleton.");
                }
                let state = set.as_native().iter().next().unwrap();
                Ok(ctx
                    .get()
                    .as_native()
                    .network_variables()
                    .map(|var| (var, state.get(var.to_index())))
                    .collect())
            }
        }
    }
}
