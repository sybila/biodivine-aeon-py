use crate::bindings::lib_param_bn::argument_types::bool_type::BoolType;
use crate::pyo3_utils::richcmp_eq_by_key;
use crate::{throw_runtime_error, throw_type_error};
use RsBooleanExpression::{Not, Variable};
use biodivine_lib_bdd::boolean_expression::BooleanExpression as RsBooleanExpression;
use biodivine_lib_bdd::boolean_expression::BooleanExpression::{And, Cond, Iff, Imp, Or, Xor};
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};
use std::collections::HashSet;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
/*
   “Pretend to be good always and even God will be fooled.”
                                   — Kurt Vonnegut

   Since we cannot properly return references to BooleanExpression subtrees to Python,
   we instead use a special "reference type" which maintains a reference-counted root pointer
   as well as an unsafe "static" reference which actually only references the root pointer,
   hence it should live long enough. This is the same mechanism we use to make "owned" iterators,
   but here it is extended a bit further by allowing multiple immutable references to the same
   structure, as long as the reference counter ensures safety.
*/

/// Represents a simple Boolean expression with support for Boolean constants, string variables, negation,
/// and common binary operators (`and`/`or`/`imp`/`iff`/`xor`).
///
/// Expressions can be converted to/from `Bdd` objects.
///
/// ```python
/// vars = BddVariableSet(["a", "b", "c"])
///
/// # The expressions are syntactically different, but represent the same function.
/// expr_x = BooleanExpression("(a & b) | (!b & c)")
/// expr_y = BooleanExpression("(c & !b) | (b & a)")
///
/// assert vars.eval_expression(expr_x) == vars.eval_expression(expr_y)
/// ```
///
///
#[pyclass(module = "biodivine_aeon", frozen)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BooleanExpression {
    root: Arc<RsBooleanExpression>,
    value: &'static RsBooleanExpression,
}

#[pymethods]
impl BooleanExpression {
    /// Build a new `BooleanExpression`, either as a copy of an existing expression, or from a string representation.
    #[new]
    fn new(value: &Bound<'_, PyAny>) -> PyResult<BooleanExpression> {
        Self::resolve_expression(value)
    }

    fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.__str__().hash(&mut hasher);
        hasher.finish()
    }

    fn __richcmp__(&self, py: Python, other: &Self, op: CompareOp) -> PyResult<Py<PyAny>> {
        richcmp_eq_by_key(py, op, &self, &other, |it| it.as_native())
    }

    pub fn __str__(&self) -> String {
        self.as_native().to_string()
    }

    fn __repr__(&self) -> String {
        format!("BooleanExpression({:?})", self.__str__())
    }

    fn __getnewargs__<'a>(&self, py: Python<'a>) -> PyResult<Bound<'a, PyTuple>> {
        // Technically, this is a "different" expression because it is created with a completely new `root`,
        // but it is much easier (and more transparent) than serializing the root expression and trying to figure
        // out how to serialize a pointer into the AST.
        PyTuple::new(py, [self.__str__()])
    }

    fn __root__(&self) -> BooleanExpression {
        Self::new_raw(self.root.clone())
    }

    #[pyo3(signature = (valuation = None, **kwargs))]
    pub fn __call__(
        &self,
        py: Python,
        valuation: Option<&Bound<'_, PyDict>>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<bool> {
        match (valuation, kwargs) {
            (Some(_), Some(_)) => throw_type_error("Cannot use both explicit and named arguments."),
            (None, None) => eval(self.as_native(), &PyDict::new(py)),
            (Some(v), None) | (None, Some(v)) => eval(self.as_native(), v),
        }
    }

    /// Return a `BooleanExpression` of a constant value.
    #[staticmethod]
    pub fn mk_const(value: BoolType) -> PyResult<BooleanExpression> {
        Ok(Self::from_native(RsBooleanExpression::Const(value.bool())))
    }

    /// Return a `BooleanExpression` of a single named variable.
    #[staticmethod]
    pub fn mk_var(name: String) -> BooleanExpression {
        Self::from_native(Variable(name))
    }

    /// Return a negation of a `BooleanExpression`.
    #[staticmethod]
    pub fn mk_not(value: &BooleanExpression) -> BooleanExpression {
        Self::from_native(Not(Box::new(value.as_native().clone())))
    }

    /// Return an `and` of two `BooleanExpression` values.
    #[staticmethod]
    pub fn mk_and(left: &BooleanExpression, right: &BooleanExpression) -> BooleanExpression {
        Self::from_native(And(
            Box::new(left.as_native().clone()),
            Box::new(right.as_native().clone()),
        ))
    }

    /// Return an `or` of two `BooleanExpression` values.
    #[staticmethod]
    pub fn mk_or(left: &BooleanExpression, right: &BooleanExpression) -> BooleanExpression {
        Self::from_native(Or(
            Box::new(left.as_native().clone()),
            Box::new(right.as_native().clone()),
        ))
    }

    /// Return an `imp` of two `BooleanExpression` values.
    #[staticmethod]
    pub fn mk_imp(left: &BooleanExpression, right: &BooleanExpression) -> BooleanExpression {
        Self::from_native(Imp(
            Box::new(left.as_native().clone()),
            Box::new(right.as_native().clone()),
        ))
    }

    /// Return an `iff` of two `BooleanExpression` values.
    #[staticmethod]
    pub fn mk_iff(left: &BooleanExpression, right: &BooleanExpression) -> BooleanExpression {
        Self::from_native(Iff(
            Box::new(left.as_native().clone()),
            Box::new(right.as_native().clone()),
        ))
    }

    /// Return a `xor` of two `BooleanExpression` values.
    #[staticmethod]
    pub fn mk_xor(left: &BooleanExpression, right: &BooleanExpression) -> BooleanExpression {
        Self::from_native(Xor(
            Box::new(left.as_native().clone()),
            Box::new(right.as_native().clone()),
        ))
    }

    /// Return an IF-THEN-ELSE condition of thee `BooleanExpression` values.
    #[staticmethod]
    pub fn mk_cond(
        e_if: &BooleanExpression,
        e_then: &BooleanExpression,
        e_else: &BooleanExpression,
    ) -> BooleanExpression {
        Self::from_native(Cond(
            Box::new(e_if.as_native().clone()),
            Box::new(e_then.as_native().clone()),
            Box::new(e_else.as_native().clone()),
        ))
    }

    /// Build an expression which is equivalent to the conjunction of the given items.
    #[staticmethod]
    pub fn mk_conjunction(items: Vec<BooleanExpression>) -> BooleanExpression {
        fn rec(items: &[BooleanExpression]) -> BooleanExpression {
            if items.is_empty() {
                // Empty conjunction is `true`.
                return BooleanExpression::from_native(RsBooleanExpression::Const(true));
            }
            if items.len() == 1 {
                return items[0].clone();
            }
            if items.len() == 2 {
                return BooleanExpression::mk_and(&items[0], &items[1]);
            }

            let first = items.first().unwrap();
            let rest = rec(&items[1..]);
            BooleanExpression::mk_and(first, &rest)
        }

        rec(&items)
    }

    /// Build an expression which is equivalent to the disjunction of the given items.
    #[staticmethod]
    pub fn mk_disjunction(items: Vec<BooleanExpression>) -> BooleanExpression {
        fn rec(items: &[BooleanExpression]) -> BooleanExpression {
            if items.is_empty() {
                // Empty disjunction is `false`.
                return BooleanExpression::from_native(RsBooleanExpression::Const(false));
            }
            if items.len() == 1 {
                return items[0].clone();
            }
            if items.len() == 2 {
                return BooleanExpression::mk_or(&items[0], &items[1]);
            }

            let first = items.first().unwrap();
            let rest = rec(&items[1..]);
            BooleanExpression::mk_or(first, &rest)
        }

        rec(&items)
    }

    /// Return true if the root of this expression is a constant.
    pub fn is_const(&self) -> bool {
        matches!(self.as_native(), &RsBooleanExpression::Const(_))
    }

    /// Return true if the root of this expression is a variable.
    pub fn is_var(&self) -> bool {
        matches!(self.as_native(), &Variable(_))
    }

    /// Return true if the root of this expression is a `not`.
    pub fn is_not(&self) -> bool {
        matches!(self.as_native(), &Not(_))
    }

    /// Return true if the root of this expression is an `and`.
    pub fn is_and(&self) -> bool {
        matches!(self.as_native(), &And(_, _))
    }

    /// Return true if the root of this expression is an `or`.
    pub fn is_or(&self) -> bool {
        matches!(self.as_native(), &Or(_, _))
    }

    /// Return true if the root of this expression is an `imp`.
    pub fn is_imp(&self) -> bool {
        matches!(self.as_native(), &Imp(_, _))
    }

    /// Return true if the root of this expression is an `iff`.
    pub fn is_iff(&self) -> bool {
        matches!(self.as_native(), &Iff(_, _))
    }

    /// Return true if the root of this expression is a `xor`.
    pub fn is_xor(&self) -> bool {
        matches!(self.as_native(), &Xor(_, _))
    }

    /// Return true if the root of this expression is an IF-THEN-ELSE condition.
    pub fn is_cond(&self) -> bool {
        matches!(self.as_native(), &Cond(_, _, _))
    }

    /// Return true if the root of this expression is a literal (`var`/`!var`).
    pub fn is_literal(&self) -> bool {
        match self.as_native() {
            Variable(_) => true,
            Not(inner) => {
                matches!(**inner, Variable(_))
            }
            _ => false,
        }
    }

    /// Return true if the root of this expression is a binary operator (`and`/`or`/`imp`/`iff`/`xor`).
    pub fn is_binary(&self) -> bool {
        matches!(
            self.as_native(),
            And(_, _) | Or(_, _) | Xor(_, _) | Imp(_, _) | Iff(_, _)
        )
    }

    /// If the root of this expression is a constant, return its value, or `None` otherwise.
    pub fn as_const(&self) -> Option<bool> {
        match self.as_native() {
            RsBooleanExpression::Const(x) => Some(*x),
            _ => None,
        }
    }

    /// If the root of this expression is a `var`, return its name, or `None` otherwise.
    pub fn as_var(&self) -> Option<String> {
        match self.as_native() {
            Variable(x) => Some(x.clone()),
            _ => None,
        }
    }

    /// If the root of this expression is a `not`, return its operand, or `None` otherwise.
    pub fn as_not(&self) -> Option<BooleanExpression> {
        match self.as_native() {
            Not(x) => Some(self.mk_child_ref(x)),
            _ => None,
        }
    }

    /// If the root of this expression is an `and`, return its two operands, or `None` otherwise.
    pub fn as_and(&self) -> Option<(BooleanExpression, BooleanExpression)> {
        match self.as_native() {
            And(l, r) => Some((self.mk_child_ref(l), self.mk_child_ref(r))),
            _ => None,
        }
    }

    /// If the root of this expression is an `or`, return its two operands, or `None` otherwise.
    pub fn as_or(&self) -> Option<(BooleanExpression, BooleanExpression)> {
        match self.as_native() {
            Or(l, r) => Some((self.mk_child_ref(l), self.mk_child_ref(r))),
            _ => None,
        }
    }

    /// If the root of this expression is an `imp`, return its two operands, or `None` otherwise.
    pub fn as_imp(&self) -> Option<(BooleanExpression, BooleanExpression)> {
        match self.as_native() {
            Imp(l, r) => Some((self.mk_child_ref(l), self.mk_child_ref(r))),
            _ => None,
        }
    }

    /// If the root of this expression is an `iff`, return its two operands, or `None` otherwise.
    pub fn as_iff(&self) -> Option<(BooleanExpression, BooleanExpression)> {
        match self.as_native() {
            Iff(l, r) => Some((self.mk_child_ref(l), self.mk_child_ref(r))),
            _ => None,
        }
    }

    /// If the root of this expression is `xor`, return its two operands, or `None` otherwise.
    pub fn as_xor(&self) -> Option<(BooleanExpression, BooleanExpression)> {
        match self.as_native() {
            Xor(l, r) => Some((self.mk_child_ref(l), self.mk_child_ref(r))),
            _ => None,
        }
    }

    /// If the root of this expression is an IF-THEN-ELSE, return its three operands,
    /// or `None` otherwise.
    pub fn as_cond(&self) -> Option<(BooleanExpression, BooleanExpression, BooleanExpression)> {
        match self.as_native() {
            Cond(e_if, e_then, e_else) => Some((
                self.mk_child_ref(e_if),
                self.mk_child_ref(e_then),
                self.mk_child_ref(e_else),
            )),
            _ => None,
        }
    }

    /// If this expression is either `var` or `!var`, return the name of the variable and whether it is positive.
    /// Otherwise, return `None`.
    pub fn as_literal(&self) -> Option<(String, bool)> {
        match self.as_native() {
            Variable(name) => Some((name.clone(), true)),
            Not(inner) => match inner.as_ref() {
                Variable(name) => Some((name.clone(), false)),
                _ => None,
            },
            _ => None,
        }
    }

    /// If the root of this expression is one of the `and`/`or`/`imp`/`iff`/`xor` operators, return the name of the
    /// operator and its two operands. Returns `None` if the root is not a binary operator.
    pub fn as_binary(&self) -> Option<(String, BooleanExpression, BooleanExpression)> {
        match self.as_native() {
            And(l, r) => Some((
                "and".to_string(),
                self.mk_child_ref(l),
                self.mk_child_ref(r),
            )),
            Or(l, r) => Some(("or".to_string(), self.mk_child_ref(l), self.mk_child_ref(r))),
            Imp(l, r) => Some((
                "imp".to_string(),
                self.mk_child_ref(l),
                self.mk_child_ref(r),
            )),
            Iff(l, r) => Some((
                "iff".to_string(),
                self.mk_child_ref(l),
                self.mk_child_ref(r),
            )),
            Xor(l, r) => Some((
                "xor".to_string(),
                self.mk_child_ref(l),
                self.mk_child_ref(r),
            )),
            _ => None,
        }
    }

    /// Return the set of Boolean variable names that appear in this `BooleanExpression`.
    pub fn support_set(&self) -> HashSet<String> {
        self.as_native().support_set()
    }
}

