use crate::bindings::lib_param_bn::argument_types::variable_id_type::VariableIdType;
use crate::bindings::lib_param_bn::variable_id::VariableIdResolvable;
use crate::bindings::pbn_control::PerturbationModel;
use crate::throw_runtime_error;
use biodivine_lib_param_bn::VariableId as VariableIdNative;
use biodivine_pbn_control::perturbation::PerturbationGraph;
use pyo3::{FromPyObject, PyResult};
use std::collections::{HashMap, HashSet};

#[derive(FromPyObject, Clone)]
pub enum PerturbationType {
    Mapping(HashMap<VariableIdType, Option<bool>>),
    Model(PerturbationModel),
}

impl PerturbationType {
    pub fn resolve(
        &self,
        graph: &PerturbationGraph,
    ) -> PyResult<HashMap<VariableIdNative, Option<bool>>> {
        let mut result = HashMap::new();
        match self {
            PerturbationType::Mapping(mapping) => {
                for (k, v) in mapping {
                    let k = k.resolve(graph.as_symbolic_context())?;
                    result.insert(k, *v);
                }
            }
            PerturbationType::Model(model) => {
                for (k, v) in model.items() {
                    result.insert(k.into(), v);
                }
            }
        }
        let perturbable = graph
            .perturbable_variables()
            .iter()
            .copied()
            .collect::<HashSet<_>>();
        for var in result.keys() {
            if !perturbable.contains(var) {
                return throw_runtime_error(format!("Variable {var} cannot be perturbed."));
            }
        }
        Ok(result)
    }
}
