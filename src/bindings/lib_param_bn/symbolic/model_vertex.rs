use std::collections::HashMap;

use pyo3::{pyclass, pymethods, Bound, Py, PyAny, PyResult};

use crate::bindings::lib_bdd::bdd_valuation::BddPartialValuation;
use crate::bindings::lib_param_bn::symbolic::set_vertex::VertexSet;
use crate::bindings::lib_param_bn::symbolic::symbolic_context::SymbolicContext;
use crate::bindings::lib_param_bn::variable_id::VariableId;
use crate::bindings::lib_param_bn::NetworkVariableContext;
use crate::{index_error, AsNative};

/// Represents a single vertex stored in a `VertexSet` (or a `ColoredVertexSet`), or a projection
/// of said vertex to the chosen variables.
///
/// Behaves like an immutable dictionary: Boolean variable values can be queried using
/// a `VariableId`, a string name, or a `BddVariable`.
#[pyclass(module = "biodivine_aeon", frozen)]
#[derive(Clone)]
pub struct VertexModel {
    ctx: Py<SymbolicContext>,
    native: biodivine_lib_bdd::BddPartialValuation,
}

#[pymethods]
impl VertexModel {
    /// Access the underlying `SymbolicContext` connected to this `VertexModel`.
    pub fn __ctx__(&self) -> Py<SymbolicContext> {
        self.ctx.clone()
    }

    pub fn __str__(&self) -> String {
        let items = self
            .to_values()
            .into_iter()
            .map(|(var, value)| {
                let name = self.ctx.get().as_native().bdd_variable_set().name_of(var);
                let value = i32::from(value);
                format!("'{name}': {value}")
            })
            .collect::<Vec<_>>();
        format!("VertexModel({{{}}})", items.join(", "))
    }

    pub fn __repr__(&self) -> String {
        self.__str__()
    }

    /// The number of actual values in this `VertexModel` (i.e. retained network variables).
    pub fn __len__(&self) -> usize {
        self.to_values().len()
    }

    pub fn __getitem__(&self, key: &Bound<'_, PyAny>) -> PyResult<bool> {
        let ctx = self.ctx.get();
        let variable = ctx.resolve_network_variable(key)?;
        let bdd_variable = ctx.as_native().get_state_variable(variable);
        self.native.get_value(bdd_variable).ok_or_else(|| {
            index_error(format!(
                "Variable `{}` not available in this projection.",
                ctx.as_native().get_network_variable_name(variable)
            ))
        })
    }

    pub fn __contains__(&self, key: &Bound<'_, PyAny>) -> PyResult<bool> {
        let ctx = self.ctx.get();
        let variable = ctx.resolve_network_variable(key)?;
        let bdd_variable = ctx.as_native().get_state_variable(variable);
        Ok(self.native.has_value(bdd_variable))
    }

    /// The actual "retained" network variables in this model.
    ///
    /// This is the list of all network variables if no projection was applied.
    pub fn keys(&self) -> Vec<VariableId> {
        let ctx = self.ctx.get();
        let values = self.to_values();
        values
            .into_iter()
            .map(|(it, _)| VariableId::from(ctx.as_native().find_state_variable(it).unwrap()))
            .collect()
    }

    /// The list of values for individual variables from `VertexModel.keys`.
    pub fn values(&self) -> Vec<bool> {
        let values = self.to_values();
        values.into_iter().map(|(_, it)| it).collect()
    }

    /// The list of key-value pairs represented in this model.
    pub fn items(&self) -> Vec<(VariableId, bool)> {
        let ctx = self.ctx.get();
        let values = self.to_values();
        values
            .into_iter()
            .map(|(k, v)| {
                let k = VariableId::from(ctx.as_native().find_state_variable(k).unwrap());
                (k, v)
            })
            .collect()
    }

    /// The same as `VertexModel.items`, but returns a dictionary instead.
    pub fn to_dict(&self) -> HashMap<VariableId, bool> {
        let ctx = self.ctx.get();
        let values = self.to_values();
        values
            .into_iter()
            .map(|(k, v)| {
                let k = VariableId::from(ctx.as_native().find_state_variable(k).unwrap());
                (k, v)
            })
            .collect()
    }

    /// The same as `VertexModel.to_dict`, but the keys in the dictionary are names, not IDs.
    pub fn to_named_dict(&self) -> HashMap<String, bool> {
        let ctx = self.ctx.get().as_native();
        self.to_values()
            .into_iter()
            .map(|(k, v)| {
                let var = ctx.find_state_variable(k).unwrap();
                let name = ctx.get_network_variable_name(var);
                (name, v)
            })
            .collect()
    }

    /// Return the underlying `BddPartialValuation` for this symbolic model.
    pub fn to_valuation(&self) -> BddPartialValuation {
        BddPartialValuation::new_raw(self.ctx.get().bdd_variable_set(), self.native.clone())
    }

    /// Return a `VertexSet` where all the variables are fixed according
    /// to the values in this `VertexModel`. Variables that are not present in the `VertexModel`
    /// are unrestricted.
    pub fn to_symbolic(&self) -> VertexSet {
        let ctx = self.ctx.get();
        let bdd = ctx
            .as_native()
            .bdd_variable_set()
            .mk_conjunctive_clause(&self.native);
        let native =
            biodivine_lib_param_bn::symbolic_async_graph::GraphVertices::new(bdd, ctx.as_native());
        VertexSet::mk_native(self.ctx.clone(), native)
    }
}

impl VertexModel {
    pub fn new_native(
        ctx: Py<SymbolicContext>,
        native: biodivine_lib_bdd::BddPartialValuation,
    ) -> VertexModel {
        VertexModel { ctx, native }
    }

    fn to_values(&self) -> Vec<(biodivine_lib_bdd::BddVariable, bool)> {
        // Only return state variables:
        let mut result = Vec::new();
        for var in self.ctx.get().as_native().state_variables() {
            if let Some(value) = self.native.get_value(*var) {
                result.push((*var, value))
            }
        }
        result
    }
}
