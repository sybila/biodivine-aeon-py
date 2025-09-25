use crate::bindings::lib_bdd::bdd_pointer::BddPointer;
use crate::bindings::lib_bdd::bdd_valuation::{BddPartialValuation, BddValuation};
use crate::bindings::lib_bdd::bdd_variable::BddVariable;
use crate::bindings::lib_bdd::bdd_variable_set::BddVariableSet;
use crate::bindings::lib_bdd::boolean_expression::BooleanExpression;
use crate::bindings::lib_bdd::op_function::{OpFunction2, OpFunction3};
use crate::{
    AsNative, runtime_error, throw_interrupted_error, throw_runtime_error, throw_type_error,
};
use biodivine_lib_bdd::Bdd as RsBdd;
use biodivine_lib_bdd::{BddPathIterator, BddSatisfyingValuations};
use macros::Wrapper;
use num_bigint::BigUint;
use num_traits::FromPrimitive;
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::io::Cursor;

/// BDD (binary decision diagram) is an acyclic, directed graph which is used to represent a
/// Boolean function. BDDs can be used to efficiently represent large sets of states, functions,
/// or spaces.
#[pyclass(module = "biodivine_aeon", frozen)]
#[derive(Clone)]
pub struct Bdd {
    ctx: Py<BddVariableSet>,
    value: RsBdd,
}

