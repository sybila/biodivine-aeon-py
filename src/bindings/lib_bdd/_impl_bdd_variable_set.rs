use super::PyBddVariableSet;
use crate::bindings::lib_bdd::{PyBdd, PyBddPartialValuation, PyBddVariable, PyBooleanExpression};
use crate::{throw_runtime_error, throw_type_error, AsNative};
use biodivine_lib_bdd::boolean_expression::BooleanExpression;
use biodivine_lib_bdd::{BddVariable, BddVariableSet, BddVariableSetBuilder};
use pyo3::prelude::*;
use pyo3::types::PyList;

#[pymethods]
impl PyBddVariableSet {
    #[new]
    pub fn new(variables: &PyAny) -> PyResult<PyBddVariableSet> {
        if let Ok(num_vars) = variables.extract::<u16>() {
            Ok(BddVariableSet::new_anonymous(num_vars).into())
        } else if let Ok(variables) = variables.extract::<Vec<String>>() {
            let mut builder = BddVariableSetBuilder::new();
            for v in variables {
                builder.make_variable(v.as_str());
            }
            Ok(builder.build().into())
        } else {
            throw_type_error("Exepcted list of variable names or a number.")
        }
    }

    fn __str__(&self) -> String {
        let variables = self.as_native().variables();
        let names = variables
            .into_iter()
            .map(|var| self.as_native().name_of(var))
            .collect::<Vec<_>>();
        format!("BddVariableSet({names:?})")
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }

    pub fn eval_expression(&self, expression: &PyAny) -> PyResult<PyBdd> {
        if let Ok(expression) = expression.extract::<PyRef<PyBooleanExpression>>() {
            let result = self
                .as_native()
                .safe_eval_expression(expression.as_native());
            if let Some(result) = result {
                Ok(result.into())
            } else {
                throw_runtime_error("Expression contains unknown variables.")
            }
        } else if let Ok(expr_string) = expression.extract::<&str>() {
            match BooleanExpression::try_from(expr_string) {
                Ok(expression) => {
                    let result = self.as_native().safe_eval_expression(&expression);
                    if let Some(result) = result {
                        Ok(result.into())
                    } else {
                        throw_runtime_error("Expression contains unknown variables.")
                    }
                }
                Err(error) => throw_runtime_error(format!("Invalid expression: {error}.")),
            }
        } else {
            throw_type_error("Expected string or `BooleanExpression`.")
        }
    }

    pub fn mk_const(&self, value: bool) -> PyBdd {
        if value {
            self.mk_true()
        } else {
            self.mk_false()
        }
    }

    pub fn mk_true(&self) -> PyBdd {
        self.as_native().mk_true().into()
    }

    pub fn mk_false(&self) -> PyBdd {
        self.as_native().mk_false().into()
    }

    pub fn mk_literal(&self, variable: &PyAny, value: bool) -> PyResult<PyBdd> {
        let bdd_var: BddVariable = self.find_variable(variable)?.unwrap().into();
        Ok(self.as_native().mk_literal(bdd_var, value).into())
    }

    pub fn mk_conjunctive_clause(&self, clause: &PyAny) -> PyResult<PyBdd> {
        let py = clause.py();
        let clause = PyBddPartialValuation::from_python_type(clause)?;
        let clause = clause.borrow(py);
        Ok(self
            .as_native()
            .mk_conjunctive_clause(clause.as_native())
            .into())
    }

    pub fn mk_disjunctive_clause(&self, clause: &PyAny) -> PyResult<PyBdd> {
        let py = clause.py();
        let clause = PyBddPartialValuation::from_python_type(clause)?;
        let clause = clause.borrow(py);
        Ok(self
            .as_native()
            .mk_disjunctive_clause(clause.as_native())
            .into())
    }

    pub fn mk_cnf(&self, clauses: &PyList) -> PyResult<PyBdd> {
        let mut result = self.as_native().mk_true();
        for clause in clauses {
            let clause = self.mk_disjunctive_clause(clause)?.into();
            result = result.and(&clause);
        }
        Ok(result.into())
    }

    pub fn mk_dnf(&self, clauses: &PyList) -> PyResult<PyBdd> {
        let mut result = self.as_native().mk_false();
        for clause in clauses {
            let clause = self.mk_conjunctive_clause(clause)?.into();
            result = result.or(&clause);
        }
        Ok(result.into())
    }

    pub fn var_count(&self) -> u16 {
        self.as_native().num_vars()
    }

    pub fn find_variable(&self, variable: &PyAny) -> PyResult<Option<PyBddVariable>> {
        if let Ok(variable) = variable.extract::<PyBddVariable>() {
            Ok(Some(variable))
        } else if let Ok(name) = variable.extract::<&str>() {
            let variable = if let Some(variable) = self.as_native().var_by_name(name) {
                Some(variable.into())
            } else {
                None
            };
            Ok(variable)
        } else {
            throw_type_error("Expected name or BddVariable.")
        }
    }

    pub fn get_variable_name(&self, variable: PyBddVariable) -> String {
        self.as_native().name_of(variable.into())
    }

    pub fn all_variables(&self) -> Vec<PyBddVariable> {
        self.as_native()
            .variables()
            .into_iter()
            .map(|it| it.into())
            .collect()
    }
}
