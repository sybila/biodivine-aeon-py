use super::PyBdd;
use crate::bindings::lib_bdd::{
    PyBddClauseIterator, PyBddPartialValuation, PyBddValuation, PyBddValuationIterator,
    PyBddVariable, PyBddVariableSet, PyBooleanExpression,
};
use crate::{throw_runtime_error, throw_type_error, AsNative};
use biodivine_lib_bdd::{Bdd, BddVariable, BddVariableSet};
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyList};
use rand::{Rng, SeedableRng};
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::io::Cursor;
use num_bigint::BigInt;
use num_traits::cast::FromPrimitive;

struct OpFunction2(Vec<Option<bool>>);
struct OpFunction3(Vec<Option<bool>>);

impl OpFunction2 {
    pub fn new(py: Python, py_function: Py<PyAny>) -> PyResult<OpFunction2> {
        let none: Option<bool> = None;
        let tt = Some(true);
        let ff = Some(false);
        let none_none: Option<bool> = py_function.call1(py, (none, none))?.extract(py)?;
        let none_false: Option<bool> = py_function.call1(py, (none, ff))?.extract(py)?;
        let false_none: Option<bool> = py_function.call1(py, (ff, none))?.extract(py)?;
        let false_false: Option<bool> = py_function.call1(py, (ff, ff))?.extract(py)?;
        let none_true: Option<bool> = py_function.call1(py, (none, tt))?.extract(py)?;
        let true_none: Option<bool> = py_function.call1(py, (tt, none))?.extract(py)?;
        let true_true: Option<bool> = py_function.call1(py, (tt, tt))?.extract(py)?;
        let true_false: Option<bool> = py_function.call1(py, (tt, ff))?.extract(py)?;
        let false_true: Option<bool> = py_function.call1(py, (ff, tt))?.extract(py)?;
        Ok(OpFunction2(vec![
            none_none,
            none_false,
            false_none,
            false_false,
            none_true,
            true_none,
            true_true,
            true_false,
            false_true,
        ]))
    }

    pub fn invoke(&self, left: Option<bool>, right: Option<bool>) -> Option<bool> {
        let i = match (left, right) {
            (None, None) => 0,
            (None, Some(false)) => 1,
            (Some(false), None) => 2,
            (Some(false), Some(false)) => 3,
            (None, Some(true)) => 4,
            (Some(true), None) => 5,
            (Some(true), Some(true)) => 6,
            (Some(true), Some(false)) => 7,
            (Some(false), Some(true)) => 8,
        };
        unsafe { *self.0.get_unchecked(i) }
    }
}

