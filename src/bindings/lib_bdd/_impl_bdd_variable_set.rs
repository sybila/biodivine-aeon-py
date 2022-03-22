use super::PyBddVariableSet;
use crate::bindings::lib_bdd::{PyBdd, PyBddVariable, PyBooleanExpression};
use crate::{throw_runtime_error, throw_type_error, AsNative};
use biodivine_lib_bdd::{BddPartialValuation, BddVariable, BddVariableSet, BddVariableSetBuilder};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

impl From<BddVariableSet> for PyBddVariableSet {
    fn from(value: BddVariableSet) -> Self {
        PyBddVariableSet(value)
    }
}

impl From<PyBddVariableSet> for BddVariableSet {
    fn from(value: PyBddVariableSet) -> Self {
        value.0
    }
}

impl AsNative<BddVariableSet> for PyBddVariableSet {
    fn as_native(&self) -> &BddVariableSet {
        &self.0
    }

    fn as_native_mut(&mut self) -> &mut BddVariableSet {
        &mut self.0
    }
}

#[pymethods]
impl PyBddVariableSet {
    fn __str__(&self) -> PyResult<String> {
        let names = self
            .as_native()
            .variables()
            .into_iter()
            .map(|v| self.as_native().name_of(v))
            .collect::<Vec<_>>();
        Ok(format!("BddVariableSet{:?}", names))
    }

    fn __repr__(&self) -> PyResult<String> {
        self.__str__()
    }

    /// Create a new `BddVariableSet` by supplying either a number of variables (that will be
    /// automatically named `x_0`, `x_1`, etc.), or a list of string names.
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

    /// Evaluate the given `BooleanExpression` into a `Bdd`. If the argument is a `String`, it is
    /// first parsed into an expression and then evaluated.
    ///
    /// Note that the method panics when the string cannot be correctly parsed.
    pub fn eval_expression(&self, expression: &PyAny) -> PyResult<PyBdd> {
        if let Ok(expression) = expression.extract::<String>() {
            Ok(self
                .as_native()
                .eval_expression_string(expression.as_str())
                .into())
        } else if let Ok(expression) = expression.extract::<PyBooleanExpression>() {
            Ok(self
                .as_native()
                .eval_expression(expression.as_native())
                .into())
        } else {
            throw_type_error("Expected String or BooleanExpression.")
        }
    }

    /// Get the total number of variables in this `BddVariableSet`.
    pub fn var_count(&self) -> u16 {
        self.as_native().num_vars()
    }

    /// Get the full list of `BddVariable` identifiers managed by this `BddVariableSet`.
    pub fn all_variables(&self, py: Python) -> PyObject {
        let variables: Vec<PyBddVariable> = self
            .as_native()
            .variables()
            .into_iter()
            .map(|v| v.into())
            .collect();
        variables.into_py(py)
    }

    /// Get a `BddVariable` identifier based on variable name. Raises an exception if a variable
    /// with the given name does not exist.
    ///
    /// For convenience during type conversion, you can also supply a `BddVariable`, in which case
    /// you get the same `BddVariable` as a result.
    pub fn find_variable(&self, variable: &PyAny) -> PyResult<PyBddVariable> {
        if let Ok(variable) = variable.extract::<PyBddVariable>() {
            Ok(variable)
        } else if let Ok(name) = variable.extract::<String>() {
            if let Some(variable) = self.as_native().var_by_name(name.as_str()) {
                Ok(variable.into())
            } else {
                throw_runtime_error(format!("Variable {} not found.", name))
            }
        } else {
            throw_type_error("Expected name or BddVariable.")
        }
    }

    /// Return the name of the given `BddVariable` identifier in this `BddVariableSet`.
    pub fn name_of(&self, variable: PyBddVariable) -> String {
        self.as_native().name_of(variable.into())
    }

    /// Create a `Bdd` corresponding to a constant function `True` or `False`.
    pub fn mk_const(&self, value: bool) -> PyBdd {
        if value {
            self.as_native().mk_true().into()
        } else {
            self.as_native().mk_false().into()
        }
    }

    /// Create a `Bdd` corresponding to a literal, i.e. `x` or `!x` for a particular
    /// variable `x`.
    ///
    /// The variable can be specified either using its name, or a `BddVariable` identifier.
    pub fn mk_literal(&self, variable: &PyAny, value: bool) -> PyResult<PyBdd> {
        let bdd_var: BddVariable = self.find_variable(variable)?.into();
        Ok(self.as_native().mk_literal(bdd_var, value).into())
    }

    /// Create a `Bdd` representing a conjunctive clause from a partial valuation of variables.
    ///
    /// This partial valuation is a dictionary mapping variables (either a name or a `BddVariable`
    /// identifier) to a Boolean value.
    ///
    /// Variables that do not appear in the dictionary do not appear in the clause.
    pub fn mk_conjunctive_clause(&self, items: &PyDict) -> PyResult<PyBdd> {
        let mut partial_valuation = BddPartialValuation::empty();
        for (key, value) in items {
            let var: BddVariable = self.find_variable(key)?.into();
            let value = value.extract::<bool>()?;
            partial_valuation.set_value(var, value);
        }

        Ok(self
            .as_native()
            .mk_conjunctive_clause(&partial_valuation)
            .into())
    }

    /// Create a `Bdd` representing a disjunctive clause from a partial valuation of variables.
    ///
    /// This partial valuation is a dictionary mapping variables (either a name or a `BddVariable`
    /// identifier) to a Boolean value.
    ///
    /// Variables that do not appear in the dictionary do not appear in the clause.
    pub fn mk_disjunctive_clause(&self, items: &PyDict) -> PyResult<PyBdd> {
        let mut partial_valuation = biodivine_lib_bdd::BddPartialValuation::empty();
        for (key, value) in items {
            let var: BddVariable = self.find_variable(key)?.into();
            let value = value.extract::<bool>()?;
            partial_valuation.set_value(var, value);
        }

        Ok(self
            .as_native()
            .mk_disjunctive_clause(&partial_valuation)
            .into())
    }

    /// Create a CNF formula. The formula is given as a list of disjunctive clauses.
    ///
    /// Each clause is given as a dictionary, mapping variables (either a name or a `BddVariable`
    /// identifier) to Boolean values.
    ///
    pub fn mk_cnf(&self, clauses: &PyList) -> PyResult<PyBdd> {
        let mut result = self.as_native().mk_true();
        for clause in clauses {
            let clause = self.mk_disjunctive_clause(clause.extract()?)?.into();
            result = result.and(&clause);
        }
        Ok(result.into())
    }

    /// Create a DNF formula. The formula is given as a list of conjunctive clauses.
    ///
    /// Each clause is given as a dictionary, mapping variables (either a name or a `BddVariable`
    /// identifier) to Boolean values.
    pub fn mk_dnf(&self, clauses: &PyList) -> PyResult<PyBdd> {
        let mut result = self.as_native().mk_false();
        for clause in clauses {
            let clause = self.mk_conjunctive_clause(clause.extract()?)?.into();
            result = result.or(&clause);
        }
        Ok(result.into())
    }
}
