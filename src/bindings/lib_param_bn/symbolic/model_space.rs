use std::collections::HashMap;

use pyo3::{pyclass, pymethods, Bound, Py, PyAny, PyResult, Python};

use crate::bindings::lib_bdd::bdd_valuation::BddPartialValuation;
use crate::bindings::lib_param_bn::symbolic::set_spaces::SpaceSet;
use crate::bindings::lib_param_bn::symbolic::symbolic_space_context::SymbolicSpaceContext;
use crate::bindings::lib_param_bn::variable_id::VariableId;
use crate::bindings::lib_param_bn::NetworkVariableContext;
use crate::{throw_index_error, AsNative};

/// Represents a single space stored in a `SpaceSet` (or a `ColoredSpaceSet`), or a projection
/// of said space to the chosen variables.
///
/// Behaves like an immutable dictionary: Boolean variable values can be queried using
/// a `VariableId`, a string name, or a `BddVariable`. If the value is unconstrained, the result
/// is `None`. If the variable is projected away, the operation throws an `IndexError`.
#[pyclass(module = "biodivine_aeon", frozen)]
#[derive(Clone)]
pub struct SpaceModel {
    ctx: Py<SymbolicSpaceContext>,
    native: biodivine_lib_bdd::BddPartialValuation,
}

#[pymethods]
impl SpaceModel {
    /// Access the underlying `SymbolicSpaceContext` connected to this `SpaceModel`.
    pub fn __ctx__(&self) -> Py<SymbolicSpaceContext> {
        self.ctx.clone()
    }

    pub fn __str__(&self) -> String {
        let ctx = self.ctx.get().as_native().inner_context();
        let items = self
            .to_values()
            .into_iter()
            .map(|(var, value)| {
                let name = ctx.get_network_variable_name(var);
                if let Some(value) = value {
                    format!("'{}': {}", name, i32::from(value))
                } else {
                    format!("'{name}': *")
                }
            })
            .collect::<Vec<_>>();
        format!("SpaceModel({{{}}})", items.join(", "))
    }

    pub fn __repr__(&self) -> String {
        self.__str__()
    }

    /// The number of actual values in this `VertexModel` (i.e. retained network variables).
    pub fn __len__(&self) -> usize {
        self.to_values().len()
    }

    pub fn __getitem__(&self, key: &Bound<'_, PyAny>, py: Python) -> PyResult<Option<bool>> {
        let ctx = self.ctx.borrow(py);
        let variable = ctx.as_ref().resolve_network_variable(key)?;
        let p = ctx.as_native().get_positive_variable(variable);
        let n = ctx.as_native().get_negative_variable(variable);
        let p = self.native.get_value(p);
        let n = self.native.get_value(n);
        match (p, n) {
            (Some(true), Some(false)) => Ok(Some(true)),
            (Some(false), Some(true)) => Ok(Some(false)),
            (Some(true), Some(true)) => Ok(None),
            _ => throw_index_error(format!(
                "Variable `{}` not available in this projection.",
                ctx.as_ref().as_native().get_network_variable_name(variable)
            )),
        }
    }

    pub fn __contains__(&self, key: &Bound<'_, PyAny>, py: Python) -> PyResult<bool> {
        let ctx = self.ctx.borrow(py);
        let variable = ctx.as_ref().resolve_network_variable(key)?;
        let p = ctx.as_native().get_positive_variable(variable);
        let n = ctx.as_native().get_negative_variable(variable);
        Ok(self.native.has_value(p) && self.native.has_value(n))
    }

    /// The actual "retained" network variables in this model.
    ///
    /// This is the list of all network variables if no projection was applied.
    pub fn keys(&self) -> Vec<VariableId> {
        let values = self.to_values();
        values
            .into_iter()
            .map(|(it, _)| VariableId::from(it))
            .collect()
    }

    /// The list of values for individual variables from `SpaceModel.keys`.
    pub fn values(&self) -> Vec<Option<bool>> {
        let values = self.to_values();
        values.into_iter().map(|(_, it)| it).collect()
    }

    /// The list of key-value pairs represented in this symbolic model.
    pub fn items(&self) -> Vec<(VariableId, Option<bool>)> {
        self.to_values()
            .into_iter()
            .map(|(a, b)| (VariableId::from(a), b))
            .collect()
    }

    /// The same as `SpaceModel.items`, but returns a dictionary instead.
    pub fn to_dict(&self) -> HashMap<VariableId, Option<bool>> {
        self.to_values()
            .into_iter()
            .map(|(a, b)| (VariableId::from(a), b))
            .collect()
    }

    /// The same as `SpaceModel.to_dict`, but the keys in the dictionary are names, not IDs.
    pub fn to_named_dict(&self) -> HashMap<String, Option<bool>> {
        let ctx = self.ctx.get().as_native().inner_context();
        self.to_values()
            .into_iter()
            .map(|(a, b)| {
                let name = ctx.get_network_variable_name(a);
                (name, b)
            })
            .collect()
    }

    /// Return the underlying `BddPartialValuation` for this symbolic model.
    pub fn to_valuation(&self, py: Python) -> BddPartialValuation {
        BddPartialValuation::new_raw(
            self.ctx.borrow(py).as_ref().bdd_variable_set(),
            self.native.clone(),
        )
    }

    /// Return a `SpaceSet` where all the variables are fixed according
    /// to the values in this `SpaceModel`. Variables that are not present in the `SpaceModel`
    /// are unrestricted, meaning their value can be any of `0`, `1`, and `*`.
    pub fn to_symbolic(&self) -> SpaceSet {
        let ctx = self.ctx.get();
        let bdd = ctx
            .as_native()
            .bdd_variable_set()
            .mk_conjunctive_clause(&self.native);
        let bdd = ctx.as_native().mk_unit_bdd().and(&bdd);
        let native = biodivine_lib_param_bn::trap_spaces::NetworkSpaces::new(bdd, ctx.as_native());
        SpaceSet::wrap_native(self.ctx.clone(), native)
    }
}

impl SpaceModel {
    pub fn new_native(
        ctx: Py<SymbolicSpaceContext>,
        native: biodivine_lib_bdd::BddPartialValuation,
    ) -> SpaceModel {
        SpaceModel { ctx, native }
    }

    fn to_values(&self) -> Vec<(biodivine_lib_param_bn::VariableId, Option<bool>)> {
        // Only return extra variables:
        let mut result = Vec::new();
        let ctx = self.ctx.get().as_native();
        let ctx_inner = ctx.inner_context();
        for var in ctx_inner.network_variables() {
            let (p, n) = (
                ctx.get_positive_variable(var),
                ctx.get_negative_variable(var),
            );
            let p = self.native.get_value(p);
            let n = self.native.get_value(n);
            match (p, n) {
                (Some(true), Some(false)) => result.push((var, Some(true))),
                (Some(false), Some(true)) => result.push((var, Some(false))),
                (Some(true), Some(true)) => result.push((var, None)),
                _ => continue,
            }
        }
        result
    }
}
