extern crate biodivine_lib_bdd;

use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

#[pyclass]
#[derive(Clone)]
pub struct Bdd(biodivine_lib_bdd::Bdd);

impl From<biodivine_lib_bdd::Bdd> for Bdd {
    fn from(value: biodivine_lib_bdd::Bdd) -> Self {
        Bdd(value)
    }
}

impl From<Bdd> for biodivine_lib_bdd::Bdd {
    fn from(value: Bdd) -> Self {
        value.0
    }
}

#[pymethods]
impl Bdd {
    /// Compute a logical negation of this Bdd.
    pub fn not(&self) -> Bdd {
        self.0.not().into()
    }

    /// Compute a logical conjunction of two formulas.
    pub fn and(&self, other: &Bdd) -> Bdd {
        self.0.and(&other.0).into()
    }

    /// Compute a logical disjunction of two formulas.
    pub fn or(&self, other: &Bdd) -> Bdd {
        self.0.or(&other.0).into()
    }

    /// Compute a logical implication of two formulas.
    pub fn imp(&self, other: &Bdd) -> Bdd {
        self.0.imp(&other.0).into()
    }

    /// Compute a logical equivalence of two formulas.
    pub fn iff(&self, other: &Bdd) -> Bdd {
        self.0.iff(&other.0).into()
    }

    /// Compute a logical xor of two formulas.
    pub fn xor(&self, other: &Bdd) -> Bdd {
        self.0.xor(&other.0).into()
    }

    /// Compute a logical conjunction of this formula with a negated second formula.
    pub fn and_not(&self, other: &Bdd) -> Bdd {
        self.0.and_not(&other.0).into()
    }

    /// Compute a projection over the given Bdd variable.
    pub fn var_project(&self, variable: &BddVariable) -> Bdd {
        self.0.var_project(variable.0).into()
    }

    /// Compute a projection over all the given Bdd variables.
    pub fn project(&self, variables: &PyList) -> PyResult<Bdd> {
        let mut vars = Vec::with_capacity(variables.len());
        for var in variables {
            vars.push(var.extract::<BddVariable>()?.0);
        }
        Ok(self.0.project(&vars).into())
    }

    /// Compute a pick operation for the given Bdd variable (biased towards 0).
    ///
    ///
    /// See the original Rust library docs for operation details.
    pub fn var_pick(&self, variable: &BddVariable) -> Bdd {
        self.0.var_pick(variable.0).into()
    }

    /// Compute a pick operation for all the given Bdd variables (biased towards 0).
    ///
    /// See the original Rust library docs for operation details.
    pub fn pick(&self, variables: &PyList) -> PyResult<Bdd> {
        let mut vars = Vec::with_capacity(variables.len());
        for var in variables {
            vars.push(var.extract::<BddVariable>()?.0);
        }
        Ok(self.0.pick(&vars).into())
    }

    /// Compute a selection for the given Bdd variable with the given value.
    pub fn var_select(&self, var: &BddVariable, value: bool) -> Bdd {
        self.0.var_select(var.0, value).into()
    }

    /// Compute a selection of the given partial valuation.
    ///
    /// The partial valuation is a dictionary ` { BddVariable: bool }` which specifies variable
    /// values that should be fixed.
    pub fn select(&self, values: &PyDict) -> PyResult<Bdd> {
        let mut valuation = Vec::new();
        for (k, v) in values {
            let key = k.extract::<BddVariable>()?;
            let value = v.extract::<bool>()?;
            valuation.push((key.0, value));
        }
        Ok(self.0.select(&valuation).into())
    }

    /// Print this Bdd to a .dot file that can be visualised using graphviz.
    ///
    /// Variable names are resolved from a given `BddVariableSet`. If not given, the names
    /// default to `x_0`, `x_1`, etc.
    #[args(variables = "None")]
    pub fn to_dot(&self, variables: Option<&BddVariableSet>) -> String {
        if let Some(value) = variables {
            self.0.to_dot_string(&value.0, true)
        } else {
            let variables = biodivine_lib_bdd::BddVariableSet::new_anonymous(self.0.num_vars());
            self.0.to_dot_string(&variables, true)
        }
        /*if let Ok(value) = variables.extract::<BddVariableSet>() {
            self.0.to_dot_string(&value.0, true)
        } else {
            let variables = biodivine_lib_bdd::BddVariableSet::new_anonymous(self.0.num_vars());
            self.0.to_dot_string(&variables, true)
        }*/
    }