impl BooleanExpression {
    pub fn from_native(expression: RsBooleanExpression) -> BooleanExpression {
        let root = Arc::new(expression);
        Self::new_raw(root)
    }

    pub fn new_raw(root: Arc<RsBooleanExpression>) -> BooleanExpression {
        let root_ref = root.as_ref();
        let value: &'static RsBooleanExpression =
            unsafe { (root_ref as *const RsBooleanExpression).as_ref().unwrap() };
        BooleanExpression { root, value }
    }

    pub fn mk_child_ref(&self, child: &RsBooleanExpression) -> BooleanExpression {
        let root = self.root.clone();
        let value: &'static RsBooleanExpression =
            unsafe { (child as *const RsBooleanExpression).as_ref().unwrap() };
        BooleanExpression { root, value }
    }

    pub fn resolve_expression(value: &Bound<'_, PyAny>) -> PyResult<BooleanExpression> {
        if let Ok(expression) = value.extract::<BooleanExpression>() {
            return Ok(expression);
        }
        if let Ok(value) = value.extract::<String>() {
            return match RsBooleanExpression::try_from(value.as_str()) {
                Ok(expression) => Ok(BooleanExpression::from_native(expression)),
                Err(message) => throw_runtime_error(format!("Invalid expression: \"{message}\".")),
            };
        }
        throw_type_error("Expected `BooleanExpression` or `str`.")
    }