/// An iterator over all satisfying clauses (`BddPartialValuation`) of a `Bdd`.
///
/// Intuitively, these clauses together represent a disjunctive normal form of the underlying Boolean function.
#[pyclass(module = "biodivine_aeon")]
pub struct _BddClauseIterator(Py<Bdd>, BddPathIterator<'static>);

/// An iterator over all satisfying valuations (`BddValuation`) of a `Bdd`.
///
/// Intuitively, these valuation together represent the `1` rows of a truth table of the underlying Boolean function.
#[pyclass(module = "biodivine_aeon")]
pub struct _BddValuationIterator(Py<Bdd>, BddSatisfyingValuations<'static>);

#[pymethods]
impl Bdd {
    /// A `Bdd` can be created as:
    ///  - A copy of a different `Bdd`.
    ///  - A conjunction of literals defined by a `BddValuation` or a `BddPartialValuation`.
    ///  - Deserialization of a string created with `Bdd.data_string()`.
    ///  - Deserialization of bytes created with `Bdd.data_bytes()`.
    ///
    #[new]
    #[pyo3(signature = (ctx, data = None))]
    fn new(ctx: &Bound<'_, PyAny>, data: Option<&Bound<'_, PyAny>>) -> PyResult<Bdd> {
        // A copy of an existing BDD.
        if let Ok(bdd) = ctx.extract::<Bdd>() {
            if data.is_some() {
                return throw_type_error("Unexpected second argument.");
            }
            return Ok(bdd.clone());
        }
        // "Instantiate" a single valuation.
        if let Ok(valuation) = ctx.extract::<BddValuation>() {
            if data.is_some() {
                return throw_type_error("Unexpected second argument.");
            }
            let value = RsBdd::from(valuation.as_native().clone());
            return Ok(Bdd {
                ctx: valuation.__ctx__(),
                value,
            });
        }
        // "Instantiate" a partial valuation using a conjunction.
        if let Ok(valuation) = ctx.extract::<BddPartialValuation>() {
            if data.is_some() {
                return throw_type_error("Unexpected second argument.");
            }
            let ctx = valuation.__ctx__();
            let value = ctx
                .get()
                .as_native()
                .mk_conjunctive_clause(valuation.as_native());
            return Ok(Bdd { ctx, value });
        }
        // Load BDD from data (string or bytes).
        if let Ok(ctx) = ctx.extract::<Py<BddVariableSet>>() {
            match data {
                None => {
                    let value = ctx.get().as_native().mk_false();
                    Ok(Bdd { ctx, value })
                }
                Some(data) => {
                    if let Ok(bytes) = data.extract::<&[u8]>() {
                        let mut reader = Cursor::new(bytes);
                        match RsBdd::read_as_bytes(&mut reader) {
                            Ok(value) => Ok(Bdd { ctx, value }),
                            Err(e) => throw_runtime_error(format!("Cannot read `Bdd`: {e}")),
                        }
                    } else if let Ok(string) = data.extract::<String>() {
                        let mut reader = Cursor::new(string);
                        match RsBdd::read_as_string(&mut reader) {
                            Ok(value) => Ok(Bdd { ctx, value }),
                            Err(e) => throw_runtime_error(format!("Cannot read `Bdd`: {e}")),
                        }
                    } else {
                        throw_type_error("Expected `str` or `bytes`.")
                    }
                }
            }
        } else {
            throw_type_error(
                "Expected one of `Bdd`, `BddValuation`, `BddPartialValuation`, or `BddVariableSet`.",
            )
        }
    }

    fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.as_native().hash(&mut hasher);
        hasher.finish()
    }

    /// A `Bdd` implements semantic equality and implication/subset partial ordering.
    fn __richcmp__(&self, other: &Self, op: CompareOp) -> bool {
        match op {
            CompareOp::Lt => self.implies(other) && !self.semantic_eq(other),
            CompareOp::Le => self.implies(other),
            CompareOp::Eq => self.semantic_eq(other),
            CompareOp::Ne => !self.semantic_eq(other),
            CompareOp::Gt => other.implies(self) && !other.semantic_eq(self),
            CompareOp::Ge => other.implies(self),
        }
    }

    fn __str__(&self) -> String {
        format!(
            "Bdd(vars = {}, len = {}, cardinality = {})",
            self.variable_count(),
            self.__len__(),
            self.cardinality(true),
        )
    }

    fn __repr__(&self) -> String {
        format!(
            "Bdd({}, \"{}\")",
            self.ctx.get().__repr__(),
            self.data_string()
        )
    }

    fn __getnewargs__<'a>(&self, py: Python<'a>) -> (Py<BddVariableSet>, Bound<'a, PyBytes>) {
        (self.ctx.clone(), self.data_bytes(py))
    }

    pub fn __ctx__(&self) -> Py<BddVariableSet> {
        self.ctx.clone()
    }

    fn __call__(&self, py: Python, valuation: &Bound<'_, PyAny>) -> PyResult<bool> {
        if let Ok(valuation) = valuation.extract::<BddValuation>() {
            Ok(self.value.eval_in(valuation.as_native()))
        } else {
            let valuation = BddValuation::new(self.ctx.bind(py).as_any(), Some(valuation))?;
            Ok(self.value.eval_in(valuation.as_native()))
        }
    }

    fn __len__(&self) -> usize {
        self.node_count()
    }

    /// Convert this `Bdd` into a serialized `str` format that can be read using the `Bdd` constructor.
    fn data_string(&self) -> String {
        self.as_native().to_string()
    }

    /// Convert this `Bdd` into a serialized `bytes` format that can be read using the `Bdd` constructor.
    fn data_bytes<'a>(&self, py: Python<'a>) -> Bound<'a, PyBytes> {
        PyBytes::new(py, &self.as_native().to_bytes())
    }

    /// Produce a `graphviz`-compatible `.dot` representation of the underlying graph. If `zero_pruned` is set,
    /// edges leading to the `0` terminal are omitted for clarity.
    ///
    /// You can use this in Jupyter notebooks to visualize the BDD:
    /// ```python
    /// bdd = ...
    ///
    /// import graphviz
    /// graphviz.Source(bdd.to_dot())
    /// ```
    #[pyo3(signature = (zero_pruned = true))]
    pub fn to_dot(&self, zero_pruned: bool) -> String {
        self.as_native()
            .to_dot_string(self.ctx.get().as_native(), zero_pruned)
    }

    /// If this BDD is a terminal node, return its Boolean value. Otherwise, return `None`.
    fn as_bool(&self) -> Option<bool> {
        self.as_native().as_bool()
    }

    /// Produce a `BooleanExpression` which is logically equivalent to the function represented by this `Bdd`.
    ///
    /// The format uses an and-or expansion of the function graph, hence it is not very
    /// practical for complicated `Bdd` objects.
    fn to_expression(&self) -> BooleanExpression {
        BooleanExpression::from_native(
            self.as_native()
                .to_boolean_expression(self.ctx.get().as_native()),
        )
    }

    /// Build a list of `BddPartialValuation` objects that represents a **disjunctive normal form**
    /// of this Boolean function.
    ///
    /// When `optimize` is set to `True`, the method dynamically optimizes the BDD to obtain
    /// a smaller DNF. This process can be non-trivial for large BDDs. However, for such BDDs,
    /// we usually can't enumerate the full DNF anyway, hence the optimization is enabled by
    /// default. When `optimize` is set to `False`, the result should be equivalent to the
    /// actual canonical clauses of this BDD as reported by `Bdd.clause_iterator`.
    ///
    /// See also `BddVariableSet.mk_dnf`.
    #[pyo3(signature = (optimize = true, size_limit = None))]
    fn to_dnf(
        &self,
        py: Python,
        optimize: bool,
        size_limit: Option<usize>,
    ) -> PyResult<Vec<BddPartialValuation>> {
        let native = if optimize {
            self.as_native()._to_optimized_dnf(&|dnf| {
                if let Some(size_limit) = size_limit
                    && size_limit < dnf.len()
                {
                    return throw_interrupted_error(format!(
                        "Exceeded size limit of {size_limit} clauses"
                    ));
                }
                py.check_signals()
            })?
        } else {
            self.as_native().to_dnf()
        };
        Ok(native
            .into_iter()
            .map(|it| BddPartialValuation::new_raw(self.ctx.clone(), it))
            .collect())
    }

    /// Build a list of `BddPartialValuation` objects that represents a **conjunctive normal form** of this
    /// Boolean function.
    ///
    /// See also `BddVariableSet.mk_cnf`.
    fn to_cnf(&self) -> Vec<BddPartialValuation> {
        self.as_native()
            .to_cnf()
            .into_iter()
            .map(|it| BddPartialValuation::new_raw(self.ctx.clone(), it))
            .collect()
    }

    /// Return the number of graph nodes in this `Bdd`.
    fn node_count(&self) -> usize {
        self.as_native().size()
    }

    /// Return the number of decision nodes for each `BddVariable` that appears in this `Bdd`
    /// (i.e. in the `Bdd.support_set`).
    fn node_count_per_variable(&self) -> HashMap<BddVariable, usize> {
        self.as_native()
            .size_per_variable()
            .into_iter()
            .map(|(a, b)| (a.into(), b))
            .collect()
    }

    /// Test structural equality of two `Bdd` objects. Compared to normal `==`, this equality test
    /// matches the two BDD graphs exactly, node by node.
    ///
    /// As such, `Bdd.structural_eq` is *faster* than normal `==`, which performs semantic equality check. However,
    /// if a `Bdd` is not reduced, two semantically equivalent BDDs can be structurally different. For the vast
    /// majority of use cases, a `Bdd` is reduced, and hence `Bdd.structural_eq` and `==` gives the same result.
    /// However, if the `Bdd` comes from an unknown source (e.g. an external file), we cannot guarantee that it is
    /// reduced and using `Bdd.structural_eq` could be unreliable as an exact equality test.
    fn structural_eq(&self, other: &Bdd) -> bool {
        self.as_native().eq(other.as_native())
    }

    /// Compares two `Bdd` objects semantically (i.e. whether they compute the same Boolean function).
    ///
    /// This is also used by the `==` operator. It is slightly faster than computing `Bdd.l_iff`, as it does
    /// not need to create the full result, just check whether it is a tautology. However, it is slower than
    /// `Bdd.structural_eq`, because it needs to still explore the product graph of the two `Bdd` objects.
    pub fn semantic_eq(&self, other: &Bdd) -> bool {
        if self.as_native().num_vars() != other.as_native().num_vars() {
            return false;
        }

        // Semantic equality check needs to verify that A <=> B is a tautology,
        // which is the same as checking if A XOR B is a contradiction.
        // We actually don't have to compute the whole result though, we just need to know
        // if it is `false`. Hence, we can stop once the result size exceeds one node.
        // Other comparison operators are implemented using similar logic.
        RsBdd::binary_op_with_limit(
            1,
            self.as_native(),
            other.as_native(),
            biodivine_lib_bdd::op_function::xor,
        )
        .is_some()
    }

    /// Tests whether the function `self` implies the function `other`. In terms of sets, this can be interpreted
    /// as the subset relation (i.e. `self` being a subset of `other`).
    ///
    /// This is slightly faster than computing `Bdd.l_imp`, as it does not need to create the full result,
    /// just check whether it is a tautology.
    pub fn implies(&self, other: &Bdd) -> bool {
        // (A <= B) if and only if A implies B, meaning (!A or B) is a tautology.
        // Hence, (A & !B) must be a contradiction.
        RsBdd::binary_op_with_limit(
            1,
            self.as_native(),
            other.as_native(),
            biodivine_lib_bdd::op_function::and_not,
        )
        .is_some()
    }

    /// Return the `BddPointer` which references the root node of this `Bdd`.
    fn root(&self) -> BddPointer {
        self.as_native().root_pointer().into()
    }

    /// Return the `(low, high)` pointers of the given BDD node.
    fn node_links(&self, pointer: &BddPointer) -> (BddPointer, BddPointer) {
        let bdd = self.as_native();
        (
            bdd.low_link_of(*pointer.as_native()).into(),
            bdd.high_link_of(*pointer.as_native()).into(),
        )
    }

    /// Return the decision variable of the given BDD node. Returns `None` for terminal nodes.
    fn node_variable(&self, pointer: &BddPointer) -> Option<BddVariable> {
        if pointer.is_terminal() {
            None
        } else {
            Some(self.as_native().var_of(*pointer.as_native()).into())
        }
    }

    /// Return the number of variables that are admissible in this `Bdd`
    /// (equivalent to `BddVariableSet.variable_count`).
    fn variable_count(&self) -> usize {
        usize::from(self.as_native().num_vars())
    }

    /// Return the `set` of `BddVariable` identifiers that are actually actively used by this specific `Bdd`.
    fn support_set(&self) -> HashSet<BddVariable> {
        self.as_native()
            .support_set()
            .into_iter()
            .map(|it| it.into())
            .collect()
    }

    /// True if the given variable is present in the support set of this `Bdd`.
    fn support_set_contains(&self, variable: &Bound<'_, PyAny>) -> PyResult<bool> {
        let var = self.ctx.get().resolve_variable(variable)?;
        Ok(self.as_native().support_set_contains(&var))
    }

    /// Return a list of all BDD node pointers in this BDD.
    fn pointers(&self) -> Vec<BddPointer> {
        self.as_native().pointers().map(|it| it.into()).collect()
    }

    /// True if this `Bdd` represents a constant $false$ function.
    fn is_false(&self) -> bool {
        self.as_native().is_false()
    }

    /// True if this `Bdd` represents a constant $true$ function.
    fn is_true(&self) -> bool {
        self.as_native().is_true()
    }

    /// True if this `Bdd` represents a single conjunctive clause.
    ///
    /// In other words, it can be obtained as the result of `Bdd(BddPartialValuation(ctx, X))`
    /// for some value of `X`.
    fn is_clause(&self) -> bool {
        self.as_native().is_clause()
    }

    /// True if this `Bdd` represents a single valuation of *all* variables.
    ///
    /// In other words, it can be obtained as the result of `Bdd(BddValuation(ctx, X))`
    /// for some value of `X`.
    fn is_valuation(&self) -> bool {
        self.as_native().is_valuation()
    }

    /// Compute the cardinality (the number of satisfying valuations) of this `Bdd`.
    ///
    /// By default, the operation uses unbounded integers, since the cardinality can grow quite quickly. However, by
    /// setting `exact=False`, you can instead obtain a faster approximate result based on floating-point arithmetic.
    /// For results that exceed the `f64` maximal value (i.e. overflow to infinity), the method will still revert
    /// to unbounded integers.
    #[pyo3(signature = (exact = true))]
    pub fn cardinality(&self, exact: bool) -> BigUint {
        if exact {
            self.as_native().exact_cardinality()
        } else {
            let result = self.as_native().cardinality();
            if let Some(value) = BigUint::from_f64(result) {
                value
            } else {
                self.as_native().exact_cardinality()
            }
        }
    }

    /// Compute the number of canonical clauses of this `Bdd`. These are the disjunctive clauses
    /// reported by `Bdd.clause_iterator`. Clause cardinality can be thus used as the expected
    /// item count for this iterator.
    pub fn clause_cardinality(&self) -> BigUint {
        self.as_native().exact_clause_cardinality()
    }

    /// Computes a logical negation (i.e. $\neg f$) of this `Bdd`.
    pub fn l_not(&self) -> Bdd {
        self.new_from(self.as_native().not())
    }

    /// Computes a logical conjunction (i.e. $f \land g$) of two `Bdd` objects.
    ///
    /// Accepts an optional `limit` argument. If the number of nodes in the resulting `Bdd` exceeds this limit,
    /// the method terminates prematurely and throws an `InterruptedError` instead of returning a result.
    #[pyo3(signature = (other, limit = None))]
    pub fn l_and(&self, other: &Bdd, limit: Option<usize>) -> PyResult<Bdd> {
        if let Some(limit) = limit {
            let result = RsBdd::binary_op_with_limit(
                limit,
                self.as_native(),
                other.as_native(),
                biodivine_lib_bdd::op_function::and,
            );
            if let Some(result) = result {
                Ok(self.new_from(result))
            } else {
                throw_interrupted_error("BDD size limit exceeded.")
            }
        } else {
            let result = self.as_native().and(other.as_native());
            Ok(self.new_from(result))
        }
    }

    /// Computes a logical disjunction (i.e. $f \lor g$) of two `Bdd` objects.
    ///
    /// Accepts an optional `limit` argument. If the number of nodes in the resulting `Bdd` exceeds this limit,
    /// the method terminates prematurely and throws an `InterruptedError` instead of returning a result.
    #[pyo3(signature = (other, limit = None))]
    pub fn l_or(&self, other: &Bdd, limit: Option<usize>) -> PyResult<Bdd> {
        if let Some(limit) = limit {
            let result = RsBdd::binary_op_with_limit(
                limit,
                self.as_native(),
                other.as_native(),
                biodivine_lib_bdd::op_function::or,
            );
            if let Some(result) = result {
                Ok(self.new_from(result))
            } else {
                throw_interrupted_error("BDD size limit exceeded.")
            }
        } else {
            let result = self.as_native().or(other.as_native());
            Ok(self.new_from(result))
        }
    }

    /// Computes a logical implication (i.e. $f \Rightarrow g$) of two `Bdd` objects.
    ///
    /// Accepts an optional `limit` argument. If the number of nodes in the resulting `Bdd` exceeds this limit,
    /// the method terminates prematurely and throws an `InterruptedError` instead of returning a result.
    #[pyo3(signature = (other, limit = None))]
    pub fn l_imp(&self, other: &Bdd, limit: Option<usize>) -> PyResult<Bdd> {
        if let Some(limit) = limit {
            let result = RsBdd::binary_op_with_limit(
                limit,
                self.as_native(),
                other.as_native(),
                biodivine_lib_bdd::op_function::imp,
            );
            if let Some(result) = result {
                Ok(self.new_from(result))
            } else {
                throw_interrupted_error("BDD size limit exceeded.")
            }
        } else {
            let result = self.as_native().imp(other.as_native());
            Ok(self.new_from(result))
        }
    }

    /// Computes a logical equivalence (i.e. `f <=> g`, or `f = g`) of two `Bdd` objects.
    ///
    /// Accepts an optional `limit` argument. If the number of nodes in the resulting `Bdd` exceeds this limit,
    /// the method terminates prematurely and throws an `InterruptedError` instead of returning a result.
    #[pyo3(signature = (other, limit = None))]
    pub fn l_iff(&self, other: &Bdd, limit: Option<usize>) -> PyResult<Bdd> {
        if let Some(limit) = limit {
            let result = RsBdd::binary_op_with_limit(
                limit,
                self.as_native(),
                other.as_native(),
                biodivine_lib_bdd::op_function::iff,
            );
            if let Some(result) = result {
                Ok(self.new_from(result))
            } else {
                throw_interrupted_error("BDD size limit exceeded.")
            }
        } else {
            let result = self.as_native().iff(other.as_native());
            Ok(self.new_from(result))
        }
    }

    /// Computes an exclusive disjunction (i.e. $f \oplus g$, or $f \not= g$) of two `Bdd` objects.
    ///
    /// Accepts an optional `limit` argument. If the number of nodes in the resulting `Bdd` exceeds this limit,
    /// the method terminates prematurely and throws an `InterruptedError` instead of returning a result.
    #[pyo3(signature = (other, limit = None))]
    pub fn l_xor(&self, other: &Bdd, limit: Option<usize>) -> PyResult<Bdd> {
        if let Some(limit) = limit {
            let result = RsBdd::binary_op_with_limit(
                limit,
                self.as_native(),
                other.as_native(),
                biodivine_lib_bdd::op_function::xor,
            );
            if let Some(result) = result {
                Ok(self.new_from(result))
            } else {
                throw_interrupted_error("BDD size limit exceeded.")
            }
        } else {
            let result = self.as_native().xor(other.as_native());
            Ok(self.new_from(result))
        }
    }

    /// Computes a logical "and not" (i.e. $f \land \neg g$) of two `Bdd` objects.
    ///
    /// Accepts an optional `limit` argument. If the number of nodes in the resulting `Bdd` exceeds this limit,
    /// the method terminates prematurely and throws an `InterruptedError` instead of returning a result.
    #[pyo3(signature = (other, limit = None))]
    pub fn l_and_not(&self, other: &Bdd, limit: Option<usize>) -> PyResult<Bdd> {
        if let Some(limit) = limit {
            let result = RsBdd::binary_op_with_limit(
                limit,
                self.as_native(),
                other.as_native(),
                biodivine_lib_bdd::op_function::and_not,
            );
            if let Some(result) = result {
                Ok(self.new_from(result))
            } else {
                throw_interrupted_error("BDD size limit exceeded.")
            }
        } else {
            let result = self.as_native().and_not(other.as_native());
            Ok(self.new_from(result))
        }
    }

    /// A standard "if-then-else" ternary operation. It is equivalent to $(a \land b) \lor (\neg a \land c)$.
    /// Additional non-standard ternary operators are available through `Bdd::apply3`.
    #[staticmethod]
    pub fn if_then_else(condition: &Bdd, then: &Bdd, other: &Bdd) -> Bdd {
        let result =
            RsBdd::if_then_else(condition.as_native(), then.as_native(), other.as_native());
        condition.new_from(result)
    }

    /// Compute a custom binary operation on two `Bdd` objects.
    ///
    ///  - `function` must be of type `(None | bool, None | bool) -> None | bool` and implements the actual logical
    ///    operation that will be performed.
    ///  - `flip_left`, `flip_right`, and `flip_output` specify whether all low/high links of a specific variable
    ///    should be swapped in the corresponding `Bdd` (this is effectively a `x <- !x` substitution performed
    ///    "for free" by the internal algorithm).
    ///  - `limit` has the same meaning as in other logical operations: if specified, the method throws
    ///    `InterruptedError` if the size of the output `Bdd` exceeds the specified `limit`.
    #[pyo3(signature = (left, right, function, flip_left = None, flip_right = None, flip_output = None, limit = None))]
    #[staticmethod]
    #[allow(clippy::too_many_arguments)] // There isn't much to do about this
    pub fn apply2(
        py: Python,
        left: &Bdd,
        right: &Bdd,
        function: Py<PyAny>,
        flip_left: Option<&Bound<'_, PyAny>>,
        flip_right: Option<&Bound<'_, PyAny>>,
        flip_output: Option<&Bound<'_, PyAny>>,
        limit: Option<usize>,
    ) -> PyResult<Bdd> {
        let ctx = left.ctx.get();
        let flip_left = flip_left.map(|it| ctx.resolve_variable(it)).transpose()?;
        let flip_right = flip_right.map(|it| ctx.resolve_variable(it)).transpose()?;
        let flip_output = flip_output.map(|it| ctx.resolve_variable(it)).transpose()?;
        let function = OpFunction2::new(py, function)?;
        if let Some(limit) = limit {
            let result = RsBdd::fused_binary_flip_op_with_limit(
                limit,
                (left.as_native(), flip_left),
                (right.as_native(), flip_right),
                flip_output,
                |l, r| function.invoke(l, r),
            );
            if let Some(result) = result {
                Ok(left.new_from(result))
            } else {
                throw_interrupted_error("BDD size limit exceeded.")
            }
        } else {
            let result = RsBdd::fused_binary_flip_op(
                (left.as_native(), flip_left),
                (right.as_native(), flip_right),
                flip_output,
                |l, r| function.invoke(l, r),
            );
            Ok(left.new_from(result))
        }
    }

    /// The same as `Bdd.apply2`, but considers a ternary Boolean function instead of binary.
    ///
    /// Currently, the operation does not support `limit`, but this could be easily added in the future.
    #[staticmethod]
    #[pyo3(signature = (a, b, c, function, flip_a = None, flip_b = None, flip_c = None, flip_out = None))]
    #[allow(clippy::too_many_arguments)] // There isn't much to do about this
    pub fn apply3(
        py: Python,
        a: &Bdd,
        b: &Bdd,
        c: &Bdd,
        function: Py<PyAny>,
        flip_a: Option<&Bound<'_, PyAny>>,
        flip_b: Option<&Bound<'_, PyAny>>,
        flip_c: Option<&Bound<'_, PyAny>>,
        flip_out: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Bdd> {
        let ctx = a.ctx.get();
        let flip_a = flip_a.map(|it| ctx.resolve_variable(it)).transpose()?;
        let flip_b = flip_b.map(|it| ctx.resolve_variable(it)).transpose()?;
        let flip_c = flip_c.map(|it| ctx.resolve_variable(it)).transpose()?;
        let flip_out = flip_out.map(|it| ctx.resolve_variable(it)).transpose()?;
        let function = OpFunction3::new(py, function)?;
        let result = RsBdd::fused_ternary_flip_op(
            (a.as_native(), flip_a),
            (b.as_native(), flip_b),
            (c.as_native(), flip_c),
            flip_out,
            |x, y, z| function.invoke(x, y, z),
        );
        Ok(a.new_from(result))
    }

    /// Instead of actually performing an operation, computes useful "metadata" about it. This is faster than
    /// running the operation in full, because there is no need to save the result.
    ///
    /// Specifically, the result of this operation is a `bool` indicating whether the result is empty, and an `int`
    /// which counts the number of low-level "product nodes" that had to be explored in the operation. This "product
    /// node count" is typically a good indicator for the actual operation complexity.
    #[pyo3(signature = (left, right, function, flip_left = None, flip_right = None, flip_output = None))]
    #[staticmethod]
    pub fn check2(
        py: Python,
        left: &Bdd,
        right: &Bdd,
        function: Py<PyAny>,
        flip_left: Option<&Bound<'_, PyAny>>,
        flip_right: Option<&Bound<'_, PyAny>>,
        flip_output: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<(bool, usize)> {
        let ctx = left.ctx.get();
        let flip_left = flip_left.map(|it| ctx.resolve_variable(it)).transpose()?;
        let flip_right = flip_right.map(|it| ctx.resolve_variable(it)).transpose()?;
        let flip_output = flip_output.map(|it| ctx.resolve_variable(it)).transpose()?;
        let function = OpFunction2::new(py, function)?;
        Ok(RsBdd::check_fused_binary_flip_op(
            usize::MAX,
            (left.as_native(), flip_left),
            (right.as_native(), flip_right),
            flip_output,
            |l, r| function.invoke(l, r),
        )
        .unwrap())
    }

    /// An `apply` function which performs two nested passes of the apply algorithm:
    ///  - First, the `outer_function` is applied to combine the left and right BDDs.
    ///  - Then, for each node of the newly created BDD which conditions on one of the specified `variables`,
    ///    the method executes the `inner_function` on its two child nodes. The result replaces the original
    ///    output node.
    ///
    /// This operation can be used to implement various combinations of logic + projection. Specifically, using
    /// `inner_function = or` implements existential projection and `inner_function = and` implements universal
    /// projection on the result of the `outer_function`. However, much "wilder" combinations are possible if
    /// you, for whatever reason, need them.
    #[staticmethod]
    pub fn apply_nested(
        py: Python,
        left: &Bdd,
        right: &Bdd,
        variables: &Bound<'_, PyAny>,
        outer_function: Py<PyAny>,
        inner_function: Py<PyAny>,
    ) -> PyResult<Bdd> {
        let trigger = left.ctx.get().resolve_variables(variables)?;
        let trigger: HashSet<biodivine_lib_bdd::BddVariable> = HashSet::from_iter(trigger);
        let outer_function = OpFunction2::new(py, outer_function)?;
        let inner_function = OpFunction2::new(py, inner_function)?;
        let result = RsBdd::binary_op_nested(
            left.as_native(),
            right.as_native(),
            |x| trigger.contains(&x),
            |l, r| outer_function.invoke(l, r),
            |l, r| inner_function.invoke(l, r),
        );
        Ok(left.new_from(result))
    }

    /// Performs a binary logical operation (`function`) on two `Bdd` objects while performing an existential
    /// projection on the given variables in the result `Bdd`.
    ///
    /// See also `Bdd.apply_nested`.
    #[staticmethod]
    pub fn apply_with_exists(
        py: Python,
        left: &Bdd,
        right: &Bdd,
        variables: &Bound<'_, PyAny>,
        function: Py<PyAny>,
    ) -> PyResult<Bdd> {
        let variables = left.ctx.get().resolve_variables(variables)?;
        let function = OpFunction2::new(py, function)?;
        let result = RsBdd::binary_op_with_exists(
            left.as_native(),
            right.as_native(),
            |l, r| function.invoke(l, r),
            &variables,
        );
        Ok(left.new_from(result))
    }

    /// Performs a binary logical operation (`function`) on two `Bdd` objects while performing a universal
    /// projection on the given variables in the result `Bdd`.
    ///
    /// See also `Bdd.apply_nested`.
    #[staticmethod]
    pub fn apply_with_for_all(
        py: Python,
        left: &Bdd,
        right: &Bdd,
        variables: &Bound<'_, PyAny>,
        function: Py<PyAny>,
    ) -> PyResult<Bdd> {
        let variables = left.ctx.get().resolve_variables(variables)?;
        let function = OpFunction2::new(py, function)?;
        let result = RsBdd::binary_op_with_for_all(
            left.as_native(),
            right.as_native(),
            |l, r| function.invoke(l, r),
            &variables,
        );
        Ok(left.new_from(result))
    }

    /// Picks one "witness" valuation for the given `BddVariable` id(s) from this `Bdd`.
    ///
    /// To understand this operation, we split the `BddVariables` into `picked` (`variables`) and `non-picked`
    /// (the rest). For every unique valuation of `non-picked` variables, the operation selects exactly one valuation
    /// of `picked` variables from the original `Bdd` (assuming the `non-picked` valuation is present at all).
    /// In other words, the result is satisfied by every `non-picked` valuation that satisfied the original `Bdd`,
    /// but each such `non-picked` valuation corresponds to exactly one full valuation of `picked + non-picked`
    /// variables.
    ///
    /// Another useful way of understanding this operation is through relations: Consider a relation
    /// `R` which is of type `A \times B` and is represented as a `Bdd`. The result of `R.pick(variables_B)`, denoted $R'$
    /// is a *sub-relation* which is a bijection: for every $a \in A$ which is present in the original $R$,
    /// we selected exactly one $b \in B$ s.t. $(a,b) \in R' \land (a, b) \in R$. If we instead compute
    /// `R.pick(variables_A)`, we obtain a different bijection: one where we selected exactly $a \in A$ for every
    /// $b \in B$.
    ///
    /// This operation is biased such that it always tries to select the lexicographically "first" witness.
    pub fn r_pick(&self, variables: &Bound<'_, PyAny>) -> PyResult<Bdd> {
        let variables = self.ctx.get().resolve_variables(variables)?;
        Ok(self.new_from(self.as_native().pick(&variables)))
    }

    /// The semantics of this operation are the same as `Bdd.r_pick`, but instead of being biased towards the
    /// "first" available witness, the operation picks the witnesses randomly.
    ///
    /// You can make the process randomized but deterministic by specifying a fixed `seed`.
    #[pyo3(signature = (variables, seed = None))]
    pub fn r_pick_random(&self, variables: &Bound<'_, PyAny>, seed: Option<u64>) -> PyResult<Bdd> {
        /// Generic helper function to handle both cases.
        fn pick_random_rng<R: Rng>(
            bdd: &Bdd,
            variables: &[biodivine_lib_bdd::BddVariable],
            rng: &mut R,
        ) -> PyResult<Bdd> {
            Ok(bdd.new_from(bdd.as_native().pick_random(variables, rng)))
        }

        let variables = self.ctx.get().resolve_variables(variables)?;
        if let Some(seed) = seed {
            let mut rng = StdRng::seed_from_u64(seed);
            pick_random_rng(self, &variables, &mut rng)
        } else {
            pick_random_rng(self, &variables, &mut rand::thread_rng())
        }
    }

    /// Eliminate the specified `variables` from this `Bdd` using *existential projection*.
    ///
    /// In terms of first-order logic, this is equivalent to applying the $\exists$ operator to the underlying
    /// Boolean function.
    pub fn r_exists(&self, variables: &Bound<'_, PyAny>) -> PyResult<Bdd> {
        let variables = self.ctx.get().resolve_variables(variables)?;
        Ok(self.new_from(self.as_native().exists(&variables)))
    }

    /// Eliminate the specified `variables` from this `Bdd` using *universal projection*.
    ///
    /// In terms of first-order logic, this is equivalent to applying the $\forall$ operator to the underlying
    /// Boolean function.
    pub fn r_for_all(&self, variables: &Bound<'_, PyAny>) -> PyResult<Bdd> {
        let variables = self.ctx.get().resolve_variables(variables)?;
        Ok(self.new_from(self.as_native().for_all(&variables)))
    }

    /// Fix the specified variables to the respective values, and then eliminate the variables using existential
    /// projection.
    pub fn r_restrict(&self, values: &Bound<'_, PyAny>) -> PyResult<Bdd> {
        let valuation = self.ctx.get().resolve_partial_valuation(values)?;
        let result = self.as_native().restrict(&valuation.to_values());
        Ok(self.new_from(result))
    }

    /// Fix the specified variables to the respective values.
    pub fn r_select(&self, values: &Bound<'_, PyAny>) -> PyResult<Bdd> {
        let valuation = self.ctx.get().resolve_partial_valuation(values)?;
        let result = self.as_native().select(&valuation.to_values());
        Ok(self.new_from(result))
    }

    /// Pick a single satisfying valuation from this `Bdd`.
    pub fn witness(&self) -> PyResult<BddValuation> {
        self.as_native()
            .sat_witness()
            .map(|it| BddValuation::new_raw(self.ctx.clone(), it))
            .ok_or_else(|| runtime_error("BDD is empty."))
    }

    /// Pick the lexicographically first valuation from this `Bdd`.
    pub fn valuation_first(&self) -> PyResult<BddValuation> {
        self.as_native()
            .first_valuation()
            .map(|it| BddValuation::new_raw(self.ctx.clone(), it))
            .ok_or_else(|| runtime_error("BDD is empty."))
    }

    /// Pick the lexicographically last valuation from this `Bdd`.
    pub fn valuation_last(&self) -> PyResult<BddValuation> {
        self.as_native()
            .last_valuation()
            .map(|it| BddValuation::new_raw(self.ctx.clone(), it))
            .ok_or_else(|| runtime_error("BDD is empty."))
    }

    /// Pick a randomized valuation from this `Bdd`.
    ///
    /// Note: At the moment, the distribution of the selected valuations is not uniform (it depends on the structure
    /// of the `Bdd`). However, in the future we plan to update the method such that it actually samples
    /// the valuations uniformly. If this is important to you, get in touch :)
    ///
    /// You can make the process randomized but deterministic by specifying a fixed `seed`.
    #[pyo3(signature = (seed = None))]
    pub fn valuation_random(&self, seed: Option<u64>) -> PyResult<BddValuation> {
        fn inner<R: Rng>(bdd: &RsBdd, rng: &mut R) -> Option<biodivine_lib_bdd::BddValuation> {
            bdd.random_valuation(rng)
        }

        let result = if let Some(seed) = seed {
            let mut rng = StdRng::seed_from_u64(seed);
            inner(self.as_native(), &mut rng)
        } else {
            inner(self.as_native(), &mut rand::thread_rng())
        };
        result
            .map(|it| BddValuation::new_raw(self.ctx.clone(), it))
            .ok_or_else(|| runtime_error("BDD is empty."))
    }

    /// Pick the valuation with the most `true` variables.
    pub fn valuation_most_positive(&self) -> PyResult<BddValuation> {
        self.as_native()
            .most_positive_valuation()
            .map(|it| BddValuation::new_raw(self.ctx.clone(), it))
            .ok_or_else(|| runtime_error("BDD is empty."))
    }

    /// Pick the valuation with the most `false` variables.
    pub fn valuation_most_negative(&self) -> PyResult<BddValuation> {
        self.as_native()
            .most_negative_valuation()
            .map(|it| BddValuation::new_raw(self.ctx.clone(), it))
            .ok_or_else(|| runtime_error("BDD is empty."))
    }

    /// Create a uniform valuation sampler for this BDD, optionally initialized with a seed.
    ///
    /// Note that a uniform sampler is made specifically for a single BDD object and cannot
    /// be reused across different BDDs.
    #[pyo3(signature = (seed = None))]
    pub fn mk_uniform_valuation_sampler(&self, seed: Option<u64>) -> UniformValuationSampler {
        let rng = StdRng::seed_from_u64(seed.unwrap_or_default());
        UniformValuationSampler(self.as_native().mk_uniform_valuation_sampler(rng))
    }

    /// Create a naive valuation sampler for this BDD, optionally initialized with a seed.
    #[pyo3(signature = (seed = None))]
    pub fn mk_naive_valuation_sampler(&self, seed: Option<u64>) -> NaiveSampler {
        let rng = StdRng::seed_from_u64(seed.unwrap_or_default());
        NaiveSampler(biodivine_lib_bdd::random_sampling::NaiveSampler::from(rng))
    }

    /// Sample a random valuation using the provided sampler.
    pub fn random_valuation_sample(
        &self,
        sampler: &Bound<'_, PyAny>,
    ) -> PyResult<Option<BddValuation>> {
        let valuation = if let Ok(uniform_sampler) = sampler.cast::<UniformValuationSampler>() {
            let mut sampler_state = uniform_sampler.borrow_mut();
            self.as_native()
                .random_valuation_sample(sampler_state.as_native_mut())
        } else if let Ok(naive_sampler) = sampler.cast::<NaiveSampler>() {
            let mut sampler_state = naive_sampler.borrow_mut();
            self.as_native()
                .random_valuation_sample(sampler_state.as_native_mut())
        } else {
            return throw_type_error("Expected `UniformValuationSampler` or `NaiveSampler`.");
        };
        Ok(valuation.map(|it| BddValuation::new_raw(self.ctx.clone(), it)))
    }

    /// An iterator over all `BddValuation` objects that satisfy this `Bdd`.
    pub fn valuation_iterator(self_: Py<Bdd>, py: Python) -> _BddValuationIterator {
        _BddValuationIterator::new(py, self_)
    }

    /// Pick the lexicographically first satisfying clause of this `Bdd`.
    pub fn clause_first(&self) -> PyResult<BddPartialValuation> {
        self.as_native()
            .first_clause()
            .map(|it| BddPartialValuation::new_raw(self.ctx.clone(), it))
            .ok_or_else(|| runtime_error("BDD is empty."))
    }

    /// Pick the lexicographically last satisfying clause of this `Bdd`.
    pub fn clause_last(&self) -> PyResult<BddPartialValuation> {
        self.as_native()
            .last_clause()
            .map(|it| BddPartialValuation::new_raw(self.ctx.clone(), it))
            .ok_or_else(|| runtime_error("BDD is empty."))
    }

    /// Pick a randomized satisfying clause from this `Bdd`.
    ///
    /// Note: At the moment, the distribution of the selected clauses is not uniform (it depends on the structure
    /// of the `Bdd`). However, in the future we plan to update the method such that it actually samples
    /// the clauses uniformly. If this is important to you, get in touch :)
    ///
    /// You can make the process randomized but deterministic by specifying a fixed `seed`.
    #[pyo3(signature = (seed = None))]
    pub fn clause_random(&self, seed: Option<u64>) -> PyResult<BddPartialValuation> {
        fn inner<R: Rng>(
            bdd: &RsBdd,
            rng: &mut R,
        ) -> Option<biodivine_lib_bdd::BddPartialValuation> {
            bdd.random_clause(rng)
        }
        let result = if let Some(seed) = seed {
            let mut rng = StdRng::seed_from_u64(seed);
            inner(self.as_native(), &mut rng)
        } else {
            inner(self.as_native(), &mut rand::thread_rng())
        };
        result
            .map(|it| BddPartialValuation::new_raw(self.ctx.clone(), it))
            .ok_or_else(|| runtime_error("BDD is empty."))
    }

    /// Compute the most restrictive conjunctive clause that covers all satisfying valuations of this BDD.
    ///
    /// In other words, if you compute the BDD corresponding to the resulting partial valuation, the resulting BDD
    /// will be a superset of this BDD, and it will be the smallest superset that can be described using
    /// a single clause.
    pub fn clause_necessary(&self) -> PyResult<BddPartialValuation> {
        self.as_native()
            .necessary_clause()
            .map(|it| BddPartialValuation::new_raw(self.ctx.clone(), it))
            .ok_or_else(|| runtime_error("BDD is empty."))
    }

    /// Compute the `BddPartialValuation` that occurs among the `Bdd.clause_iterator` items and
    /// has the highest amount of fixed variables.
    ///
    /// Note that this is not the most fixed valuation among *all valuations* that
    /// satisfy this function (that is always a full valuation of all BDD variables). In other
    /// words, the result of this operation tells you more about the *structure* of a `Bdd` than
    /// the underlying Boolean function itself.
    pub fn clause_most_fixed(&self) -> PyResult<BddPartialValuation> {
        self.as_native()
            .most_fixed_clause()
            .map(|it| BddPartialValuation::new_raw(self.ctx.clone(), it))
            .ok_or_else(|| runtime_error("BDD is empty."))
    }

    /// Compute the `BddPartialValuation` that occurs among the `Bdd.clause_iterator` items and
    /// has the lowest amount of fixed variables.
    ///
    /// Note that this is not the most free valuation among *all valuations* that
    /// satisfy this function (that would require a more thorough optimization algorithm). In other
    /// words, the result of this operation tells you more about the *structure* of a `Bdd` than
    /// the underlying Boolean function itself.
    pub fn clause_most_free(&self) -> PyResult<BddPartialValuation> {
        self.as_native()
            .most_free_clause()
            .map(|it| BddPartialValuation::new_raw(self.ctx.clone(), it))
            .ok_or_else(|| runtime_error("BDD is empty."))
    }

    /// An iterator over all DNF clauses (i.e. `BddPartialValuation` objects) that satisfy this `Bdd`.
    pub fn clause_iterator(self_: Py<Bdd>, py: Python) -> _BddClauseIterator {
        _BddClauseIterator::new(py, self_)
    }

    /// Replace the occurrence of a given `variable` with a specific `function`.
    ///
    /// Note that at the moment, the result is not well-defined if `function` also depends on the substituted
    /// variable (the variable is eliminated both from the original `Bdd` and from `function`). We are planning
    /// to fix this in the future.
    pub fn substitute(&self, variable: &Bound<'_, PyAny>, function: &Bdd) -> PyResult<Bdd> {
        let ctx = self.ctx.get();
        let variable = ctx.resolve_variable(variable)?;
        let result = self.as_native().substitute(variable, function.as_native());
        Ok(self.new_from(result))
    }

    /// Rename `Bdd` variables based on the provided `(from, to)` pairs.
    ///
    /// At the moment, this operation *cannot* modify the graph structure of the `Bdd`. It can only replace variable
    /// identifiers with new ones. As such, rename operation is only permitted if it does not violate
    /// the current ordering. If this is not satisfied, the method panics.
    pub fn rename(&self, py: Python, replace_with: Vec<(Py<PyAny>, Py<PyAny>)>) -> PyResult<Bdd> {
        let mut permutation = HashMap::new();
        for (a, b) in replace_with {
            let a = self.ctx.get().resolve_variable(a.bind(py))?;
            let b = self.ctx.get().resolve_variable(b.bind(py))?;
            permutation.insert(a, b);
        }
        let mut result = self.value.clone();
        unsafe {
            result.rename_variables(&permutation);
        }
        Ok(self.new_from(result))
    }

    /// Raise a `RuntimeError` if this BDD violates some internal invariants.
    ///
    /// Normally, all BDDs should satisfy internal invariants at all times, but in case we load
    /// a corrupted BDD file or the BDD is transferred from some other implementation other than
    /// our own, the BDD structure might become corrupted.
    pub fn validate(&self) -> PyResult<()> {
        self.as_native().validate().map_err(runtime_error)
    }

    /// Compute valuation weights of each node. This can be useful for implementing custom
    /// sampling strategies. The return value is a list of non-negative integers indexed
    /// by the BDD node indices (corresponding to `BddPointer` values).
    pub fn node_valuation_weights(&self) -> Vec<BigUint> {
        self.as_native().node_valuation_weights()
    }

    /// Over-approximate this BDD to have at most `target_size` decision nodes.
    pub fn overapproximate_to_size(&self, target_size: usize) -> Bdd {
        self.new_from(self.as_native().overapproximate_to_size(target_size))
    }

    /// Under-approximate this BDD to have at most `target_size` decision nodes.
    pub fn underapproximate_to_size(&self, target_size: usize) -> Bdd {
        self.new_from(self.as_native().underapproximate_to_size(target_size))
    }

    /// Over-approximate this BDD by eliminating the specified decision nodes.
    pub fn overapproximate(&self, to_eliminate: Vec<BddPointer>) -> PyResult<Bdd> {
        let nodes: Vec<biodivine_lib_bdd::BddPointer> =
            to_eliminate.into_iter().map(|p| *p.as_native()).collect();
        Ok(self.new_from(self.as_native().overapproximate(&nodes)))
    }

    /// Under-approximate this BDD by eliminating the specified decision nodes.
    pub fn underapproximate(&self, to_eliminate: Vec<BddPointer>) -> PyResult<Bdd> {
        let nodes: Vec<biodivine_lib_bdd::BddPointer> =
            to_eliminate.into_iter().map(|p| *p.as_native()).collect();
        Ok(self.new_from(self.as_native().underapproximate(&nodes)))
    }

    /// Compute a new `Bdd` which over-approximates this `Bdd`, and its
    /// cardinality is greater or equal to the given `target`.
    pub fn overapproximate_to_cardinality(&self, target: BigUint) -> PyResult<Bdd> {
        Ok(self.new_from(self.as_native().overapproximate_to_cardinality(&target)))
    }

    /// Compute a new `Bdd` which under-approximates this `Bdd`, and its
    /// cardinality is less or equal to the given `target`.
    pub fn underapproximate_to_cardinality(&self, target: BigUint) -> PyResult<Bdd> {
        Ok(self.new_from(self.as_native().underapproximate_to_cardinality(&target)))
    }
}

impl AsNative<RsBdd> for Bdd {
    fn as_native(&self) -> &RsBdd {
        &self.value
    }

    fn as_native_mut(&mut self) -> &mut RsBdd {
        &mut self.value
    }
}

impl Bdd {
    /// A helper constructor which creates the context from a dynamic Python reference.
    pub fn new_raw(ctx: PyRef<'_, BddVariableSet>, value: RsBdd) -> Bdd {
        Bdd {
            ctx: ctx.into(),
            value,
        }
    }

    pub fn new_raw_2(ctx: Py<BddVariableSet>, value: RsBdd) -> Bdd {
        Bdd { ctx, value }
    }

    /// A helper constructor that copies the context from the current `Bdd`.
    pub fn new_from(&self, value: RsBdd) -> Bdd {
        Bdd {
            ctx: self.ctx.clone(),
            value,
        }
    }
}

#[pymethods]
impl _BddValuationIterator {
    #[new]
    pub fn new(py: Python, bdd: Py<Bdd>) -> _BddValuationIterator {
        // This hack allows us to "launder" lifetimes between Rust and Python.
        // It is only safe because we copy the `Bdd` and attach it to the "laundered" reference,
        // so there is no (realistic) way the reference can outlive the copy of the `Bdd`.
        // Fortunately, the iterator items are clones and do not reference the `Bdd` directly,
        // so the "laundered" pointer does not spread beyond the internal code of the iterator.
        let iterator = {
            let bdd_ref = bdd.borrow(py);
            let bdd_ref: &'static RsBdd =
                unsafe { (bdd_ref.as_native() as *const RsBdd).as_ref().unwrap() };
            bdd_ref.sat_valuations()
        };
        _BddValuationIterator(bdd, iterator)
    }

    pub fn __str__(&self) -> String {
        format!("BddValuationIterator({})", self.0.get().__str__())
    }

    pub fn __repr__(&self) -> String {
        format!("<{}>", self.__str__())
    }

    pub fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    pub fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<BddValuation> {
        slf.1.next().map(|it| {
            let ctx = slf.0.get().__ctx__();
            BddValuation::new_raw(ctx, it)
        })
    }

    pub fn next(slf: PyRefMut<'_, Self>) -> Option<BddValuation> {
        _BddValuationIterator::__next__(slf)
    }
}

#[pymethods]
impl _BddClauseIterator {
    #[new]
    pub fn new(py: Python, bdd: Py<Bdd>) -> _BddClauseIterator {
        // See `BddValuationIterator` for discussion why this is safe.
        let iterator = {
            let bdd_ref = bdd.borrow(py);
            let bdd_ref: &'static RsBdd =
                unsafe { (bdd_ref.as_native() as *const RsBdd).as_ref().unwrap() };
            bdd_ref.sat_clauses()
        };
        _BddClauseIterator(bdd, iterator)
    }

    pub fn __str__(&self) -> String {
        format!("BddClauseIterator({})", self.0.get().__str__())
    }

    pub fn __repr__(&self) -> String {
        format!("<{}>", self.__str__())
    }

    pub fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    pub fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<BddPartialValuation> {
        slf.1.next().map(|it| {
            let ctx = slf.0.get().__ctx__();
            BddPartialValuation::new_raw(ctx, it)
        })
    }

    pub fn next(slf: PyRefMut<'_, Self>) -> Option<BddPartialValuation> {
        Self::__next__(slf)
    }
}

/// A naive valuation sampler for BDDs.
#[pyclass(module = "biodivine_aeon")]
#[derive(Clone, Wrapper)]
pub struct NaiveSampler(biodivine_lib_bdd::random_sampling::NaiveSampler<StdRng>);

/// A uniform valuation sampler for BDDs.
#[pyclass(module = "biodivine_aeon")]
#[derive(Clone, Wrapper)]
pub struct UniformValuationSampler(
    biodivine_lib_bdd::random_sampling::UniformValuationSampler<StdRng>,
);