    /// Produces a raw string representation of this Bdd that can be saved into a file or sent
    /// over the network.
    pub fn to_raw_string(&self) -> String {
        self.0.to_string()
    }

    /// Read a Bdd from a raw string representation.
    #[staticmethod]
    pub fn from_raw_string(data: &str) -> Bdd {
        // This will panic on error, but the necessary function to extract the error
        // is private in the Bdd struct (for now).
        biodivine_lib_bdd::Bdd::from_string(data).into()
    }

    /// Check if this formula represents a single conjunctive clause (a single path in Bdd format).
    pub fn is_conjunctive_clause(&self) -> bool {
        self.0.is_clause()
    }

    /// Check that this Bdd represents a single valuation with all variables fixed.
    pub fn is_valuation(&self) -> bool {
        self.0.is_valuation()
    }

    /// Return the number of nodes in this Bdd.
    pub fn node_count(&self) -> usize {
        self.0.size()
    }

    /// Return the number of variables supported by this Bdd (not all have to be used).
    pub fn var_count(&self) -> usize {
        self.0.is_valuation() as usize
    }

    /// True is this Bdd represents a tautology.
    pub fn is_true(&self) -> bool {
        self.0.is_true()
    }

    /// True is this Bdd represents a contradiction.
    pub fn is_false(&self) -> bool {
        self.0.is_false()
    }

    /// Return an count of satisfying valuations in this Bdd (the number may be approximate
    /// when the Bdd is sufficiently large).
    pub fn cardinality(&self) -> f64 {
        self.0.cardinality()
    }

    /// Return a bool vector representing one satisfying valuation of this Bdd.
    ///
    /// If the Bdd is not satisfiable, the vector is empty
    pub fn sat_witness(&self, py: Python) -> PyObject {
        if let Some(valuation) = self.0.sat_witness() {
            valuation.vector().into_py(py)
        } else {
            Vec::<bool>::new().into_py(py)
        }
    }

    /// Convert this Bdd into a Boolean expression.
    ///
    /// Note that this is not doing any fancy minimisation of the formula, so the result can
    /// be very large!
    ///
    /// The first argument is a variable set that will supply the variable names. If it is not
    /// given, then default names are used.
    #[args(variables = "None")]
    pub fn to_boolean_expression(&self, variables: Option<&BddVariableSet>) -> BooleanExpression {
        if let Some(variables) = variables {
            self.0.to_boolean_expression(&variables.0).into()
        } else {
            let variables = biodivine_lib_bdd::BddVariableSet::new_anonymous(self.0.num_vars());
            self.0.to_boolean_expression(&variables).into()
        }
    }
}

// BooleanVariableSet ****************

#[pyclass]
#[derive(Clone)]
pub struct BddVariableSet(biodivine_lib_bdd::BddVariableSet);

impl From<BddVariableSet> for biodivine_lib_bdd::BddVariableSet {
    fn from(value: BddVariableSet) -> Self {
        value.0
    }
}

impl From<biodivine_lib_bdd::BddVariableSet> for BddVariableSet {
    fn from(value: biodivine_lib_bdd::BddVariableSet) -> Self {
        BddVariableSet(value)
    }
}

#[pymethods]
impl BddVariableSet {
    /// Create a new BddVariableSet by supplying either a number of variables (which will be
    /// named x_0, x_1, etc.), or a list of names.
    #[new]
    pub fn new(arg1: &PyAny) -> PyResult<BddVariableSet> {
        if let Ok(num_vars) = arg1.extract::<u16>() {
            Ok(BddVariableSet(
                biodivine_lib_bdd::BddVariableSet::new_anonymous(num_vars),
            ))
        } else if let Ok(len) = arg1.len() {
            let mut builder = biodivine_lib_bdd::BddVariableSetBuilder::new();
            for i in 0..len {
                let name = arg1.get_item(i)?;
                let name: String = name.extract()?;
                builder.make_variable(name.as_str());
            }
            Ok(BddVariableSet(builder.build()))
        } else {
            Err(PyTypeError::new_err(
                "Expected number of variables or a list of variable names.",
            ))
        }
    }

    /// Evaluate the given Boolean expression into a Bdd. If the argument is a string, it is
    /// first parsed into an expression and then evaluated.
    pub fn eval_expression(&self, expression: &PyAny) -> PyResult<Bdd> {
        if let Ok(expression) = expression.extract::<String>() {
            Ok(self.0.eval_expression_string(&expression).into())
        } else if let Ok(expression) = expression.extract::<BooleanExpression>() {
            let ex: biodivine_lib_bdd::boolean_expression::BooleanExpression = expression.into();
            Ok(self.0.eval_expression(&ex).into())
        } else {
            Err(PyTypeError::new_err(
                "Expected string or a BooleanExpression instance.",
            ))
        }
    }