impl OpFunction3 {
    pub fn new(py: Python, py_function: Py<PyAny>) -> PyResult<OpFunction3> {
        let none: Option<bool> = None;
        let tt = Some(true);
        let ff = Some(false);
        let none_none_none: Option<bool> =
            py_function.call1(py, (none, none, none))?.extract(py)?;
        let none_none_false: Option<bool> = py_function.call1(py, (none, none, ff))?.extract(py)?;
        let none_false_none: Option<bool> = py_function.call1(py, (none, ff, none))?.extract(py)?;
        let false_none_none: Option<bool> = py_function.call1(py, (ff, none, none))?.extract(py)?;
        let none_false_false: Option<bool> = py_function.call1(py, (none, ff, ff))?.extract(py)?;
        let false_none_false: Option<bool> = py_function.call1(py, (ff, none, ff))?.extract(py)?;
        let false_false_none: Option<bool> = py_function.call1(py, (ff, ff, none))?.extract(py)?;
        let false_false_false: Option<bool> = py_function.call1(py, (ff, ff, ff))?.extract(py)?;
        let none_none_true: Option<bool> = py_function.call1(py, (none, none, tt))?.extract(py)?;
        let none_true_none: Option<bool> = py_function.call1(py, (none, tt, none))?.extract(py)?;
        let true_none_none: Option<bool> = py_function.call1(py, (tt, none, none))?.extract(py)?;
        let none_true_true: Option<bool> = py_function.call1(py, (none, tt, tt))?.extract(py)?;
        let true_none_true: Option<bool> = py_function.call1(py, (tt, none, tt))?.extract(py)?;
        let true_true_none: Option<bool> = py_function.call1(py, (tt, tt, none))?.extract(py)?;
        let true_true_true: Option<bool> = py_function.call1(py, (tt, tt, tt))?.extract(py)?;
        let none_false_true: Option<bool> = py_function.call1(py, (none, ff, tt))?.extract(py)?;
        let none_true_false: Option<bool> = py_function.call1(py, (none, tt, ff))?.extract(py)?;
        let false_none_true: Option<bool> = py_function.call1(py, (ff, none, tt))?.extract(py)?;
        let true_none_false: Option<bool> = py_function.call1(py, (tt, none, ff))?.extract(py)?;
        let false_true_none: Option<bool> = py_function.call1(py, (ff, tt, none))?.extract(py)?;
        let true_false_none: Option<bool> = py_function.call1(py, (tt, ff, none))?.extract(py)?;
        let false_false_true: Option<bool> = py_function.call1(py, (ff, ff, tt))?.extract(py)?;
        let false_true_false: Option<bool> = py_function.call1(py, (ff, tt, ff))?.extract(py)?;
        let true_false_false: Option<bool> = py_function.call1(py, (tt, ff, ff))?.extract(py)?;
        let false_true_true: Option<bool> = py_function.call1(py, (ff, tt, tt))?.extract(py)?;
        let true_false_true: Option<bool> = py_function.call1(py, (tt, ff, tt))?.extract(py)?;
        let true_true_false: Option<bool> = py_function.call1(py, (tt, tt, ff))?.extract(py)?;
        Ok(OpFunction3(vec![
            none_none_none,
            none_none_false,
            none_false_none,
            false_none_none,
            none_false_false,
            false_none_false,
            false_false_none,
            false_false_false,
            none_none_true,
            none_true_none,
            true_none_none,
            none_true_true,
            true_none_true,
            true_true_none,
            true_true_true,
            none_false_true,
            none_true_false,
            false_none_true,
            true_none_false,
            false_true_none,
            true_false_none,
            false_false_true,
            false_true_false,
            true_false_false,
            false_true_true,
            true_false_true,
            true_true_false,
        ]))
    }

    pub fn invoke(&self, a: Option<bool>, b: Option<bool>, c: Option<bool>) -> Option<bool> {
        let i = match (a, b, c) {
            (None, None, None) => 0,
            (None, None, Some(false)) => 1,
            (None, Some(false), None) => 2,
            (Some(false), None, None) => 3,
            (None, Some(false), Some(false)) => 4,
            (Some(false), None, Some(false)) => 5,
            (Some(false), Some(false), None) => 6,
            (Some(false), Some(false), Some(false)) => 7,
            (None, None, Some(true)) => 8,
            (None, Some(true), None) => 9,
            (Some(true), None, None) => 10,
            (None, Some(true), Some(true)) => 11,
            (Some(true), None, Some(true)) => 12,
            (Some(true), Some(true), None) => 13,
            (Some(true), Some(true), Some(true)) => 14,
            (None, Some(false), Some(true)) => 15,
            (None, Some(true), Some(false)) => 16,
            (Some(false), None, Some(true)) => 17,
            (Some(true), None, Some(false)) => 18,
            (Some(false), Some(true), None) => 19,
            (Some(true), Some(false), None) => 20,
            (Some(false), Some(false), Some(true)) => 21,
            (Some(false), Some(true), Some(false)) => 22,
            (Some(true), Some(false), Some(false)) => 23,
            (Some(false), Some(true), Some(true)) => 24,
            (Some(true), Some(false), Some(true)) => 25,
            (Some(true), Some(true), Some(false)) => 26,
        };
        unsafe { *self.0.get_unchecked(i) }
    }
}

#[pymethods]
impl PyBdd {
    pub fn __str__(&self) -> String {
        format!(
            "Bdd(var_count={}, node_count={})",
            self.as_native().num_vars(),
            self.as_native().size()
        )
    }

