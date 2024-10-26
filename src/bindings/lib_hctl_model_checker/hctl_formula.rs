use crate::bindings::lib_param_bn::symbolic::asynchronous_graph::AsynchronousGraph;
use crate::bindings::lib_param_bn::symbolic::symbolic_context::SymbolicContext;
use crate::pyo3_utils::richcmp_eq_by_key;
use crate::{throw_runtime_error, throw_type_error, AsNative};
use biodivine_hctl_model_checker::mc_utils::{
    check_hctl_var_support, collect_unique_hctl_vars, collect_unique_wild_cards,
};
use biodivine_hctl_model_checker::preprocessing::hctl_tree::{HctlTreeNode, NodeType};
use biodivine_hctl_model_checker::preprocessing::operator_enums::{
    Atomic, BinaryOp, HybridOp, UnaryOp,
};
use biodivine_hctl_model_checker::preprocessing::parser::{
    parse_and_minimize_extended_formula, parse_and_minimize_hctl_formula, parse_extended_formula,
    parse_hctl_formula,
};
use pyo3::basic::CompareOp;
use pyo3::prelude::PyAnyMethods;
use pyo3::types::{PyDict, PyTuple};
use pyo3::{pyclass, pymethods, Bound, Py, PyAny, PyResult, Python};
use std::collections::HashSet;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;

/// Represents the syntax tree of a HCTL formula.
///
/// The string format for representing these formulas is available
/// on the repository site of the original [Rust library](https://github.com/sybila/biodivine-hctl-model-checker).
///
/// Note that this format uses `~` instead of `!` to represent negation (compared to other
/// expression formats used in AEON).
#[pyclass(module = "biodivine_aeon", frozen)]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HctlFormula {
    // The same memory management trick as we used for UpdateFunction and BooleanExpression.
    root: Arc<HctlTreeNode>,
    value: &'static HctlTreeNode,
}

impl HctlFormula {
    pub fn from_native(expression: HctlTreeNode) -> HctlFormula {
        let root = Arc::new(expression);
        Self::new_raw(root)
    }

    pub fn new_raw(root: Arc<HctlTreeNode>) -> HctlFormula {
        let root_ref = root.as_ref();
        let value: &'static HctlTreeNode =
            unsafe { (root_ref as *const HctlTreeNode).as_ref().unwrap() };
        HctlFormula { root, value }
    }

    pub fn mk_child_ref(&self, child: &HctlTreeNode) -> HctlFormula {
        let root = self.root.clone();
        let value: &'static HctlTreeNode =
            unsafe { (child as *const HctlTreeNode).as_ref().unwrap() };
        HctlFormula { root, value }
    }

    pub fn resolve_formula(
        value: &Bound<'_, PyAny>,
        allow_extended: bool,
        minimize_with: Option<&SymbolicContext>,
    ) -> PyResult<HctlFormula> {
        let formula_string = if let Ok(existing) = value.extract::<HctlFormula>() {
            existing.__str__()
        } else if let Ok(formula_string) = value.extract::<String>() {
            formula_string
        } else {
            return throw_type_error("Expected `String` or `HctlFormula`.");
        };

        let formula_native = match (allow_extended, minimize_with) {
            (false, None) => parse_hctl_formula(formula_string.as_str()),
            (true, None) => parse_extended_formula(formula_string.as_str()),
            (false, Some(ctx)) => {
                parse_and_minimize_hctl_formula(ctx.as_native(), formula_string.as_str())
            }
            (true, Some(ctx)) => {
                parse_and_minimize_extended_formula(ctx.as_native(), formula_string.as_str())
            }
        };

        match formula_native {
            Err(e) => throw_runtime_error(e),
            Ok(formula) => Ok(Self::from_native(formula)),
        }
    }

    pub fn as_native(&self) -> &HctlTreeNode {
        self.value
    }
}

fn encode_hybrid_operator(op: &HybridOp) -> String {
    match op {
        HybridOp::Exists => "exists",
        HybridOp::Forall => "forall",
        HybridOp::Bind => "bind",
        HybridOp::Jump => "jump",
    }
    .to_string()
}

fn encode_unary_operator(op: &UnaryOp) -> String {
    match op {
        UnaryOp::Not => "not",
        UnaryOp::EX => "exist_next",
        UnaryOp::AX => "all_next",
        UnaryOp::EF => "exist_future",
        UnaryOp::AF => "all_future",
        UnaryOp::EG => "exist_global",
        UnaryOp::AG => "all_global",
    }
    .to_string()
}

fn encode_binary_operator(op: &BinaryOp) -> String {
    match op {
        BinaryOp::And => "and",
        BinaryOp::Or => "or",
        BinaryOp::Imp => "imp",
        BinaryOp::Iff => "iff",
        BinaryOp::Xor => "xor",
        BinaryOp::EU => "exist_until",
        BinaryOp::AU => "all_until",
        BinaryOp::EW => "exist_weak_until",
        BinaryOp::AW => "all_weak_until",
    }
    .to_string()
}
fn resolve_hybrid_operator(op: String) -> PyResult<HybridOp> {
    match op.as_str() {
        "exists" => Ok(HybridOp::Exists),
        "forall" => Ok(HybridOp::Forall),
        "bind" => Ok(HybridOp::Bind),
        "jump" => Ok(HybridOp::Jump),
        _ => throw_type_error("Expected one of 'exists', 'forall', 'bind', and 'jump'."),
    }
}

