use biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices;
use pyo3::{Bound, Py, PyAny, PyResult, Python, pyclass, pymethods};
use std::collections::HashMap;

use crate::bindings::lib_bdd::bdd_valuation::BddPartialValuation;
use crate::bindings::lib_param_bn::NetworkVariableContext;
use crate::bindings::lib_param_bn::variable_id::VariableId;
use crate::bindings::pbn_control::asynchronous_perturbation_graph::AsynchronousPerturbationGraph;
use crate::bindings::pbn_control::set_perturbation::PerturbationSet;
use crate::{AsNative, throw_index_error, throw_runtime_error};

/// Represents a single perturbation stored in a `PerturbationSet` (or a `ColoredPerturbationSet`),
/// or a projection of said perturbation to the chosen variables.
///
/// Behaves like an immutable dictionary: Boolean variable values can be queried using
/// a `VariableId`, a string name, or a `BddVariable`. Values that are perturbable but not
/// perturbed return `None`, while values that are not perturbable raise a `KeyError` exception.
#[pyclass(module = "biodivine_aeon", frozen)]
#[derive(Clone)]
pub struct PerturbationModel {
    ctx: Py<AsynchronousPerturbationGraph>,
    native: biodivine_lib_bdd::BddPartialValuation,
    parameter_mapping: HashMap<biodivine_lib_param_bn::VariableId, biodivine_lib_bdd::BddVariable>,
}

#[pymethods]
impl PerturbationModel {
    /// Access the underlying `AsynchronousPerturbationGraph`
    /// connected to this `PerturbationModel`.
    pub fn __ctx__(&self) -> Py<AsynchronousPerturbationGraph> {
        self.ctx.clone()
    }

    pub fn __str__(&self) -> String {
        let items = self
            .items()
            .into_iter()
            .map(|(var, value)| {
                let name = self
                    .ctx
                    .get()
                    .as_native()
                    .as_original()
                    .get_variable_name(var.into());
                if let Some(value) = value {
                    let value = i32::from(value);
                    format!("'{name}': {value}")
                } else {
                    format!("'{name}': None")
                }
            })
            .collect::<Vec<_>>();
        format!("PerturbationModel({{{}}})", items.join(", "))
    }

    pub fn __repr__(&self) -> String {
        self.__str__()
    }

    /// The number of actual values in this `PerturbationModel` (i.e., retained network variables).
    pub fn __len__(&self) -> usize {
        self.items().len()
    }

    pub fn __getitem__(&self, py: Python, key: &Bound<'_, PyAny>) -> PyResult<Option<bool>> {
        let ctx_ref = self.ctx.borrow(py);
        let var = ctx_ref.as_ref().resolve_network_variable(key)?;
        let ctx = ctx_ref.as_native();
        let s_var = ctx.as_symbolic_context().get_state_variable(var);
        let Some(par) = ctx.get_perturbation_parameter(var) else {
            let var_name = ctx.as_original().get_variable_name(var);
            return throw_runtime_error(format!("Variable {var_name} cannot be perturbed."));
        };
        let p_var = ctx
            .as_symbolic_context()
            .get_explicit_function_table(par)
            .symbolic_variables()[0];
        let s_value = self.native.get_value(s_var);
        let p_value = self.native.get_value(p_var);
        match (s_value, p_value) {
            (None, None) => {
                let var_name = ctx.as_original().get_variable_name(var);
                throw_index_error(format!(
                    "Variable {var_name} not available in this projection."
                ))
            }
            (Some(true), Some(true)) => Ok(Some(true)),
            (Some(false), Some(true)) => Ok(Some(false)),
            (_, Some(false)) => Ok(None),
            (None, Some(true)) | (Some(_), None) => {
                panic!("This valuation does not represent a valid perturbation.")
            }
        }
    }

    pub fn __contains__(&self, py: Python, key: &Bound<'_, PyAny>) -> PyResult<bool> {
        let ctx = self.ctx.borrow(py);
        let variable = ctx.as_ref().resolve_network_variable(key)?;
        if let Some(p_var) = self.parameter_mapping.get(&variable) {
            Ok(self.native.has_value(*p_var))
        } else {
            Ok(false)
        }
    }

    /// The actual "retained" network variables in this model.
    ///
    /// This is the list of all network variables if no projection was applied.
    pub fn keys(&self) -> Vec<VariableId> {
        self.items().into_iter().map(|(k, _)| k).collect()
    }

    /// The list of variables that are perturbed in this model (either `true` or `false`).
    pub fn perturbed(&self) -> Vec<VariableId> {
        self.items()
            .into_iter()
            .filter(|(_, v)| v.is_some())
            .map(|(k, _)| k)
            .collect()
    }

