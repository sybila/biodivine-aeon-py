use super::PyBddVariableSet;
use crate::bindings::lib_bdd::{
    PyBdd, PyBddPartialValuation, PyBddValuation, PyBddVariable, PyBooleanExpression,
};
use crate::{throw_runtime_error, throw_type_error, AsNative};
use biodivine_lib_bdd::boolean_expression::BooleanExpression;
use biodivine_lib_bdd::{BddPartialValuation, BddVariable, BddVariableSet, BddVariableSetBuilder};
use pyo3::prelude::*;
use pyo3::types::PyList;

#[pymethods]
impl PyBddVariableSet {
    fn __str__(&self) -> PyResult<String> {
        let names = self
            .as_native()
            .variables()
            .into_iter()
            .map(|v| self.as_native().name_of(v))
            .collect::<Vec<_>>();
        Ok(format!("BddVariableSet{names:?}"))
    }

    fn __repr__(&self) -> PyResult<String> {
        self.__str__()
    }

    #[new]
    pub fn new(arg1: &PyAny) -> PyResult<PyBddVariableSet> {
        if let Ok(num_vars) = arg1.extract::<u16>() {
            Ok(BddVariableSet::new_anonymous(num_vars).into())
        } else if let Ok(len) = arg1.len() {
            let mut builder = BddVariableSetBuilder::new();
            for i in 0..len {
                let name = arg1.get_item(i)?;
                let name: String = name.extract()?;
                builder.make_variable(name.as_str());
            }
            Ok(builder.build().into())
        } else {
            throw_type_error(
                "Expected a number of (anonymous) variables or a list of variable names.",
            )
        }
    }

    pub fn eval_expression(&self, expression: &PyAny) -> PyResult<PyBdd> {
        if let Ok(expression) = expression.extract::<String>() {
            match BooleanExpression::try_from(expression.as_str()) {
                Ok(ex) => {
                    if let Some(bdd) = self.as_native().safe_eval_expression(&ex) {
                        Ok(bdd.into())
                    } else {
                        throw_runtime_error("Expression contains unknown variables.")
                    }
                }
                Err(err) => throw_runtime_error(format!("Invalid expression: {err}.")),
            }
        } else if let Ok(expression) = expression.extract::<PyBooleanExpression>() {
            if let Some(bdd) = self
                .as_native()
                .safe_eval_expression(expression.as_native())
            {
                Ok(bdd.into())
            } else {
                throw_runtime_error("Expression contains unknown variables.")
            }
        } else {
            throw_type_error("Expected String or BooleanExpression.")
        }
    }

    pub fn var_count(&self) -> u16 {
        self.as_native().num_vars()
    }

    pub fn all_variables(&self) -> Vec<PyBddVariable> {
        self.as_native()
            .variables()
            .into_iter()
            .map(|it| it.into())
            .collect()
    }

    pub fn find_variable(&self, variable: &PyAny) -> PyResult<Option<PyBddVariable>> {
        if let Ok(variable) = variable.extract::<PyBddVariable>() {
            Ok(Some(variable))
        } else if let Ok(name) = variable.extract::<String>() {
            Ok(self
                .as_native()
                .var_by_name(name.as_str())
                .map(|it| it.into()))
        } else {
            throw_type_error("Expected name or BddVariable.")
        }
    }

    pub fn name_of(&self, variable: PyBddVariable) -> String {
        self.as_native().name_of(variable.into())
    }

    pub fn mk_const(&self, value: bool) -> PyBdd {
        if value {
            self.as_native().mk_true().into()
        } else {
            self.as_native().mk_false().into()
        }
    }

    pub fn mk_literal(&self, variable: &PyAny, value: bool) -> PyResult<PyBdd> {
        let bdd_var: BddVariable = self.find_variable(variable)?.unwrap().into();
        Ok(self.as_native().mk_literal(bdd_var, value).into())
    }

    pub fn mk_valuation(&self, valuation: &PyAny) -> PyResult<PyBdd> {
        let valuation = PyBddValuation::from_python(valuation)?;
        let mut result = self.as_native().mk_true();
        for var in self.as_native().variables() {
            let value = valuation.as_native().value(var);
            let literal = self.as_native().mk_literal(var, value);
            result = result.and(&literal);
        }
        Ok(result.into())
    }

    pub fn mk_conjunctive_clause(&self, clause: &PyAny) -> PyResult<PyBdd> {
        let clause = PyBddPartialValuation::from_python(clause, Some(self))?;
        Ok(self
            .as_native()
            .mk_conjunctive_clause(clause.as_native())
            .into())
    }

    pub fn mk_disjunctive_clause(&self, clause: &PyAny) -> PyResult<PyBdd> {
        let clause = PyBddPartialValuation::from_python(clause, Some(self))?;
        Ok(self
            .as_native()
            .mk_disjunctive_clause(clause.as_native())
            .into())
    }

    pub fn mk_cnf(&self, clauses: &PyList) -> PyResult<PyBdd> {
        let mut native_clauses: Vec<BddPartialValuation> = Vec::new();
        for clause in clauses {
            native_clauses.push(PyBddPartialValuation::from_python(clause, Some(self))?.into());
        }
        Ok(self.as_native().mk_cnf(&native_clauses).into())
    }

    pub fn mk_dnf(&self, clauses: &PyList) -> PyResult<PyBdd> {
        let mut native_clauses: Vec<BddPartialValuation> = Vec::new();
        for clause in clauses {
            native_clauses.push(PyBddPartialValuation::from_python(clause, Some(self))?.into());
        }
        Ok(self.as_native().mk_dnf(&native_clauses).into())
    }

    pub fn transfer_from(&self, bdd: &PyBdd, ctx: &PyBddVariableSet) -> Option<PyBdd> {
        self.as_native()
            .transfer_from(bdd.as_native(), ctx.as_native())
            .map(|it| it.into())
    }
}