    /// Get the total number of variables in this variable set.
    pub fn num_vars(&self) -> u16 {
        self.0.num_vars()
    }

    /// Get the full list of variables in this set.
    pub fn all_variables(&self, py: Python) -> PyObject {
        let variables: Vec<BddVariable> =
            self.0.variables().into_iter().map(|v| v.into()).collect();
        variables.into_py(py)
    }

    /// Get a variable reference using its name. Raises an exception if a variable is not found.
    pub fn find_variable(&self, name: String) -> PyResult<BddVariable> {
        if let Some(var) = self.0.var_by_name(name.as_str()) {
            Ok(BddVariable(var))
        } else {
            Err(PyTypeError::new_err(format!(
                "Variable {} not found.",
                name
            )))
        }
    }

    /// Return the name of the given variable in this set.
    pub fn name_of(&self, variable: BddVariable) -> String {
        self.0.name_of(variable.into())
    }

    /// Create a Bdd corresponding to a constant function.
    pub fn mk_const(&self, value: bool) -> Bdd {
        if value {
            self.0.mk_true().into()
        } else {
            self.0.mk_false().into()
        }
    }

    /// Create a Bdd corresponding to a literal function, i.e. `x` or `!x` for a particular
    /// variable `x`.
    pub fn mk_literal(&self, variable: &PyAny, value: bool) -> PyResult<Bdd> {
        let bdd_var: biodivine_lib_bdd::BddVariable;
        if let Ok(variable) = variable.extract::<BddVariable>() {
            bdd_var = variable.into();
        } else if let Ok(name) = variable.extract::<String>() {
            if let Some(value) = self.0.var_by_name(name.as_str()) {
                bdd_var = value;
            } else {
                return Err(PyTypeError::new_err(format!("Unknown variable {}.", name)));
            }
        } else {
            return Err(PyTypeError::new_err(
                "Expected string or Bdd variable as first argument.",
            ));
        }

        Ok(self.0.mk_literal(bdd_var, value).into())
    }

    /// Create a Bdd representing a conjunctive clause. The function takes one argument.
    /// This argument is a dictionary mapping variables (a name or a Bdd variable object) to either
    /// `true` or `false`, depending on whether they appear in the clause as positive or
    /// negative literals.
    ///
    /// Variables which do not appear in the dictionary do not appear in the clause.
    pub fn mk_conjunctive_clause(&self, items: &PyDict) -> PyResult<Bdd> {
        let mut partial_valuation = biodivine_lib_bdd::BddPartialValuation::empty();
        for (key, value) in items {
            let var: biodivine_lib_bdd::BddVariable;
            if let Ok(name) = key.extract::<String>() {
                var = self.find_variable(name)?.into();
            } else if let Ok(variable) = key.extract::<BddVariable>() {
                var = variable.into();
            } else {
                return Err(PyTypeError::new_err(
                    "A key must ba a Bdd variable or a name.",
                ));
            }

            let value = value.extract::<bool>()?;

            partial_valuation.set_value(var, value);
        }

        Ok(self.0.mk_conjunctive_clause(&partial_valuation).into())
    }

    /// Create a Bdd representing a disjunctive clause. The argument is a partial valuation
    /// of Bdd variables in the clause (see `mk_conjunctive_clause`).
    pub fn mk_disjunctive_clause(&self, items: &PyDict) -> PyResult<Bdd> {
        let mut partial_valuation = biodivine_lib_bdd::BddPartialValuation::empty();
        for (key, value) in items {
            let var: biodivine_lib_bdd::BddVariable;
            if let Ok(name) = key.extract::<String>() {
                var = self.find_variable(name)?.into();
            } else if let Ok(variable) = key.extract::<BddVariable>() {
                var = variable.into();
            } else {
                return Err(PyTypeError::new_err(
                    "A key must ba a Bdd variable or a name.",
                ));
            }

            let value = value.extract::<bool>()?;

            partial_valuation.set_value(var, value);
        }

        Ok(self.0.mk_disjunctive_clause(&partial_valuation).into())
    }