    /// Returns a dictionary of only the perturbed variables and their values.
    pub fn perturbed_dict(&self) -> HashMap<VariableId, bool> {
        self.items()
            .into_iter()
            .filter_map(|(k, v)| v.map(|v| (k, v)))
            .collect()
    }

    /// Returns a dictionary of only the perturbed variables (identified by names)
    /// and their values.
    pub fn perturbed_named_dict(&self) -> HashMap<String, bool> {
        let ctx = self.ctx.get().as_native().as_symbolic_context();
        self.items()
            .into_iter()
            .filter_map(|(k, v)| {
                if let Some(v) = v {
                    let name = ctx.get_network_variable_name(k.into());
                    Some((name, v))
                } else {
                    None
                }
            })
            .collect()
    }

    /// The list of variables that are unperturbed in this model (i.e. `None`).
    pub fn unperturbed(&self) -> Vec<VariableId> {
        self.items()
            .into_iter()
            .filter(|(_, v)| v.is_none())
            .map(|(k, _)| k)
            .collect()
    }

    /// The size of this perturbation. That is, the number of perturbed variables.
    pub fn perturbation_size(&self) -> usize {
        self.items()
            .into_iter()
            .filter(|(_, v)| v.is_some())
            .count()
    }

    /// The list of values for individual variables from `PerturbationModel.keys`.
    pub fn values(&self) -> Vec<Option<bool>> {
        self.items().into_iter().map(|(_, v)| v).collect()
    }

    /// The list of key-value pairs represented in this model.
    pub fn items(&self) -> Vec<(VariableId, Option<bool>)> {
        let ctx = self.ctx.get().as_native();
        let mut result = Vec::new();
        for var in ctx.perturbable_variables() {
            let s_var = ctx.as_symbolic_context().get_state_variable(*var);
            let p_var = *self.parameter_mapping.get(var).unwrap();
            let s_value = self.native.get_value(s_var);
            let p_value = self.native.get_value(p_var);
            let value = match (s_value, p_value) {
                (None, None) => continue,
                (Some(true), Some(true)) => Some(true),
                (Some(false), Some(true)) => Some(false),
                (_, Some(false)) => None,
                (None, Some(true)) | (Some(_), None) => {
                    panic!("This valuation does not represent a valid perturbation.")
                }
            };
            result.push((VariableId::from(*var), value));
        }
        result
    }

    /// The same as `PerturbationModel.items`, but returns a dictionary instead.
    pub fn to_dict(&self) -> HashMap<VariableId, Option<bool>> {
        self.items().into_iter().collect()
    }

    /// The same as `PerturbationModel.to_dict`, but the keys in the dictionary are names, not IDs.
    pub fn to_named_dict(&self) -> HashMap<String, Option<bool>> {
        let ctx = self.ctx.get().as_native().as_original().symbolic_context();
        self.items()
            .into_iter()
            .map(|(k, v)| {
                let name = ctx.get_network_variable_name(k.into());
                (name, v)
            })
            .collect()
    }

    /// Return the underlying `BddPartialValuation` for this symbolic model.
    pub fn to_valuation(&self, py: Python) -> BddPartialValuation {
        BddPartialValuation::new_raw(
            self.ctx
                .borrow(py)
                .as_ref()
                .symbolic_context()
                .borrow(py)
                .bdd_variable_set(),
            self.native.clone(),
        )
    }

    /// Return a `PerturbationSet` where all the variables are fixed according
    /// to the values in this `PerturbationModel`. Variables that are not present in the
    /// `PerturbationModel` are unrestricted.
    pub fn to_symbolic(&self) -> PerturbationSet {
        let ctx = self.ctx.get();
        let bdd = ctx
            .as_native()
            .as_symbolic_context()
            .bdd_variable_set()
            .mk_conjunctive_clause(&self.native);
        let native = GraphColoredVertices::new(bdd, ctx.as_native().as_symbolic_context());
        PerturbationSet::mk_native(self.ctx.clone(), native)
    }
}

impl PerturbationModel {
    pub fn new_native(
        ctx: Py<AsynchronousPerturbationGraph>,
        native: biodivine_lib_bdd::BddPartialValuation,
        parameter_mapping: HashMap<
            biodivine_lib_param_bn::VariableId,
            biodivine_lib_bdd::BddVariable,
        >,
    ) -> PerturbationModel {
        PerturbationModel {
            ctx,
            native,
            parameter_mapping,
        }
    }

    pub fn native_valuation(&self) -> &biodivine_lib_bdd::BddPartialValuation {
        &self.native
    }
}