fn resolve_temporal_unary_operator(op: String) -> PyResult<UnaryOp> {
    match op.as_str() {
        "exist_next" => Ok(UnaryOp::EX),
        "all_next" => Ok(UnaryOp::AX),
        "exist_future" => Ok(UnaryOp::EF),
        "all_future" => Ok(UnaryOp::AF),
        "exist_global" => Ok(UnaryOp::EG),
        "all_global" => Ok(UnaryOp::AG),
        _ => throw_type_error("Expected one of 'exist_next', 'all_next', 'exist_future', 'all_future', 'exist_global', 'all_global'.")
    }
}

fn resolve_temporal_binary_operator(op: String) -> PyResult<BinaryOp> {
    match op.as_str() {
        "exist_until" => Ok(BinaryOp::EU),
        "all_until" => Ok(BinaryOp::AU),
        "exist_weak_until" => Ok(BinaryOp::EW),
        "all_weak_until" => Ok(BinaryOp::AW),
        _ => throw_type_error(
            "Expected one of 'exist_until', 'all_until', 'exist_weak_until', 'all_weak_until'.",
        ),
    }
}

fn resolve_boolean_binary_operator(op: String) -> PyResult<BinaryOp> {
    match op.as_str() {
        "and" => Ok(BinaryOp::And),
        "or" => Ok(BinaryOp::Or),
        "iff" => Ok(BinaryOp::Iff),
        "imp" => Ok(BinaryOp::Imp),
        "xor" => Ok(BinaryOp::Xor),
        _ => throw_type_error("Expected one of 'and', 'or', 'iff', 'imp', 'xor'."),
    }
}

#[pymethods]
impl HctlFormula {
    /// Create a new `HctlFormula`, either parsing it from a string, or as a copy of an existing
    /// formula.
    ///
    /// If `allow_extended` is specified, the parser will also recognize "extended" propositions
    /// (denoted `%prop%`) that can be used to reference pre-computed symbolic sets.
    ///
    /// If `minimize_with` is specified with a `SymbolicContext`, the created formula is
    /// automatically converted to a canonical format, using standardized variable names and
    /// removing redundancies (for this, a `SymbolicContext` is required in order to check
    /// the mapping between propositions and network variables).
    #[new]
    #[pyo3(signature = (value, allow_extended = true, minimize_with = None))]
    pub fn new(
        value: &Bound<'_, PyAny>,
        allow_extended: bool,
        minimize_with: Option<&SymbolicContext>,
    ) -> PyResult<HctlFormula> {
        Self::resolve_formula(value, allow_extended, minimize_with)
    }

    fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.__str__().hash(&mut hasher);
        hasher.finish()
    }

    fn __richcmp__(&self, py: Python, other: &Self, op: CompareOp) -> Py<PyAny> {
        richcmp_eq_by_key(py, op, &self, &other, |it| it.as_native())
    }

    pub fn __str__(&self) -> String {
        self.as_native().to_string()
    }

    fn __repr__(&self) -> String {
        format!("HctlFormula({:?})", self.__str__())
    }

    fn __copy__(&self) -> HctlFormula {
        self.mk_child_ref(self.value)
    }

    fn __deepcopy__(&self, _memo: &Bound<'_, PyDict>) -> HctlFormula {
        HctlFormula::from_native(self.value.clone())
    }

    fn __getnewargs__<'a>(&self, py: Python<'a>) -> Bound<'a, PyTuple> {
        // Technically, this is a "different" expression because it is created with a completely new `root`,
        // but it is much easier (and more transparent) than serializing the root expression and trying to figure
        // out how to serialize a pointer into the AST.
        PyTuple::new_bound(py, [self.__str__()])
    }

    fn __root__(&self) -> HctlFormula {
        Self::new_raw(self.root.clone())
    }

    /// Create a new `HctlFormula` that uses a hybrid operator (see also `HybridOperator`).
    ///
    /// Optionally, you can provide a named `domain` which further restricts the validity
    /// of the operator. Note that such formulas have separate pattern matching methods
    /// (e.g. `HctlFormula.is_hybrid_in` instead of `HctlFormula.is_hybrid`).
    #[staticmethod]
    #[pyo3(signature = (op, state_variable, inner, domain = None))]
    fn mk_hybrid(
        op: String,
        state_variable: String,
        inner: &HctlFormula,
        domain: Option<String>,
    ) -> PyResult<HctlFormula> {
        let op = resolve_hybrid_operator(op)?;
        if op == HybridOp::Jump && domain.is_some() {
            return throw_type_error("Jump operator does not support domain restrictions.");
        }
        let formula_native = HctlTreeNode::mk_hybrid(
            inner.as_native().clone(),
            state_variable.as_str(),
            domain,
            op,
        );
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that uses the `3{x}` operator.
    ///
    /// Optionally, you can provide a named `domain` which further restricts the validity
    /// of the operator (i.e. it creates the `3{x} in %domain%` operator). Note that such
    /// formulas have separate pattern matching methods (e.g. `HctlFormula.is_exists_in`
    /// instead of `HctlFormula.is_exists`).
    #[staticmethod]
    #[pyo3(signature = (state_variable, inner, domain = None))]
    fn mk_exists(
        state_variable: String,
        inner: &HctlFormula,
        domain: Option<String>,
    ) -> PyResult<HctlFormula> {
        let formula_native = HctlTreeNode::mk_hybrid(
            inner.as_native().clone(),
            state_variable.as_str(),
            domain,
            HybridOp::Exists,
        );
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that uses the `V{x}` operator.
    ///
    /// Optionally, you can provide a named `domain` which further restricts the validity
    /// of the operator (i.e. it creates the `V{x} in %domain%` operator). Note that such
    /// formulas have separate pattern matching methods (e.g. `HctlFormula.is_forall_in`
    /// instead of `HctlFormula.is_forall`).
    #[staticmethod]
    #[pyo3(signature = (state_variable, inner, domain = None))]
    fn mk_forall(
        state_variable: String,
        inner: &HctlFormula,
        domain: Option<String>,
    ) -> PyResult<HctlFormula> {
        let formula_native = HctlTreeNode::mk_hybrid(
            inner.as_native().clone(),
            state_variable.as_str(),
            domain,
            HybridOp::Forall,
        );
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that uses the `!{x}` operator.
    ///
    /// Optionally, you can provide a named `domain` which further restricts the validity
    /// of the operator (i.e. it creates the `!{x} in %domain%` operator). Note that such
    /// formulas have separate pattern matching methods (e.g. `HctlFormula.is_forall_in`
    /// instead of `HctlFormula.is_forall`).
    #[staticmethod]
    #[pyo3(signature = (state_variable, inner, domain = None))]
    fn mk_bind(
        state_variable: String,
        inner: &HctlFormula,
        domain: Option<String>,
    ) -> PyResult<HctlFormula> {
        let formula_native = HctlTreeNode::mk_hybrid(
            inner.as_native().clone(),
            state_variable.as_str(),
            domain,
            HybridOp::Bind,
        );
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that uses the `@{x}` operator.
    #[staticmethod]
    fn mk_jump(state_variable: String, inner: &HctlFormula) -> PyResult<HctlFormula> {
        let formula_native = HctlTreeNode::mk_hybrid(
            inner.as_native().clone(),
            state_variable.as_str(),
            None,
            HybridOp::Jump,
        );
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that uses a temporal operator (see `TemporalUnaryOperator` and
    /// `TemporalBinaryOperator`).
    ///
    /// For unary operators, provide only one `HctlFormula`. For binary operators, provide
    /// both formulas.
    #[staticmethod]
    #[pyo3(signature = (op, a, b = None))]
    fn mk_temporal(op: String, a: HctlFormula, b: Option<HctlFormula>) -> PyResult<HctlFormula> {
        let native = if let Some(b) = b {
            let op = resolve_temporal_binary_operator(op)?;
            HctlTreeNode::mk_binary(a.as_native().clone(), b.as_native().clone(), op)
        } else {
            let op = resolve_temporal_unary_operator(op)?;
            HctlTreeNode::mk_unary(a.as_native().clone(), op)
        };
        Ok(Self::from_native(native))
    }

    /// Create a new `HctlFormula` that uses a binary Boolean operator (see `BinaryOperator`).
    #[staticmethod]
    fn mk_boolean(op: String, a: &HctlFormula, b: &HctlFormula) -> PyResult<HctlFormula> {
        let op = resolve_boolean_binary_operator(op)?;
        let native = HctlTreeNode::mk_binary(a.as_native().clone(), b.as_native().clone(), op);
        Ok(Self::from_native(native))
    }

    /// Create a new `HctlFormula` that uses the `~` operator.
    #[staticmethod]
    fn mk_not(inner: &HctlFormula) -> PyResult<HctlFormula> {
        let formula_native = HctlTreeNode::mk_unary(inner.as_native().clone(), UnaryOp::Not);
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that uses the `&` operator.
    #[staticmethod]
    fn mk_and(a: &HctlFormula, b: &HctlFormula) -> PyResult<HctlFormula> {
        let formula_native =
            HctlTreeNode::mk_binary(a.as_native().clone(), b.as_native().clone(), BinaryOp::And);
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that uses the `|` operator.
    #[staticmethod]
    fn mk_or(a: &HctlFormula, b: &HctlFormula) -> PyResult<HctlFormula> {
        let formula_native =
            HctlTreeNode::mk_binary(a.as_native().clone(), b.as_native().clone(), BinaryOp::Or);
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that uses the `=>` operator.
    #[staticmethod]
    fn mk_imp(a: &HctlFormula, b: &HctlFormula) -> PyResult<HctlFormula> {
        let formula_native =
            HctlTreeNode::mk_binary(a.as_native().clone(), b.as_native().clone(), BinaryOp::Imp);
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that uses the `<=>` operator.
    #[staticmethod]
    fn mk_iff(a: &HctlFormula, b: &HctlFormula) -> PyResult<HctlFormula> {
        let formula_native =
            HctlTreeNode::mk_binary(a.as_native().clone(), b.as_native().clone(), BinaryOp::Iff);
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that uses the `^` operator.
    #[staticmethod]
    fn mk_xor(a: &HctlFormula, b: &HctlFormula) -> PyResult<HctlFormula> {
        let formula_native =
            HctlTreeNode::mk_binary(a.as_native().clone(), b.as_native().clone(), BinaryOp::Xor);
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that uses the `EX` operator.
    #[staticmethod]
    fn mk_exist_next(inner: &HctlFormula) -> PyResult<HctlFormula> {
        let formula_native = HctlTreeNode::mk_unary(inner.as_native().clone(), UnaryOp::EX);
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that uses the `AX` operator.
    #[staticmethod]
    fn mk_all_next(inner: &HctlFormula) -> PyResult<HctlFormula> {
        let formula_native = HctlTreeNode::mk_unary(inner.as_native().clone(), UnaryOp::AX);
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that uses the `EF` operator.
    #[staticmethod]
    fn mk_exist_future(inner: &HctlFormula) -> PyResult<HctlFormula> {
        let formula_native = HctlTreeNode::mk_unary(inner.as_native().clone(), UnaryOp::EF);
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that uses the `AF` operator.
    #[staticmethod]
    fn mk_all_future(inner: &HctlFormula) -> PyResult<HctlFormula> {
        let formula_native = HctlTreeNode::mk_unary(inner.as_native().clone(), UnaryOp::AF);
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that uses the `EG` operator.
    #[staticmethod]
    fn mk_exist_global(inner: &HctlFormula) -> PyResult<HctlFormula> {
        let formula_native = HctlTreeNode::mk_unary(inner.as_native().clone(), UnaryOp::EG);
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that uses the `AG` operator.
    #[staticmethod]
    fn mk_all_global(inner: &HctlFormula) -> PyResult<HctlFormula> {
        let formula_native = HctlTreeNode::mk_unary(inner.as_native().clone(), UnaryOp::AG);
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that uses the `EU` operator.
    #[staticmethod]
    fn mk_exist_until(a: &HctlFormula, b: &HctlFormula) -> PyResult<HctlFormula> {
        let formula_native =
            HctlTreeNode::mk_binary(a.as_native().clone(), b.as_native().clone(), BinaryOp::EU);
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that uses the `AU` operator.
    #[staticmethod]
    fn mk_all_until(a: &HctlFormula, b: &HctlFormula) -> PyResult<HctlFormula> {
        let formula_native =
            HctlTreeNode::mk_binary(a.as_native().clone(), b.as_native().clone(), BinaryOp::AU);
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that uses the `EW` operator.
    #[staticmethod]
    fn mk_exist_weak_until(a: &HctlFormula, b: &HctlFormula) -> PyResult<HctlFormula> {
        let formula_native =
            HctlTreeNode::mk_binary(a.as_native().clone(), b.as_native().clone(), BinaryOp::EW);
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that uses the `AW` operator.
    #[staticmethod]
    fn mk_all_weak_until(a: &HctlFormula, b: &HctlFormula) -> PyResult<HctlFormula> {
        let formula_native =
            HctlTreeNode::mk_binary(a.as_native().clone(), b.as_native().clone(), BinaryOp::AW);
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that represents a quantified state variable.
    #[staticmethod]
    fn mk_state_var(name: String) -> PyResult<HctlFormula> {
        let formula_native = HctlTreeNode::mk_variable(name.as_str());
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that represents a network variable proposition.
    #[staticmethod]
    fn mk_network_var(name: String) -> PyResult<HctlFormula> {
        let formula_native = HctlTreeNode::mk_proposition(name.as_str());
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that represents a Boolean constant.
    #[staticmethod]
    fn mk_const(value: bool) -> PyResult<HctlFormula> {
        let formula_native = HctlTreeNode::mk_constant(value);
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that represents an extended proposition.
    #[staticmethod]
    fn mk_extended_prop(name: String) -> PyResult<HctlFormula> {
        let formula_native = HctlTreeNode::mk_wild_card(name.as_str());
        Ok(Self::from_native(formula_native))
    }

    /// Check if this `HctlFormula` represents on of the hybrid operators *without* a domain
    /// restriction (see `HybridOperator`).
    fn is_hybrid(&self) -> bool {
        matches!(self.value.node_type, NodeType::Hybrid(_, _, None, _))
    }

    /// Check if this `HctlFormula` represents on of the hybrid operators *with* a domain
    /// restriction (see `HybridOperator`).
    fn is_hybrid_in(&self) -> bool {
        matches!(self.value.node_type, NodeType::Hybrid(_, _, Some(_), _))
    }

    /// Check if this `HctlFormula` represents on of the temporal operators (see `TemporalUnaryOperator` and `TemporalBinaryOperator`).
    fn is_temporal(&self) -> bool {
        self.is_temporal_unary() || self.is_temporal_binary()
    }

    /// Check if this `HctlFormula` represents on of the unary temporal operators (see `TemporalUnaryOperator`).
    fn is_temporal_unary(&self) -> bool {
        matches!(
            self.value.node_type,
            NodeType::Unary(
                UnaryOp::EX | UnaryOp::AX | UnaryOp::EF | UnaryOp::AF | UnaryOp::EG | UnaryOp::AG,
                _
            )
        )
    }

    /// Check if this `HctlFormula` represents on of the binary temporal operators (see `TemporalBinaryOperator`).
    fn is_temporal_binary(&self) -> bool {
        matches!(
            self.value.node_type,
            NodeType::Binary(
                BinaryOp::EU | BinaryOp::AU | BinaryOp::EW | BinaryOp::AW,
                _,
                _
            )
        )
    }

    /// Check if this `HctlFormula` represents on of the binary Boolean operators (see `BinaryOperator`).
    fn is_boolean(&self) -> bool {
        matches!(
            self.value.node_type,
            NodeType::Binary(
                BinaryOp::And | BinaryOp::Or | BinaryOp::Imp | BinaryOp::Iff | BinaryOp::Xor,
                _,
                _
            )
        )
    }

    /// Check if this `HctlFormula` represents a quantified state proposition.
    fn is_state_var(&self) -> bool {
        matches!(self.value.node_type, NodeType::Terminal(Atomic::Var(_)))
    }

    /// Check if this `HctlFormula` represents a network variable proposition.
    fn is_network_var(&self) -> bool {
        matches!(self.value.node_type, NodeType::Terminal(Atomic::Prop(_)))
    }

    /// Check if this `HctlFormula` represents a constant.
    fn is_const(&self) -> bool {
        matches!(
            self.value.node_type,
            NodeType::Terminal(Atomic::True | Atomic::False)
        )
    }

    /// Check if this `HctlFormula` represents an extended proposition.
    fn is_extended_prop(&self) -> bool {
        matches!(
            self.value.node_type,
            NodeType::Terminal(Atomic::WildCardProp(_))
        )
    }

    /// Check if this `HctlFormula` represents the `3{x}` operator.
    fn is_exists(&self) -> bool {
        matches!(
            self.value.node_type,
            NodeType::Hybrid(HybridOp::Exists, _, None, _)
        )
    }

    /// Check if this `HctlFormula` represents the `3{x} in %domain%` operator.
    fn is_exists_in(&self) -> bool {
        matches!(
            self.value.node_type,
            NodeType::Hybrid(HybridOp::Exists, _, Some(_), _)
        )
    }

    /// Check if this `HctlFormula` represents the `V{x}` operator.
    fn is_forall(&self) -> bool {
        matches!(
            self.value.node_type,
            NodeType::Hybrid(HybridOp::Forall, _, None, _)
        )
    }

    /// Check if this `HctlFormula` represents the `V{x} in %domain%` operator.
    fn is_forall_in(&self) -> bool {
        matches!(
            self.value.node_type,
            NodeType::Hybrid(HybridOp::Forall, _, Some(_), _)
        )
    }

    /// Check if this `HctlFormula` represents the `!{x}` operator.
    fn is_bind(&self) -> bool {
        matches!(
            self.value.node_type,
            NodeType::Hybrid(HybridOp::Bind, _, None, _)
        )
    }

    /// Check if this `HctlFormula` represents the `!{x} in %domain%` operator.
    fn is_bind_in(&self) -> bool {
        matches!(
            self.value.node_type,
            NodeType::Hybrid(HybridOp::Bind, _, Some(_), _)
        )
    }

    /// Check if this `HctlFormula` represents the `@{x}` operator.
    fn is_jump(&self) -> bool {
        matches!(
            self.value.node_type,
            NodeType::Hybrid(HybridOp::Jump, _, _, _)
        )
    }

    /// Check if this `HctlFormula` represents the `~` operator.
    fn is_not(&self) -> bool {
        matches!(self.value.node_type, NodeType::Unary(UnaryOp::Not, _))
    }

    /// Check if this `HctlFormula` represents the `&` operator.
    fn is_and(&self) -> bool {
        matches!(self.value.node_type, NodeType::Binary(BinaryOp::And, _, _))
    }

    /// Check if this `HctlFormula` represents the `|` operator.
    fn is_or(&self) -> bool {
        matches!(self.value.node_type, NodeType::Binary(BinaryOp::Or, _, _))
    }

    /// Check if this `HctlFormula` represents the `=>` operator.
    fn is_imp(&self) -> bool {
        matches!(self.value.node_type, NodeType::Binary(BinaryOp::Imp, _, _))
    }

    /// Check if this `HctlFormula` represents the `<=>` operator.
    fn is_iff(&self) -> bool {
        matches!(self.value.node_type, NodeType::Binary(BinaryOp::Iff, _, _))
    }

    /// Check if this `HctlFormula` represents the `^ (xor)` operator.
    fn is_xor(&self) -> bool {
        matches!(self.value.node_type, NodeType::Binary(BinaryOp::Xor, _, _))
    }

    /// Check if this `HctlFormula` represents the `EX` operator.
    fn is_exist_next(&self) -> bool {
        matches!(self.value.node_type, NodeType::Unary(UnaryOp::EX, _))
    }

    /// Check if this `HctlFormula` represents the `AX` operator.
    fn is_all_next(&self) -> bool {
        matches!(self.value.node_type, NodeType::Unary(UnaryOp::AX, _))
    }

    /// Check if this `HctlFormula` represents the `EF` operator.
    fn is_exist_future(&self) -> bool {
        matches!(self.value.node_type, NodeType::Unary(UnaryOp::EF, _))
    }

    /// Check if this `HctlFormula` represents the `AF` operator.
    fn is_all_future(&self) -> bool {
        matches!(self.value.node_type, NodeType::Unary(UnaryOp::AF, _))
    }

    /// Check if this `HctlFormula` represents the `EG` operator.
    fn is_exist_global(&self) -> bool {
        matches!(self.value.node_type, NodeType::Unary(UnaryOp::EG, _))
    }

    /// Check if this `HctlFormula` represents the `AG` operator.
    fn is_all_global(&self) -> bool {
        matches!(self.value.node_type, NodeType::Unary(UnaryOp::AG, _))
    }

    /// Check if this `HctlFormula` represents the `EU` operator.
    fn is_exist_until(&self) -> bool {
        matches!(self.value.node_type, NodeType::Binary(BinaryOp::EU, _, _))
    }

    /// Check if this `HctlFormula` represents the `AU` operator.
    fn is_all_until(&self) -> bool {
        matches!(self.value.node_type, NodeType::Binary(BinaryOp::AU, _, _))
    }

    /// Check if this `HctlFormula` represents the `EW` operator.
    fn is_exist_weak_until(&self) -> bool {
        matches!(self.value.node_type, NodeType::Binary(BinaryOp::EW, _, _))
    }

    /// Check if this `HctlFormula` represents the `AW` operator.
    fn is_all_weak_until(&self) -> bool {
        matches!(self.value.node_type, NodeType::Binary(BinaryOp::AW, _, _))
    }

    /// Return the operator, variable and argument if this `HctlFormula` represents a
    /// hybrid operator *without domain restriction*.
    fn as_hybrid(&self) -> Option<(String, String, HctlFormula)> {
        match &self.value.node_type {
            NodeType::Hybrid(op, var, None, inner) => Some((
                encode_hybrid_operator(op),
                var.clone(),
                self.mk_child_ref(inner),
            )),
            _ => None,
        }
    }

    /// Return the operator, variable, *domain* and argument if this `HctlFormula` represents a
    /// hybrid operator *with domain restriction*.
    fn as_hybrid_in(&self) -> Option<(String, String, String, HctlFormula)> {
        match &self.value.node_type {
            NodeType::Hybrid(op, var, Some(domain), inner) => Some((
                encode_hybrid_operator(op),
                var.clone(),
                domain.clone(),
                self.mk_child_ref(inner),
            )),
            _ => None,
        }
    }

    /// Return the operator and argument if this `HctlFormula` represents a unary temporal operator.
    fn as_temporal_unary(&self) -> Option<(String, HctlFormula)> {
        match &self.value.node_type {
            NodeType::Unary(op, inner) => match op {
                UnaryOp::EX
                | UnaryOp::AX
                | UnaryOp::EF
                | UnaryOp::AF
                | UnaryOp::EG
                | UnaryOp::AG => Some((encode_unary_operator(op), self.mk_child_ref(inner))),
                _ => None,
            },
            _ => None,
        }
    }

    /// Return the operator and arguments if this `HctlFormula` represents a binary temporal operator.
    fn as_temporal_binary(&self) -> Option<(String, HctlFormula, HctlFormula)> {
        match &self.value.node_type {
            NodeType::Binary(op, a, b) => match op {
                BinaryOp::EU | BinaryOp::AU | BinaryOp::EW | BinaryOp::AW => Some((
                    encode_binary_operator(op),
                    self.mk_child_ref(a),
                    self.mk_child_ref(b),
                )),
                _ => None,
            },
            _ => None,
        }
    }

    /// Return the operator and arguments if this `HctlFormula` represents a binary Boolean operator.
    fn as_boolean(&self) -> Option<(String, HctlFormula, HctlFormula)> {
        match &self.value.node_type {
            NodeType::Binary(op, a, b) => match op {
                BinaryOp::And | BinaryOp::Or | BinaryOp::Imp | BinaryOp::Iff | BinaryOp::Xor => {
                    Some((
                        encode_binary_operator(op),
                        self.mk_child_ref(a),
                        self.mk_child_ref(b),
                    ))
                }
                _ => None,
            },
            _ => None,
        }
    }

    /// Return the variable name if this `HctlFormula` represents a state variable proposition.
    fn as_state_var(&self) -> Option<String> {
        match &self.value.node_type {
            NodeType::Terminal(Atomic::Var(name)) => Some(name.clone()),
            _ => None,
        }
    }

    /// Return the variable name if this `HctlFormula` represents a network variable proposition.
    fn as_network_var(&self) -> Option<String> {
        match &self.value.node_type {
            NodeType::Terminal(Atomic::Prop(name)) => Some(name.clone()),
            _ => None,
        }
    }

    /// Return the Boolean value if this `HctlFormula` represents a constant.
    fn as_const(&self) -> Option<bool> {
        match &self.value.node_type {
            NodeType::Terminal(Atomic::True) => Some(true),
            NodeType::Terminal(Atomic::False) => Some(false),
            _ => None,
        }
    }

    /// Return the property name if this `HctlFormula` represents an extended proposition (`%name%`).
    fn as_extended_prop(&self) -> Option<String> {
        match &self.value.node_type {
            NodeType::Terminal(Atomic::WildCardProp(name)) => Some(name.clone()),
            _ => None,
        }
    }

    /// Return the state variable name and the child formula if this `HctlFormula` represents
    /// the `3{x}` hybrid operator.
    ///
    /// (This method returns `None` if the formula represents the `3{x} in %domain%` operator.)
    fn as_exists(&self) -> Option<(String, HctlFormula)> {
        match &self.value.node_type {
            NodeType::Hybrid(HybridOp::Exists, name, None, inner) => {
                Some((name.clone(), self.mk_child_ref(inner)))
            }
            _ => None,
        }
    }

    /// Return the state variable name, *domain name*, and the child formula if this
    /// `HctlFormula` represents the `3{x} in %domain%` hybrid operator.
    fn as_exists_in(&self) -> Option<(String, String, HctlFormula)> {
        match &self.value.node_type {
            NodeType::Hybrid(HybridOp::Exists, name, Some(domain), inner) => {
                Some((name.clone(), domain.clone(), self.mk_child_ref(inner)))
            }
            _ => None,
        }
    }

    /// Return the state variable name and the child formula if this `HctlFormula` represents
    /// the `V{x}` hybrid operator.
    ///
    /// (This method returns `None` if the formula represents the `V{x} in %domain%` operator.)
    fn as_forall(&self) -> Option<(String, HctlFormula)> {
        match &self.value.node_type {
            NodeType::Hybrid(HybridOp::Forall, name, None, inner) => {
                Some((name.clone(), self.mk_child_ref(inner)))
            }
            _ => None,
        }
    }

    /// Return the state variable name, *domain name*, and the child formula if this
    /// `HctlFormula` represents the `V{x} in %domain%` hybrid operator.
    fn as_forall_in(&self) -> Option<(String, String, HctlFormula)> {
        match &self.value.node_type {
            NodeType::Hybrid(HybridOp::Forall, name, Some(domain), inner) => {
                Some((name.clone(), domain.clone(), self.mk_child_ref(inner)))
            }
            _ => None,
        }
    }

    /// Return the state variable name and the child formula if this `HctlFormula` represents
    /// the `!{x}` hybrid operator.
    ///
    /// (This method returns `None` if the formula represents the `!{x} in %domain%` operator.)
    fn as_bind(&self) -> Option<(String, HctlFormula)> {
        match &self.value.node_type {
            NodeType::Hybrid(HybridOp::Bind, name, None, inner) => {
                Some((name.clone(), self.mk_child_ref(inner)))
            }
            _ => None,
        }
    }

    /// Return the state variable name, *domain name*, and the child formula if this
    /// `HctlFormula` represents the `!{x} in %domain%` hybrid operator.
    fn as_bind_in(&self) -> Option<(String, String, HctlFormula)> {
        match &self.value.node_type {
            NodeType::Hybrid(HybridOp::Bind, name, Some(domain), inner) => {
                Some((name.clone(), domain.clone(), self.mk_child_ref(inner)))
            }
            _ => None,
        }
    }

    /// Return the state variable name and the child formula if this `HctlFormula` represents
    /// the `@{x}` hybrid operator.
    fn as_jump(&self) -> Option<(String, HctlFormula)> {
        match &self.value.node_type {
            NodeType::Hybrid(HybridOp::Jump, name, None, inner) => {
                Some((name.clone(), self.mk_child_ref(inner)))
            }
            _ => None,
        }
    }

    /// Return the child formula if this `HctlFormula` represents the `~` operator.
    fn as_not(&self) -> Option<HctlFormula> {
        match &self.value.node_type {
            NodeType::Unary(UnaryOp::Not, inner) => Some(self.mk_child_ref(inner)),
            _ => None,
        }
    }

    /// Return the two child formulas if this `HctlFormula` represents the `&` Boolean operator.
    fn as_and(&self) -> Option<(HctlFormula, HctlFormula)> {
        match &self.value.node_type {
            NodeType::Binary(BinaryOp::And, a, b) => {
                Some((self.mk_child_ref(a), self.mk_child_ref(b)))
            }
            _ => None,
        }
    }

    /// Return the two child formulas if this `HctlFormula` represents the `|` Boolean operator.
    fn as_or(&self) -> Option<(HctlFormula, HctlFormula)> {
        match &self.value.node_type {
            NodeType::Binary(BinaryOp::Or, a, b) => {
                Some((self.mk_child_ref(a), self.mk_child_ref(b)))
            }
            _ => None,
        }
    }

    /// Return the two child formulas if this `HctlFormula` represents the `=>` Boolean operator.
    fn as_imp(&self) -> Option<(HctlFormula, HctlFormula)> {
        match &self.value.node_type {
            NodeType::Binary(BinaryOp::Imp, a, b) => {
                Some((self.mk_child_ref(a), self.mk_child_ref(b)))
            }
            _ => None,
        }
    }

    /// Return the two child formulas if this `HctlFormula` represents the `<=>` Boolean operator.
    fn as_iff(&self) -> Option<(HctlFormula, HctlFormula)> {
        match &self.value.node_type {
            NodeType::Binary(BinaryOp::Iff, a, b) => {
                Some((self.mk_child_ref(a), self.mk_child_ref(b)))
            }
            _ => None,
        }
    }

    /// Return the two child formulas if this `HctlFormula` represents the `^` Boolean operator.
    fn as_xor(&self) -> Option<(HctlFormula, HctlFormula)> {
        match &self.value.node_type {
            NodeType::Binary(BinaryOp::Xor, a, b) => {
                Some((self.mk_child_ref(a), self.mk_child_ref(b)))
            }
            _ => None,
        }
    }

    /// Return the child formula if this `HctlFormula` represents the `EX` temporal operator.
    fn as_exist_next(&self) -> Option<HctlFormula> {
        match &self.value.node_type {
            NodeType::Unary(UnaryOp::EX, inner) => Some(self.mk_child_ref(inner)),
            _ => None,
        }
    }

    /// Return the child formula if this `HctlFormula` represents the `AX` temporal operator.
    fn as_all_next(&self) -> Option<HctlFormula> {
        match &self.value.node_type {
            NodeType::Unary(UnaryOp::AX, inner) => Some(self.mk_child_ref(inner)),
            _ => None,
        }
    }

    /// Return the child formula if this `HctlFormula` represents the `EF` temporal operator.
    fn as_exist_future(&self) -> Option<HctlFormula> {
        match &self.value.node_type {
            NodeType::Unary(UnaryOp::EF, inner) => Some(self.mk_child_ref(inner)),
            _ => None,
        }
    }

    /// Return the child formula if this `HctlFormula` represents the `AF` temporal operator.
    fn as_all_future(&self) -> Option<HctlFormula> {
        match &self.value.node_type {
            NodeType::Unary(UnaryOp::AF, inner) => Some(self.mk_child_ref(inner)),
            _ => None,
        }
    }

    /// Return the child formula if this `HctlFormula` represents the `EG` temporal operator.
    fn as_exist_global(&self) -> Option<HctlFormula> {
        match &self.value.node_type {
            NodeType::Unary(UnaryOp::EG, inner) => Some(self.mk_child_ref(inner)),
            _ => None,
        }
    }

    /// Return the child formula if this `HctlFormula` represents the `AG` temporal operator.
    fn as_all_global(&self) -> Option<HctlFormula> {
        match &self.value.node_type {
            NodeType::Unary(UnaryOp::AG, inner) => Some(self.mk_child_ref(inner)),
            _ => None,
        }
    }

    /// Return the two child formulas if this `HctlFormula` represents the `EU` temporal operator.
    fn as_exist_until(&self) -> Option<(HctlFormula, HctlFormula)> {
        match &self.value.node_type {
            NodeType::Binary(BinaryOp::EU, a, b) => {
                Some((self.mk_child_ref(a), self.mk_child_ref(b)))
            }
            _ => None,
        }
    }

    /// Return the two child formulas if this `HctlFormula` represents the `AU` temporal operator.
    fn as_all_until(&self) -> Option<(HctlFormula, HctlFormula)> {
        match &self.value.node_type {
            NodeType::Binary(BinaryOp::AU, a, b) => {
                Some((self.mk_child_ref(a), self.mk_child_ref(b)))
            }
            _ => None,
        }
    }

    /// Return the two child formulas if this `HctlFormula` represents the `EW` temporal operator.
    fn as_exist_weak_until(&self) -> Option<(HctlFormula, HctlFormula)> {
        match &self.value.node_type {
            NodeType::Binary(BinaryOp::EW, a, b) => {
                Some((self.mk_child_ref(a), self.mk_child_ref(b)))
            }
            _ => None,
        }
    }

    /// Return the two child formulas if this `HctlFormula` represents the `AW` temporal operator.
    fn as_all_weak_until(&self) -> Option<(HctlFormula, HctlFormula)> {
        match &self.value.node_type {
            NodeType::Binary(BinaryOp::AW, a, b) => {
                Some((self.mk_child_ref(a), self.mk_child_ref(b)))
            }
            _ => None,
        }
    }

    /// Returns `True` if the provided `AsynchronousGraph` has enough extra symbolic variables
    /// such that it can be used to model-check this `HctlFormula`.
    fn is_compatible_with(&self, context: &AsynchronousGraph) -> bool {
        check_hctl_var_support(context.as_native(), self.as_native().clone())
    }

    /// Returns the set of HCTL state variables that are used in this formula.
    pub fn used_state_variables(&self) -> HashSet<String> {
        collect_unique_hctl_vars(self.as_native().clone())
    }

    /// Returns the set of extended property names and domain names that are used in this formula.
    pub fn used_extended_properties(&self) -> HashSet<String> {
        collect_unique_wild_cards(self.as_native().clone()).0
    }

    /// Return the direct child sub-formulas of this `HctlFormula` (one child for unary and hybrid
    /// operators, two children for binary operators, no children for atoms).
    ///
    /// For binary operators, the left-most child is returned first.
    pub fn children(&self) -> Vec<HctlFormula> {
        match &self.value.node_type {
            NodeType::Terminal(_) => vec![],
            NodeType::Unary(_, child) => vec![self.mk_child_ref(child)],
            NodeType::Hybrid(_, _, _, child) => vec![self.mk_child_ref(child)],
            NodeType::Binary(_, a, b) => vec![self.mk_child_ref(a), self.mk_child_ref(b)],
        }
    }

    /// Return the string representation of the operator used by this `HctlFormula`, or `None`
    /// if this is an atom.
    pub fn operator(&self) -> Option<String> {
        match &self.value.node_type {
            NodeType::Terminal(_) => None,
            NodeType::Unary(op, _) => Some(encode_unary_operator(op)),
            NodeType::Hybrid(op, _, _, _) => Some(encode_hybrid_operator(op)),
            NodeType::Binary(op, _, _) => Some(encode_binary_operator(op)),
        }
    }
}
