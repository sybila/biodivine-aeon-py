use std::collections::HashMap;

use biodivine_lib_param_bn::VariableId;
use pyo3::{pyclass, FromPyObject};

use crate::{bindings::lib_param_bn::variable_id::VariableId as VariableIdBinding, AsNative};

#[pyclass(module = "biodivine_aeon", frozen)]
#[derive(FromPyObject)]
pub enum SubspaceRepresentation {
    List(Vec<(VariableIdBinding, bool)>),
    Dict(HashMap<VariableIdBinding, bool>),
}

impl From<SubspaceRepresentation> for Vec<(VariableId, bool)> {
    fn from(subspace: SubspaceRepresentation) -> Self {
        match subspace {
            SubspaceRepresentation::List(vec) => vec
                .into_iter()
                .map(|(var, value)| (*var.as_native(), value))
                .collect(),

            SubspaceRepresentation::Dict(map) => map
                .into_iter()
                .map(|(var, value)| (*var.as_native(), value))
                .collect(),
        }
    }
}

impl From<Vec<(VariableId, bool)>> for SubspaceRepresentation {
    /// Create a new [SubspaceRepresentation] from the given list of variable-value pairs.
    /// Dict is chosen as default for backwards compatibility reasons.
    fn from(subspace: Vec<(VariableId, bool)>) -> Self {
        SubspaceRepresentation::Dict(
            subspace
                .into_iter()
                .map(|(var, value)| (VariableIdBinding::from(var), value))
                .collect(),
        )
    }
}