    pub fn as_native(&self) -> &RsBooleanExpression {
        self.value
    }
}

fn eval(e: &RsBooleanExpression, valuation: &Bound<'_, PyDict>) -> PyResult<bool> {
    match e {
        RsBooleanExpression::Const(x) => Ok(*x),
        Variable(name) => {
            let Some(value) = valuation.get_item(name)? else {
                return throw_runtime_error(format!("Missing value of {name}."));
            };
            value.extract::<BoolType>().map(bool::from)
        }
        Not(inner) => {
            let inner = eval(inner, valuation)?;
            Ok(!inner)
        }
        And(left, right) => {
            let left = eval(left, valuation)?;
            let right = eval(right, valuation)?;
            Ok(left && right)
        }
        Or(left, right) => {
            let left = eval(left, valuation)?;
            let right = eval(right, valuation)?;
            Ok(left || right)
        }
        Xor(left, right) => {
            let left = eval(left, valuation)?;
            let right = eval(right, valuation)?;
            Ok(left != right)
        }
        Imp(left, right) => {
            let left = eval(left, valuation)?;
            let right = eval(right, valuation)?;
            Ok(!left || right)
        }
        Iff(left, right) => {
            let left = eval(left, valuation)?;
            let right = eval(right, valuation)?;
            Ok(left == right)
        }
        Cond(test, branch1, branch2) => {
            if eval(test, valuation)? {
                eval(branch1, valuation)
            } else {
                eval(branch2, valuation)
            }
        }
    }
}