    /// Create a conjunction of a list of disjunctive clauses (see `mk_disjunctive_clause` for
    /// the supported argument format).
    pub fn mk_cnf(&self, clauses: &PyList) -> PyResult<Bdd> {
        let mut result = self.0.mk_true();
        for clause in clauses {
            let clause: biodivine_lib_bdd::Bdd =
                self.mk_disjunctive_clause(clause.extract()?)?.into();
            result = result.and(&clause);
        }
        Ok(result.into())
    }

    /// Create a disjunction of a list of conjunctive clauses (see `mk_conjunctive_clause` for
    /// the supported argument format).
    pub fn mk_dnf(&self, clauses: &PyList) -> PyResult<Bdd> {
        let mut result = self.0.mk_false();
        for clause in clauses {
            let clause: biodivine_lib_bdd::Bdd =
                self.mk_conjunctive_clause(clause.extract()?)?.into();
            result = result.or(&clause);
        }
        Ok(result.into())
    }
}

// BooleanExpression ****************

#[pyclass]
#[derive(Clone)]
pub struct BooleanExpression(biodivine_lib_bdd::boolean_expression::BooleanExpression);

impl From<biodivine_lib_bdd::boolean_expression::BooleanExpression> for BooleanExpression {
    fn from(value: biodivine_lib_bdd::boolean_expression::BooleanExpression) -> Self {
        BooleanExpression(value)
    }
}

impl From<BooleanExpression> for biodivine_lib_bdd::boolean_expression::BooleanExpression {
    fn from(value: BooleanExpression) -> Self {
        value.0
    }
}

#[pymethods]
impl BooleanExpression {
    #[staticmethod]
    pub fn parse(value: &str) -> PyResult<BooleanExpression> {
        let parsed: Result<biodivine_lib_bdd::boolean_expression::BooleanExpression, String> =
            std::convert::TryFrom::try_from(value);
        match parsed {
            Ok(e) => Ok(e.into()),
            Err(message) => Err(PyTypeError::new_err(message)),
        }
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{}", self.0))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("BooleanExpression({})", self.0))
    }
}

// BddVariableSetBuilder *****************

#[pyclass]
pub struct BddVariableSetBuilder(biodivine_lib_bdd::BddVariableSetBuilder, Vec<String>);

#[pymethods]
impl BddVariableSetBuilder {
    /// Create a new, empty variable set builder.
    #[new]
    pub fn new() -> Self {
        BddVariableSetBuilder(biodivine_lib_bdd::BddVariableSetBuilder::new(), Vec::new())
    }

    /// Create a new variable with the given name. Returns a `BddVariable` instance that can be
    /// later used to create and query actual BDDs.
    pub fn make_variable(&mut self, name: &str) -> BddVariable {
        let var = self.0.make_variable(name).into();
        self.1.push(name.to_string());
        var
    }

    /// Create multiple variables with the names supplied as a list. Returns a list of
    /// `BddVariable` objects.
    pub fn make(&mut self, py: Python, names: &PyList) -> PyResult<PyObject> {
        let mut result: Vec<BddVariable> = Vec::new();
        for i in 0..names.len() {
            let name = names.get_item(i)?;
            let name: String = name.extract()?;
            let var = self.0.make_variable(name.as_str());
            self.1.push(name.to_string());
            result.push(var.into());
        }

        Ok(result.into_py(py))
    }

    /// Convert this builder to an actual variable set.
    pub fn build(&self) -> BddVariableSet {
        let mut builder = biodivine_lib_bdd::BddVariableSetBuilder::new();
        for name in &self.1 {
            builder.make_variable(name);
        }
        BddVariableSet(builder.build())
    }
}

// BddVariable *****************

#[pyclass]
#[derive(Clone)]
pub struct BddVariable(biodivine_lib_bdd::BddVariable);

impl From<biodivine_lib_bdd::BddVariable> for BddVariable {
    fn from(value: biodivine_lib_bdd::BddVariable) -> Self {
        BddVariable(value)
    }
}

impl From<BddVariable> for biodivine_lib_bdd::BddVariable {
    fn from(value: BddVariable) -> Self {
        value.0
    }
}

#[pymethods]
impl BddVariable {
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{}", self.0))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("BddVariable({})", self.0))
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn biodivine_bdd(_py: Python, module: &PyModule) -> PyResult<()> {
    module.add_class::<Bdd>()?;
    module.add_class::<BddVariable>()?;
    module.add_class::<BddVariableSet>()?;
    module.add_class::<BddVariableSetBuilder>()?;
    module.add_class::<BooleanExpression>()?;
    Ok(())
}
