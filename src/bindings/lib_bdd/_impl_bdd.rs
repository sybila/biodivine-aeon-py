use super::PyBdd;
use crate::bindings::lib_bdd::{
    PyBddClauseIterator, PyBddPartialValuation, PyBddValuation, PyBddValuationIterator,
    PyBddVariable, PyBddVariableSet, PyBooleanExpression,
};
use crate::{throw_runtime_error, throw_type_error, AsNative};
use biodivine_lib_bdd::{Bdd, BddVariable, BddVariableSet};
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use rand::{Rng, SeedableRng};
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

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

    pub fn l_not(&self) -> PyBdd {
        self.as_native().not().into()
    }

    #[pyo3(signature = (other, limit = None))]
    pub fn l_and(&self, other: &PyBdd, limit: Option<usize>) -> PyResult<PyBdd> {
        if let Some(limit) = limit {
            let (left, right) = (self.as_native(), other.as_native());
            let result =
                Bdd::binary_op_with_limit(limit, left, right, biodivine_lib_bdd::op_function::and);
            if let Some(result) = result {
                Ok(result.into())
            } else {
                throw_runtime_error("BDD size limit exceeded.")
            }
        } else {
            Ok(self.as_native().and(other.as_native()).into())
        }
    }

    #[pyo3(signature = (other, limit = None))]
    pub fn l_or(&self, other: &PyBdd, limit: Option<usize>) -> PyResult<PyBdd> {
        if let Some(limit) = limit {
            let (left, right) = (self.as_native(), other.as_native());
            let result =
                Bdd::binary_op_with_limit(limit, left, right, biodivine_lib_bdd::op_function::or);
            if let Some(result) = result {
                Ok(result.into())
            } else {
                throw_runtime_error("BDD size limit exceeded.")
            }
        } else {
            Ok(self.as_native().or(other.as_native()).into())
        }
    }

    #[pyo3(signature = (other, limit = None))]
    pub fn l_imp(&self, other: &PyBdd, limit: Option<usize>) -> PyResult<PyBdd> {
        if let Some(limit) = limit {
            let (left, right) = (self.as_native(), other.as_native());
            let result =
                Bdd::binary_op_with_limit(limit, left, right, biodivine_lib_bdd::op_function::imp);
            if let Some(result) = result {
                Ok(result.into())
            } else {
                throw_runtime_error("BDD size limit exceeded.")
            }
        } else {
            Ok(self.as_native().imp(other.as_native()).into())
        }
    }

    #[pyo3(signature = (other, limit = None))]
    pub fn l_iff(&self, other: &PyBdd, limit: Option<usize>) -> PyResult<PyBdd> {
        if let Some(limit) = limit {
            let (left, right) = (self.as_native(), other.as_native());
            let result =
                Bdd::binary_op_with_limit(limit, left, right, biodivine_lib_bdd::op_function::iff);
            if let Some(result) = result {
                Ok(result.into())
            } else {
                throw_runtime_error("BDD size limit exceeded.")
            }
        } else {
            Ok(self.as_native().iff(other.as_native()).into())
        }
    }

    #[pyo3(signature = (other, limit = None))]
    pub fn l_xor(&self, other: &PyBdd, limit: Option<usize>) -> PyResult<PyBdd> {
        if let Some(limit) = limit {
            let (left, right) = (self.as_native(), other.as_native());
            let result =
                Bdd::binary_op_with_limit(limit, left, right, biodivine_lib_bdd::op_function::xor);
            if let Some(result) = result {
                Ok(result.into())
            } else {
                throw_runtime_error("BDD size limit exceeded.")
            }
        } else {
            Ok(self.as_native().xor(other.as_native()).into())
        }
    }

    #[pyo3(signature = (other, limit = None))]
    pub fn l_and_not(&self, other: &PyBdd, limit: Option<usize>) -> PyResult<PyBdd> {
        if let Some(limit) = limit {
            let (left, right) = (self.as_native(), other.as_native());
            let result = Bdd::binary_op_with_limit(
                limit,
                left,
                right,
                biodivine_lib_bdd::op_function::and_not,
            );
            if let Some(result) = result {
                Ok(result.into())
            } else {
                throw_runtime_error("BDD size limit exceeded.")
            }
        } else {
            Ok(self.as_native().and_not(other.as_native()).into())
        }
    }

    pub fn sem_eq(&self, other: &PyBdd) -> bool {
        // First, try structural equality, if it does not work, use semantic equality.
        self == other || self.as_native().iff(other.as_native()).is_true()
    }

    pub fn project_exist(&self, variables: &PyAny) -> PyResult<PyBdd> {
        let variables = extract_variable_list(variables)?;
        Ok(self.as_native().exists(&variables).into())
    }

    pub fn project_for_all(&self, variables: &PyAny) -> PyResult<PyBdd> {
        let variables = extract_variable_list(variables)?;
        Ok(self.as_native().for_all(&variables).into())
    }

    pub fn pick(&self, variables: &PyAny) -> PyResult<PyBdd> {
        let variables = extract_variable_list(variables)?;
        Ok(self.as_native().pick(&variables).into())
    }

    #[pyo3(signature = (variables, seed = None))]
    pub fn pick_random(&self, variables: &PyAny, seed: Option<u64>) -> PyResult<PyBdd> {
        /// Generic helper function to handle both cases.
        fn pick_random_rng<R: Rng>(
            bdd: &PyBdd,
            variables: &[BddVariable],
            rng: &mut R,
        ) -> PyResult<PyBdd> {
            Ok(bdd.as_native().pick_random(variables, rng).into())
        }

        let variables = extract_variable_list(variables)?;
        if let Some(seed) = seed {
            let mut rng = rand::prelude::StdRng::seed_from_u64(seed);
            pick_random_rng(self, &variables, &mut rng)
        } else {
            pick_random_rng(self, &variables, &mut rand::thread_rng())
        }
    }

    pub fn select(&self, values: &PyAny) -> PyResult<PyBdd> {
        let valuation = PyBddPartialValuation::from_python(values)?;
        Ok(self
            .as_native()
            .select(&valuation.as_native().to_values())
            .into())
    }

    pub fn restrict(&self, values: &PyDict) -> PyResult<PyBdd> {
        let valuation = PyBddPartialValuation::from_python(values)?;
        Ok(self
            .as_native()
            .restrict(&valuation.as_native().to_values())
            .into())
    }

    pub fn valuation_iterator(&self) -> PyBddValuationIterator {
        // This hack allows us to "launder" lifetimes between Rust and Python.
        // It is only safe because we copy the `Bdd` and attach it to the "laundered" reference,
        // so there is no (realistic) way the reference can outlive the copy of the `Bdd`.
        // Fortunately, the iterator items are clones and do not reference the `Bdd` directly,
        // so the "laundered" references do not spread beyond the internal code of the iterator.
        let copy = self.as_native().clone();
        let copy_ref: &'static Bdd = unsafe { (&copy as *const Bdd).as_ref().unwrap() };
        PyBddValuationIterator(copy_ref.sat_valuations(), copy)
    }

    pub fn valuation_witness(&self) -> Option<PyBddValuation> {
        self.as_native().sat_witness().map(|it| it.into())
    }

    #[pyo3(signature = (seed = None))]
    pub fn valuation_random(&self, seed: Option<u64>) -> Option<PyBddValuation> {
        fn inner<R: Rng>(bdd: &Bdd, rng: &mut R) -> Option<PyBddValuation> {
            bdd.random_valuation(rng).map(|it| it.into())
        }

        if let Some(seed) = seed {
            let mut rng = rand::prelude::StdRng::seed_from_u64(seed);
            inner(self.as_native(), &mut rng)
        } else {
            inner(self.as_native(), &mut rand::thread_rng())
        }
    }

    pub fn clause_iterator(&self) -> PyBddClauseIterator {
        // See `iter_valuations` for safety discussion.
        let copy = self.as_native().clone();
        let copy_ref: &'static Bdd = unsafe { (&copy as *const Bdd).as_ref().unwrap() };
        PyBddClauseIterator(copy_ref.sat_clauses(), copy)
    }

    pub fn clause_witness(&self) -> Option<PyBddPartialValuation> {
        self.as_native().first_clause().map(|it| it.into())
    }

    #[pyo3(signature = (seed = None))]
    pub fn clause_random(&self, seed: Option<u64>) -> Option<PyBddPartialValuation> {
        fn inner<R: Rng>(bdd: &Bdd, rng: &mut R) -> Option<PyBddPartialValuation> {
            bdd.random_clause(rng).map(|it| it.into())
        }
        if let Some(seed) = seed {
            let mut rng = rand::prelude::StdRng::seed_from_u64(seed);
            inner(self.as_native(), &mut rng)
        } else {
            inner(self.as_native(), &mut rand::thread_rng())
        }
    }

    pub fn clause_necessary(&self) -> Option<PyBddPartialValuation> {
        self.as_native().necessary_clause().map(|it| it.into())
    }

    #[pyo3(signature = (variables = None, zero_pruned = true))]
    pub fn to_dot(&self, variables: Option<&PyBddVariableSet>, zero_pruned: bool) -> String {
        if let Some(variables) = variables {
            self.as_native()
                .to_dot_string(variables.as_native(), zero_pruned)
        } else {
            let variables = BddVariableSet::new_anonymous(self.as_native().num_vars());
            self.as_native().to_dot_string(&variables, zero_pruned)
        }
    }

    pub fn to_raw_string(&self) -> String {
        self.as_native().to_string()
    }

    #[staticmethod]
    pub fn from_raw_string(data: &str) -> PyResult<PyBdd> {
        match Bdd::read_as_string(&mut data.as_bytes()) {
            Ok(bdd) => Ok(bdd.into()),
            Err(error) => throw_runtime_error(format!("Invalid BDD string: {error}")),
        }
    }

    pub fn is_conjunctive_clause(&self) -> bool {
        self.as_native().is_clause()
    }

    pub fn is_valuation(&self) -> bool {
        self.as_native().is_valuation()
    }

    pub fn node_count(&self) -> usize {
        self.as_native().size()
    }

    pub fn var_count(&self) -> usize {
        usize::from(self.as_native().num_vars())
    }

    pub fn is_true(&self) -> bool {
        self.as_native().is_true()
    }

    pub fn is_false(&self) -> bool {
        self.as_native().is_false()
    }

    pub fn cardinality(&self) -> f64 {
        self.as_native().cardinality()
    }

    #[pyo3(signature = (variables = None))]
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

    pub fn support_set(&self) -> HashSet<PyBddVariable> {
        self.as_native()
            .support_set()
            .into_iter()
            .map(PyBddVariable::from)
            .collect()
    }

    pub fn size_per_variable(&self) -> HashMap<PyBddVariable, usize> {
        self.as_native()
            .size_per_variable()
            .into_iter()
            .map(|(k, v)| (PyBddVariable::from(k), v))
            .collect()
    }
}

/// A helper function to extract a list of variables from an argument that can be
/// either a list or a single variable.
fn extract_variable_list(any: &PyAny) -> PyResult<Vec<BddVariable>> {
    if let Ok(list) = any.downcast::<PyList>() {
        let mut vars: Vec<BddVariable> = Vec::with_capacity(list.len());
        for var in list {
            vars.push(var.extract::<PyBddVariable>()?.into());
        }
        Ok(vars)
    } else if let Ok(var) = any.extract::<PyBddVariable>() {
        Ok(vec![var.into()])
    } else {
        throw_type_error(
            "Expected either a single `BddVariable`, or a list of `BddVariable` objects.",
        )
    }
}
