use crate::bindings::lib_bdd::{
    PyBdd, PyBddPartialValuation, PyBddValuation, PyBddVariable, PyBddVariableSet,
};
use crate::bindings::lib_param_bn::{
    PyBooleanNetwork, PyFnUpdate, PyParameterId, PySymbolicContext, PyVariableId,
};
use crate::{throw_runtime_error, throw_type_error, AsNative};
use biodivine_lib_bdd::{BddPartialValuation, BddValuation, BddVariable, BddVariableSet};
use biodivine_lib_param_bn::symbolic_async_graph::SymbolicContext;
use biodivine_lib_param_bn::{FnUpdate, VariableId};
use pyo3::prelude::*;
use std::collections::HashMap;

fn convert_var_list(variables: &[BddVariable]) -> Vec<PyBddVariable> {
    variables.iter().map(|it| (*it).into()).collect()
}

fn read_arg_list(variables: &[PyVariableId]) -> Vec<VariableId> {
    variables.iter().map(|it| (*it).into()).collect()
}

fn read_fn_arg_list(variables: &[PyFnUpdate]) -> Vec<FnUpdate> {
    variables.iter().map(|it| (*it).clone().into()).collect()
}

/// A helper method that reads a `BddValuation` object from `PyAny`, ensuring that the
/// required `variables` are present in the result when the input is only a partial valuation.
fn read_valuation(
    vars: &BddVariableSet,
    variables: &[BddVariable],
    valuation: &PyAny,
) -> PyResult<BddValuation> {
    if let Ok(valuation) = PyBddValuation::from_python(valuation) {
        Ok(valuation.into())
    } else if let Ok(valuation) = PyBddPartialValuation::from_python(valuation, None) {
        let valuation: BddPartialValuation = valuation.into();
        let mut result = BddValuation::all_false(vars.num_vars());
        for var in variables {
            if let Some(value) = valuation.get_value(*var) {
                result.set_value(*var, value)
            } else {
                return throw_type_error(
                    "The given BDD valuation is missing some required variables.",
                );
            }
        }
        Ok(result)
    } else {
        throw_type_error("Expected either a partial or complete BDD valuation.")
    }
}

#[pymethods]
impl PySymbolicContext {
    #[new]
    pub fn new(
        network: &PyBooleanNetwork,
        extra_state_variables: Option<HashMap<PyVariableId, u16>>,
    ) -> PyResult<PySymbolicContext> {
        let context = if let Some(extra_state_variables) = extra_state_variables {
            let extra_state_variables = extra_state_variables
                .into_iter()
                .map(|(k, v)| (VariableId::from(k), v))
                .collect();
            SymbolicContext::with_extra_state_variables(network.as_native(), &extra_state_variables)
        } else {
            SymbolicContext::new(network.as_native())
        };
        match context {
            Ok(context) => Ok(context.into()),
            Err(err) => throw_runtime_error(err),
        }
    }

    pub fn num_state_variables(&self) -> usize {
        self.as_native().num_state_variables()
    }

    pub fn num_parameter_variables(&self) -> usize {
        self.as_native().num_parameter_variables()
    }

    pub fn num_extra_state_variables(&self) -> usize {
        self.as_native().num_extra_state_variables()
    }

    pub fn bdd_variable_set(&self) -> PyBddVariableSet {
        self.as_native().bdd_variable_set().clone().into()
    }

    pub fn state_variables(&self) -> Vec<PyBddVariable> {
        convert_var_list(self.as_native().state_variables())
    }

    pub fn all_extra_state_variables(&self) -> Vec<PyBddVariable> {
        convert_var_list(self.as_native().all_extra_state_variables())
    }

    pub fn extra_state_variables(&self, variable: PyVariableId) -> Vec<PyBddVariable> {
        convert_var_list(self.as_native().extra_state_variables(variable.into()))
    }

    pub fn extra_state_variables_by_offset(
        &self,
        offset: usize,
    ) -> Vec<(PyVariableId, PyBddVariable)> {
        self.as_native()
            .extra_state_variables_by_offset(offset)
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect()
    }

    pub fn get_state_variable(&self, variable: PyVariableId) -> PyBddVariable {
        self.as_native().get_state_variable(variable.into()).into()
    }