    pub fn __repr__(&self) -> String {
        format!("<{}>", self.__str__())
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> bool {
        match op {
            CompareOp::Lt => self.__lt__(other),
            CompareOp::Le => self.__le__(other),
            CompareOp::Eq => self.__eq__(other),
            CompareOp::Ne => !self.__eq__(other),
            CompareOp::Gt => other.__lt__(self),
            CompareOp::Ge => other.__le__(self),
        }
    }

    pub fn __eq__(&self, other: &PyBdd) -> bool {
        // Semantic equality check needs to verify that A <=> B is a tautology,
        // which is the same as checking if A XOR B is a contradiction.
        // We actually don't have to compute the whole result though, we just need to know
        // if it is `false`. Hence we can stop once the result size exceeds one node.
        // Other comparison operators are implemented using similar logic.
        Bdd::binary_op_with_limit(
            1,
            self.as_native(),
            other.as_native(),
            biodivine_lib_bdd::op_function::xor,
        )
        .is_some()
    }

    pub fn __le__(&self, other: &PyBdd) -> bool {
        // (A <= B) if and only if A implies B, meaning (!A or B) is a tautology.
        // Hence (A & !B) must be a contradiction.
        Bdd::binary_op_with_limit(
            1,
            self.as_native(),
            other.as_native(),
            biodivine_lib_bdd::op_function::and_not,
        )
        .is_some()
    }

    pub fn __lt__(&self, other: &PyBdd) -> bool {
        // This is the same as __le__, but it must also hold that A != B.
        // AFAIK this cannot be a single operation, because we'd actually need to test
        // if the partial result is a contradiction, which we cannot do *inside* the
        // apply algorithm.
        self.__le__(other) && !self.__eq__(other)
    }

    pub fn graph_eq(&self, other: &PyBdd) -> bool {
        self.as_native() == other.as_native()
    }

    fn __hash__(&self) -> isize {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish() as isize
    }

    pub fn l_not(&self) -> PyBdd {
        self.as_native().not().into()
    }

    #[pyo3(signature = (other, limit = None))]
    pub fn l_and(&self, other: &PyBdd, limit: Option<usize>) -> PyResult<PyBdd> {
        if let Some(limit) = limit {
            let result = Bdd::binary_op_with_limit(
                limit,
                self.as_native(),
                other.as_native(),
                biodivine_lib_bdd::op_function::and,
            );
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
            let result = Bdd::binary_op_with_limit(
                limit,
                self.as_native(),
                other.as_native(),
                biodivine_lib_bdd::op_function::or,
            );
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
            let result = Bdd::binary_op_with_limit(
                limit,
                self.as_native(),
                other.as_native(),
                biodivine_lib_bdd::op_function::imp,
            );
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
            let result = Bdd::binary_op_with_limit(
                limit,
                self.as_native(),
                other.as_native(),
                biodivine_lib_bdd::op_function::iff,
            );
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
            let result = Bdd::binary_op_with_limit(
                limit,
                self.as_native(),
                other.as_native(),
                biodivine_lib_bdd::op_function::xor,
            );
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
            let result = Bdd::binary_op_with_limit(
                limit,
                self.as_native(),
                other.as_native(),
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

    #[pyo3(signature = (left, right, function, flip_left = None, flip_right = None, flip_output = None, limit = None))]
    #[staticmethod]
    #[allow(clippy::too_many_arguments)] // There isn't much to do about this
    pub fn apply2(
        py: Python,
        left: &PyBdd,
        right: &PyBdd,
        function: Py<PyAny>,
        flip_left: Option<PyBddVariable>,
        flip_right: Option<PyBddVariable>,
        flip_output: Option<PyBddVariable>,
        limit: Option<usize>,
    ) -> PyResult<PyBdd> {
        let flip_left: Option<BddVariable> = flip_left.map(|it| it.into());
        let flip_right: Option<BddVariable> = flip_right.map(|it| it.into());
        let flip_output: Option<BddVariable> = flip_output.map(|it| it.into());
        let function = OpFunction2::new(py, function)?;
        if let Some(limit) = limit {
            let result = Bdd::fused_binary_flip_op_with_limit(
                limit,
                (left.as_native(), flip_left),
                (right.as_native(), flip_right),
                flip_output,
                |l, r| function.invoke(l, r),
            );
            if let Some(result) = result {
                Ok(result.into())
            } else {
                throw_runtime_error("BDD size limit exceeded.")
            }
        } else {
            Ok(Bdd::fused_binary_flip_op(
                (left.as_native(), flip_left),
                (right.as_native(), flip_right),
                flip_output,
                |l, r| function.invoke(l, r),
            )
            .into())
        }
    }

    #[staticmethod]
    #[pyo3(signature = (a, b, c, function, flip_a = None, flip_b = None, flip_c = None, flip_out = None))]
    #[allow(clippy::too_many_arguments)] // There isn't much to do about this
    pub fn apply3(
        py: Python,
        a: &PyBdd,
        b: &PyBdd,
        c: &PyBdd,
        function: Py<PyAny>,
        flip_a: Option<PyBddVariable>,
        flip_b: Option<PyBddVariable>,
        flip_c: Option<PyBddVariable>,
        flip_out: Option<PyBddVariable>,
    ) -> PyResult<PyBdd> {
        let flip_a: Option<BddVariable> = flip_a.map(|it| it.into());
        let flip_b: Option<BddVariable> = flip_b.map(|it| it.into());
        let flip_c: Option<BddVariable> = flip_c.map(|it| it.into());
        let flip_out: Option<BddVariable> = flip_out.map(|it| it.into());
        let function = OpFunction3::new(py, function)?;
        Ok(Bdd::fused_ternary_flip_op(
            (a.as_native(), flip_a),
            (b.as_native(), flip_b),
            (c.as_native(), flip_c),
            flip_out,
            |x, y, z| function.invoke(x, y, z),
        )
        .into())
    }

    #[pyo3(signature = (left, right, function, flip_left = None, flip_right = None, flip_output = None))]
    #[staticmethod]
    pub fn check2(
        py: Python,
        left: &PyBdd,
        right: &PyBdd,
        function: Py<PyAny>,
        flip_left: Option<PyBddVariable>,
        flip_right: Option<PyBddVariable>,
        flip_output: Option<PyBddVariable>,
    ) -> PyResult<(bool, usize)> {
        let flip_left: Option<BddVariable> = flip_left.map(|it| it.into());
        let flip_right: Option<BddVariable> = flip_right.map(|it| it.into());
        let flip_output: Option<BddVariable> = flip_output.map(|it| it.into());
        let function = OpFunction2::new(py, function)?;
        Ok(Bdd::check_fused_binary_flip_op(
            usize::MAX,
            (left.as_native(), flip_left),
            (right.as_native(), flip_right),
            flip_output,
            |l, r| function.invoke(l, r),
        )
        .unwrap())
    }

    pub fn r_pick(&self, variables: &PyAny) -> PyResult<PyBdd> {
        let variables = extract_variable_list(variables)?;
        Ok(self.as_native().pick(&variables).into())
    }

    #[pyo3(signature = (variables, seed = None))]
    pub fn r_pick_random(&self, variables: &PyAny, seed: Option<u64>) -> PyResult<PyBdd> {
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

    pub fn r_project_exist(&self, variables: &PyAny) -> PyResult<PyBdd> {
        let variables = extract_variable_list(variables)?;
        Ok(self.as_native().project(&variables).into())
    }

    pub fn r_project_for_all(&self, variables: &PyAny) -> PyResult<PyBdd> {
        let variables = extract_variable_list(variables)?;
        let mut result = self.as_native().clone();
        for var in variables {
            result = Bdd::fused_binary_flip_op(
                (&result, None),
                (&result, Some(var)),
                None,
                biodivine_lib_bdd::op_function::and,
            );
        }
        Ok(result.into())
    }

    pub fn r_restrict(&self, py: Python, values: &PyAny) -> PyResult<PyBdd> {
        let valuation = PyBddPartialValuation::from_python_type(values)?;
        let valuation = valuation.borrow(py);
        Ok(self
            .as_native()
            .restrict(&valuation.as_native().to_values())
            .into())
    }

    pub fn r_select(&self, py: Python, values: &PyAny) -> PyResult<PyBdd> {
        let valuation = PyBddPartialValuation::from_python_type(values)?;
        let valuation = valuation.borrow(py);
        Ok(self
            .as_native()
            .select(&valuation.as_native().to_values())
            .into())
    }

    pub fn var_count(&self) -> usize {
        usize::from(self.as_native().num_vars())
    }

    pub fn support_set(&self) -> HashSet<PyBddVariable> {
        self.as_native()
            .support_set()
            .into_iter()
            .map(|it| it.into())
            .collect()
    }

    pub fn is_true(&self) -> bool {
        self.as_native().is_true()
    }

    pub fn is_false(&self) -> bool {
        self.as_native().is_false()
    }

    pub fn is_clause(&self) -> bool {
        self.as_native().is_clause()
    }

    pub fn is_valuation(&self) -> bool {
        self.as_native().is_valuation()
    }

    #[pyo3(signature = (exact = false))]
    pub fn cardinality(&self, exact: bool) -> BigInt {
        if exact {
            self.as_native().exact_cardinality()
        } else {
            let result = self.as_native().cardinality();
            if let Some(value) = BigInt::from_f64(result) {
                value
            } else {
                self.as_native().exact_cardinality()
            }
        }
    }

    pub fn node_count(&self) -> usize {
        self.as_native().size()
    }

    pub fn node_count_per_variable(&self) -> HashMap<PyBddVariable, usize> {
        self.as_native()
            .size_per_variable()
            .into_iter()
            .map(|(k, v)| (PyBddVariable::from(k), v))
            .collect()
    }

    pub fn witness(&self) -> Option<PyBddValuation> {
        self.as_native().sat_witness().map(|it| it.into())
    }

    pub fn valuation_first(&self) -> Option<PyBddValuation> {
        self.as_native().first_valuation().map(|it| it.into())
    }

    pub fn valuation_last(&self) -> Option<PyBddValuation> {
        self.as_native().last_valuation().map(|it| it.into())
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

    pub fn valuation_most_positive(&self) -> Option<PyBddValuation> {
        self.as_native()
            .most_positive_valuation()
            .map(|it| it.into())
    }

    pub fn valuation_most_negative(&self) -> Option<PyBddValuation> {
        self.as_native()
            .most_negative_valuation()
            .map(|it| it.into())
    }

    pub fn valuation_iterator(self_: Py<PyBdd>, py: Python) -> PyBddValuationIterator {
        PyBddValuationIterator::new(py, self_)
    }

    pub fn clause_first(&self) -> Option<PyBddPartialValuation> {
        self.as_native().first_clause().map(|it| it.into())
    }

    pub fn clause_last(&self) -> Option<PyBddPartialValuation> {
        self.as_native().last_clause().map(|it| it.into())
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

    pub fn clause_iterator(self_: Py<PyBdd>, py: Python) -> PyBddClauseIterator {
        PyBddClauseIterator::new(py, self_)
    }

    #[pyo3(signature = (variables = None))]
    pub fn to_expression(&self, variables: Option<&PyBddVariableSet>) -> PyBooleanExpression {
        if let Some(variables) = variables {
            self.as_native()
                .to_boolean_expression(variables.as_native())
                .into()
        } else {
            let variables = BddVariableSet::new_anonymous(self.as_native().num_vars());
            self.as_native().to_boolean_expression(&variables).into()
        }
    }

    #[staticmethod]
    pub fn from_valuation(valuation: PyBddValuation) -> PyBdd {
        Bdd::from(valuation.0).into()
    }

    #[staticmethod]
    pub fn from_expression(variables: &PyBddVariableSet, expression: &PyAny) -> PyResult<PyBdd> {
        variables.eval_expression(expression)
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

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();
        // This should be ok to unwrap because writing into a vector can't fail.
        self.as_native().write_as_bytes(&mut result).unwrap();
        result
    }

    #[staticmethod]
    pub fn from_bytes(bytes: &PyBytes) -> PyBdd {
        let mut cursor = Cursor::new(bytes.as_bytes());
        Bdd::read_as_bytes(&mut cursor).unwrap().into()
    }

    #[pyo3(signature = (variables = None, zero_pruned = true))]
    pub fn to_dot_string(&self, variables: Option<&PyBddVariableSet>, zero_pruned: bool) -> String {
        if let Some(variables) = variables {
            self.as_native()
                .to_dot_string(variables.as_native(), zero_pruned)
        } else {
            let variables = BddVariableSet::new_anonymous(self.as_native().num_vars());
            self.as_native().to_dot_string(&variables, zero_pruned)
        }
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
