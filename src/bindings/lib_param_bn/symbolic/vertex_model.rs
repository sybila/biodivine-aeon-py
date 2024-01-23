use crate::bindings::lib_param_bn::symbolic::symbolic_context::SymbolicContext;
use crate::bindings::lib_param_bn::variable_id::VariableId;
use crate::{index_error, AsNative};
use biodivine_lib_bdd::BddPartialValuation;
use pyo3::{pyclass, pymethods, Py, PyAny, PyResult};
use std::collections::HashMap;

/// Represents a single vertex stored in a `VertexSet` (or a `ColoredVertexSet`), or a projection
/// of said vertex to the chosen variables.
///
/// Behaves like an immutable dictionary: Boolean variable values can be queried using
/// a `VariableId`, a string name, or a `BddVariable`.
#[pyclass(module = "biodivine_aeon", frozen)]
#[derive(Clone)]
pub struct VertexModel {
    ctx: Py<SymbolicContext>,
    native: BddPartialValuation,
}

#[pymethods]
impl VertexModel {
    /// Access the underlying `SymbolicContext` connected to this `VertexModel`.
    pub fn __ctx__(&self) -> Py<SymbolicContext> {
        self.ctx.clone()
    }

    pub fn __str__(&self) -> String {
        let items = self
            .native
            .to_values()
            .into_iter()
            .map(|(var, value)| {
                let name = self.ctx.get().as_native().bdd_variable_set().name_of(var);
                let value = i32::from(value);
                format!("'{}': {}", name, value)
            })
            .collect::<Vec<_>>();
        format!("VertexModel({{{}}})", items.join(", "))
    }

    /// The number of actual values in this `VertexModel` (i.e. retained network variables).
    pub fn __len__(&self) -> usize {
        usize::from(self.native.cardinality())
    }

    pub fn __getitem__(&self, key: &PyAny) -> PyResult<bool> {
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

    pub fn __contains__(&self, key: &PyAny) -> PyResult<bool> {
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
        let values = self.native.to_values();
        values
            .into_iter()
            .map(|(it, _)| VariableId::from(ctx.as_native().find_state_variable(it).unwrap()))
            .collect()
    }

    /// The list of values for individual variables from `VertexModel.keys`.
    pub fn values(&self) -> Vec<bool> {
        let values = self.native.to_values();
        values.into_iter().map(|(_, it)| it).collect()
    }

    /// The list of key-value pairs represented in this model.
    pub fn items(&self) -> Vec<(VariableId, bool)> {
        let ctx = self.ctx.get();
        let values = self.native.to_values();
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
        let values = self.native.to_values();
        values
            .into_iter()
            .map(|(k, v)| {
                let k = VariableId::from(ctx.as_native().find_state_variable(k).unwrap());
                (k, v)
            })
            .collect()
    }
}

impl VertexModel {
    pub fn new_native(ctx: Py<SymbolicContext>, native: BddPartialValuation) -> VertexModel {
        VertexModel { ctx, native }
    }
}