    pub fn get_extra_state_variable(&self, variable: PyVariableId, offset: usize) -> PyBddVariable {
        self.as_native()
            .get_extra_state_variable(variable.into(), offset)
            .into()
    }

    pub fn get_implicit_function_table(
        &self,
        variable: PyVariableId,
    ) -> Vec<(Vec<bool>, PyBddVariable)> {
        self.as_native()
            .get_implicit_function_table(variable.into())
            .into_iter()
            .map(|(k, v)| (k, v.into()))
            .collect()
    }

    pub fn get_explicit_function_table(
        &self,
        parameter: PyParameterId,
    ) -> Vec<(Vec<bool>, PyBddVariable)> {
        self.as_native()
            .get_explicit_function_table(parameter.into())
            .into_iter()
            .map(|(k, v)| (k, v.into()))
            .collect()
    }

    pub fn mk_constant(&self, value: bool) -> PyBdd {
        self.as_native().mk_constant(value).into()
    }

    pub fn mk_state_variable_is_true(&self, variable: PyVariableId) -> PyBdd {
        self.as_native()
            .mk_state_variable_is_true(variable.into())
            .into()
    }

    pub fn mk_extra_state_variable_is_true(&self, variable: PyVariableId, offset: usize) -> PyBdd {
        self.as_native()
            .mk_extra_state_variable_is_true(variable.into(), offset)
            .into()
    }

    pub fn mk_uninterpreted_function_is_true(
        &self,
        parameter: PyParameterId,
        arguments: Vec<PyFnUpdate>,
    ) -> PyBdd {
        self.as_native()
            .mk_uninterpreted_function_is_true(parameter.into(), &read_fn_arg_list(&arguments))
            .into()
    }

    pub fn mk_implicit_function_is_true(
        &self,
        variable: PyVariableId,
        arguments: Vec<PyVariableId>,
    ) -> PyBdd {
        self.as_native()
            .mk_implicit_function_is_true(variable.into(), &read_arg_list(&arguments))
            .into()
    }

    pub fn mk_update_function_is_true(&self, function: &PyFnUpdate) -> PyBdd {
        self.as_native()
            .mk_fn_update_true(function.as_native())
            .into()
    }

    pub fn instantiate_implicit_function(
        &self,
        valuation: &PyAny,
        variable: PyVariableId,
        arguments: Vec<PyVariableId>,
    ) -> PyResult<PyBdd> {
        let required_variables = self
            .as_native()
            .get_implicit_function_table(variable.into())
            .symbolic_variables();
        let valuation = read_valuation(
            self.as_native().bdd_variable_set(),
            required_variables,
            valuation,
        )?;
        Ok(self
            .as_native()
            .instantiate_implicit_function(&valuation, variable.into(), &read_arg_list(&arguments))
            .into())
    }

    pub fn instantiate_uninterpreted_function(
        &self,
        valuation: &PyAny,
        parameter: PyParameterId,
        arguments: Vec<PyFnUpdate>,
    ) -> PyResult<PyBdd> {
        let required_variables = self
            .as_native()
            .get_explicit_function_table(parameter.into())
            .symbolic_variables();
        let valuation = read_valuation(
            self.as_native().bdd_variable_set(),
            required_variables,
            valuation,
        )?;
        Ok(self
            .as_native()
            .instantiate_uninterpreted_function(
                &valuation,
                parameter.into(),
                &read_fn_arg_list(&arguments),
            )
            .into())
    }

    pub fn instantiate_fn_update(
        &self,
        valuation: &PyAny,
        function: &PyFnUpdate,
    ) -> PyResult<PyBdd> {
        let mut required_variables = Vec::new();
        for param in function.as_native().collect_parameters() {
            required_variables.extend(
                self.as_native()
                    .get_explicit_function_table(param)
                    .symbolic_variables()
                    .iter()
                    .cloned(),
            )
        }
        let valuation = read_valuation(
            self.as_native().bdd_variable_set(),
            &required_variables,
            valuation,
        )?;
        Ok(self
            .as_native()
            .instantiate_fn_update(&valuation, function.as_native())
            .into())
    }

    pub fn transfer_from(&self, bdd: &PyBdd, context: &PySymbolicContext) -> Option<PyBdd> {
        self.as_native()
            .transfer_from(bdd.as_native(), context.as_native())
            .map(|it| it.into())
    }
}
