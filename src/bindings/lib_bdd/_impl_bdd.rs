use super::PyBdd;
use crate::bindings::lib_bdd::{PyBddVariable, PyBddVariableSet, PyBooleanExpression};
use crate::{throw_runtime_error, AsNative};
use biodivine_lib_bdd::{Bdd, BddVariable, BddVariableSet};
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

impl From<Bdd> for PyBdd {
    fn from(value: Bdd) -> Self {
        PyBdd(value)
    }
}

impl From<PyBdd> for Bdd {
    fn from(value: PyBdd) -> Self {
        value.0
    }
}

impl AsNative<Bdd> for PyBdd {
    fn as_native(&self) -> &Bdd {
        &self.0
    }

    fn as_native_mut(&mut self) -> &mut Bdd {
        &mut self.0
    }
}

#[pymethods]
impl PyBdd {
    fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Lt => throw_runtime_error("Unsupported operation."),
            CompareOp::Le => throw_runtime_error("Unsupported operation."),
            CompareOp::Eq => Ok(self.0 == other.0),
            CompareOp::Ne => Ok(self.0 != other.0),
            CompareOp::Gt => throw_runtime_error("Unsupported operation."),
            CompareOp::Ge => throw_runtime_error("Unsupported operation."),
        }
    }

    fn __hash__(&self) -> isize {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish() as isize
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!(
            "Bdd(size={}, cardinality={})",
            self.node_count(),
            self.cardinality()
        ))
    }

    fn __repr__(&self) -> PyResult<String> {
        self.__str__()
    }

    /// Compute a logical negation of this `Bdd`.
    pub fn not(&self) -> PyBdd {
        self.as_native().not().into()
    }

    /// Compute a logical conjunction of two formulas.
    pub fn and(&self, other: &PyBdd) -> PyBdd {
        self.as_native().and(other.as_native()).into()
    }

    /// Compute a logical disjunction of two formulas.
    pub fn or(&self, other: &PyBdd) -> PyBdd {
        self.as_native().or(other.as_native()).into()
    }

    /// Compute a logical implication of two formulas.
    pub fn imp(&self, other: &PyBdd) -> PyBdd {
        self.as_native().imp(other.as_native()).into()
    }

    /// Compute a logical equivalence of two formulas.
    pub fn iff(&self, other: &PyBdd) -> PyBdd {
        self.as_native().iff(other.as_native()).into()
    }

    /// Compute a logical xor of two formulas.
    pub fn xor(&self, other: &PyBdd) -> PyBdd {
        self.as_native().xor(other.as_native()).into()
    }

    /// Compute a logical conjunction of this formula with a negated second formula.
    pub fn and_not(&self, other: &PyBdd) -> PyBdd {
        self.as_native().and_not(other.as_native()).into()
    }

    /// Compute a projection over the given `Bdd` variable.
    pub fn var_project(&self, variable: PyBddVariable) -> PyBdd {
        self.as_native().var_project(variable.into()).into()
    }

    /// Compute a projection over all the given `Bdd` variables.
    pub fn project(&self, variables: &PyList) -> PyResult<PyBdd> {
        let mut vars: Vec<BddVariable> = Vec::with_capacity(variables.len());
        for var in variables {
            vars.push(var.extract::<PyBddVariable>()?.into());
        }
        Ok(self.as_native().project(&vars).into())
    }

    /// Compute a pick operation for the given `Bdd` variable (biased towards 0).
    ///
    /// See the original Rust library docs for details about semantics.
    pub fn var_pick(&self, variable: PyBddVariable) -> PyBdd {
        self.as_native().var_pick(variable.into()).into()
    }

    /// Compute a pick operation for all the given `Bdd` variables (biased towards 0).
    ///
    /// See the original Rust library docs for details about semantics.
    pub fn pick(&self, variables: &PyList) -> PyResult<PyBdd> {
        let mut vars: Vec<BddVariable> = Vec::with_capacity(variables.len());
        for var in variables {
            vars.push(var.extract::<PyBddVariable>()?.into());
        }
        Ok(self.as_native().pick(&vars).into())
    }

    /// Compute a selection for the given `Bdd` variable with the given value.
    pub fn var_select(&self, var: PyBddVariable, value: bool) -> PyBdd {
        self.as_native().var_select(var.into(), value).into()
    }

    /// Compute a selection of the given partial valuation.
    ///
    /// The partial valuation is a dictionary ` { BddVariable: bool }` which specifies variable
    /// values that should be fixed.
    pub fn select(&self, values: &PyDict) -> PyResult<PyBdd> {
        let mut valuation: Vec<(BddVariable, bool)> = Vec::new();
        for (k, v) in values {
            let key = k.extract::<PyBddVariable>()?;
            let value = v.extract::<bool>()?;
            valuation.push((key.into(), value));
        }
        Ok(self.as_native().select(&valuation).into())
    }

    /// Print this `Bdd` to a `.dot` file that can be visualised using e.g. `graphviz`.
    ///
    /// Variable names are resolved from the given `BddVariableSet`. If not given, the names
    /// default to `x_0`, `x_1`, etc.
    #[args(variables = "None")]
    pub fn to_dot(&self, variables: Option<&PyBddVariableSet>) -> String {
        if let Some(variables) = variables {
            self.as_native().to_dot_string(variables.as_native(), true)
        } else {
            let variables = BddVariableSet::new_anonymous(self.as_native().num_vars());
            self.as_native().to_dot_string(&variables, true)
        }
    }

    /// Produces a raw string representation of this `Bdd` that can be saved into a file or sent
    /// over the network.
    pub fn to_raw_string(&self) -> String {
        self.as_native().to_string()
    }

    /// Read a `Bdd` from a raw string representation.
    ///
    /// **WARNING**: This operation performs very basic integrity checks of the `Bdd`, but it is
    /// absolutely possible to create a potentially incompatible `Bdd` this way.
    #[staticmethod]
    pub fn from_raw_string(data: &str) -> PyBdd {
        // This will panic on error, but the necessary function to extract the error
        // is private in the Bdd struct (for now).
        Bdd::from_string(data).into()
    }

    /// Check if this formula represents a single conjunctive clause
    /// (i.e. the `Bdd` is a single path).
    ///
    /// This is similar to `is_valuation`, but in `is_valuation`, we require that all decision
    /// variables appear on this path.
    pub fn is_conjunctive_clause(&self) -> bool {
        self.as_native().is_clause()
    }

    /// Check that this `Bdd` represents a single valuation: i.e. there is exactly
    /// one value for each variable that satisfies this `Bdd`.
    pub fn is_valuation(&self) -> bool {
        self.as_native().is_valuation()
    }

    /// Return the number of nodes in this `Bdd`, i.e. the size of the symbolic representation.
    pub fn node_count(&self) -> usize {
        self.as_native().size()
    }

    /// Return the number of variables supported by this `Bdd` (not all have to be used).
    pub fn var_count(&self) -> usize {
        usize::from(self.as_native().num_vars())
    }

    /// `True` if this `Bdd` represents a tautology.
    pub fn is_true(&self) -> bool {
        self.as_native().is_true()
    }

    /// `True` if this `Bdd` represents a contradiction.
    pub fn is_false(&self) -> bool {
        self.as_native().is_false()
    }

    /// Return an count of satisfying valuations in this `Bdd` (the number may be approximate
    /// when the `Bdd` is too large).
    pub fn cardinality(&self) -> f64 {
        self.as_native().cardinality()
    }

    /// Return a list of Booleans representing one satisfying valuation of this `Bdd`.
    ///
    /// If the `Bdd` is not satisfiable, returns `None`.
    pub fn sat_witness(&self, py: Python) -> Option<PyObject> {
        self.as_native()
            .sat_witness()
            .map(|witness| witness.vector().into_py(py))
    }

    /// Convert this `Bdd` into a `BooleanExpression`.
    ///
    /// Note that this is not doing any fancy minimisation of the formula, so the result can
    /// be very large! The main purpose of this function is to enable conversion of networks
    /// (e.g. witnesses) back to strings.
    ///
    /// The first argument is a `BddVariableSet` that supplies the variable names. If it is not
    /// given, then default names (`x_0`, `x_1`, ...) are used.
    #[args(variables = "None")]
    pub fn to_boolean_expression(
        &self,
        variables: Option<&PyBddVariableSet>,
    ) -> PyBooleanExpression {
        if let Some(variables) = variables {
            self.as_native()
                .to_boolean_expression(variables.as_native())
                .into()
        } else {
            let variables = BddVariableSet::new_anonymous(self.as_native().num_vars());
            self.as_native().to_boolean_expression(&variables).into()
        }
    }
}
