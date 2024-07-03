use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;

use biodivine_hctl_model_checker::preprocessing::node::{HctlTreeNode, NodeType};
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

use crate::bindings::lib_param_bn::symbolic::symbolic_context::SymbolicContext;
use crate::pyo3_utils::richcmp_eq_by_key;
use crate::{throw_runtime_error, throw_type_error, AsNative};

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
        UnaryOp::Ex => "exist_next",
        UnaryOp::Ax => "all_next",
        UnaryOp::Ef => "exist_future",
        UnaryOp::Af => "all_future",
        UnaryOp::Eg => "exist_global",
        UnaryOp::Ag => "all_global",
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
        BinaryOp::Eu => "exist_until",
        BinaryOp::Au => "all_until",
        BinaryOp::Ew => "exist_weak_until",
        BinaryOp::Aw => "all_weak_until",
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
        "exist_next" => Ok(UnaryOp::Ex),
        "all_next" => Ok(UnaryOp::Ex),
        "exist_future" => Ok(UnaryOp::Ef),
        "all_future" => Ok(UnaryOp::Af),
        "exist_global" => Ok(UnaryOp::Eg),
        "all_global" => Ok(UnaryOp::Ag),
        _ => throw_type_error("Expected one of 'exist_next', 'all_next', 'exist_future', 'all_future', 'exist_global', 'all_global'.")
    }
}

fn resolve_temporal_binary_operator(op: String) -> PyResult<BinaryOp> {
    match op.as_str() {
        "exist_until" => Ok(BinaryOp::Eu),
        "all_until" => Ok(BinaryOp::Au),
        "exist_weak_until" => Ok(BinaryOp::Ew),
        "all_weak_until" => Ok(BinaryOp::Aw),
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
    /// the mapping between hybrid state variables and their symbolic representation).
    #[new]
    #[pyo3(signature = (value, allow_extended = true, minimize_with = None))]
    fn new(
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
    #[staticmethod]
    fn mk_hybrid(op: String, state_variable: String, inner: &HctlFormula) -> PyResult<HctlFormula> {
        let op = resolve_hybrid_operator(op)?;
        let formula_native =
            HctlTreeNode::mk_hybrid_node(inner.as_native().clone(), state_variable, op);
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that uses the `3{x}` operator.
    #[staticmethod]
    fn mk_exists(state_variable: String, inner: &HctlFormula) -> PyResult<HctlFormula> {
        let formula_native = HctlTreeNode::mk_hybrid_node(
            inner.as_native().clone(),
            state_variable,
            HybridOp::Exists,
        );
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that uses the `V{x}` operator.
    #[staticmethod]
    fn mk_forall(state_variable: String, inner: &HctlFormula) -> PyResult<HctlFormula> {
        let formula_native = HctlTreeNode::mk_hybrid_node(
            inner.as_native().clone(),
            state_variable,
            HybridOp::Forall,
        );
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that uses the `!{x}` operator.
    #[staticmethod]
    fn mk_bind(state_variable: String, inner: &HctlFormula) -> PyResult<HctlFormula> {
        let formula_native =
            HctlTreeNode::mk_hybrid_node(inner.as_native().clone(), state_variable, HybridOp::Bind);
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that uses the `@{x}` operator.
    #[staticmethod]
    fn mk_jump(state_variable: String, inner: &HctlFormula) -> PyResult<HctlFormula> {
        let formula_native =
            HctlTreeNode::mk_hybrid_node(inner.as_native().clone(), state_variable, HybridOp::Jump);
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
            HctlTreeNode::mk_binary_node(a.as_native().clone(), b.as_native().clone(), op)
        } else {
            let op = resolve_temporal_unary_operator(op)?;
            HctlTreeNode::mk_unary_node(a.as_native().clone(), op)
        };
        Ok(Self::from_native(native))
    }

    /// Create a new `HctlFormula` that uses a binary Boolean operator (see `BinaryOperator`).
    #[staticmethod]
    fn mk_boolean(op: String, a: &HctlFormula, b: &HctlFormula) -> PyResult<HctlFormula> {
        let op = resolve_boolean_binary_operator(op)?;
        let native = HctlTreeNode::mk_binary_node(a.as_native().clone(), b.as_native().clone(), op);
        Ok(Self::from_native(native))
    }

    /// Create a new `HctlFormula` that uses the `~` operator.
    #[staticmethod]
    fn mk_not(inner: &HctlFormula) -> PyResult<HctlFormula> {
        let formula_native = HctlTreeNode::mk_unary_node(inner.as_native().clone(), UnaryOp::Not);
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that uses the `&` operator.
    #[staticmethod]
    fn mk_and(a: &HctlFormula, b: &HctlFormula) -> PyResult<HctlFormula> {
        let formula_native = HctlTreeNode::mk_binary_node(
            a.as_native().clone(),
            b.as_native().clone(),
            BinaryOp::And,
        );
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that uses the `|` operator.
    #[staticmethod]
    fn mk_or(a: &HctlFormula, b: &HctlFormula) -> PyResult<HctlFormula> {
        let formula_native = HctlTreeNode::mk_binary_node(
            a.as_native().clone(),
            b.as_native().clone(),
            BinaryOp::Or,
        );
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that uses the `=>` operator.
    #[staticmethod]
    fn mk_imp(a: &HctlFormula, b: &HctlFormula) -> PyResult<HctlFormula> {
        let formula_native = HctlTreeNode::mk_binary_node(
            a.as_native().clone(),
            b.as_native().clone(),
            BinaryOp::Imp,
        );
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that uses the `<=>` operator.
    #[staticmethod]
    fn mk_iff(a: &HctlFormula, b: &HctlFormula) -> PyResult<HctlFormula> {
        let formula_native = HctlTreeNode::mk_binary_node(
            a.as_native().clone(),
            b.as_native().clone(),
            BinaryOp::Iff,
        );
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that uses the `^` operator.
    #[staticmethod]
    fn mk_xor(a: &HctlFormula, b: &HctlFormula) -> PyResult<HctlFormula> {
        let formula_native = HctlTreeNode::mk_binary_node(
            a.as_native().clone(),
            b.as_native().clone(),
            BinaryOp::Xor,
        );
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that uses the `EX` operator.
    #[staticmethod]
    fn mk_exist_next(inner: &HctlFormula) -> PyResult<HctlFormula> {
        let formula_native = HctlTreeNode::mk_unary_node(inner.as_native().clone(), UnaryOp::Ex);
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that uses the `AX` operator.
    #[staticmethod]
    fn mk_all_next(inner: &HctlFormula) -> PyResult<HctlFormula> {
        let formula_native = HctlTreeNode::mk_unary_node(inner.as_native().clone(), UnaryOp::Ax);
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that uses the `EF` operator.
    #[staticmethod]
    fn mk_exist_future(inner: &HctlFormula) -> PyResult<HctlFormula> {
        let formula_native = HctlTreeNode::mk_unary_node(inner.as_native().clone(), UnaryOp::Ef);
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that uses the `AF` operator.
    #[staticmethod]
    fn mk_all_future(inner: &HctlFormula) -> PyResult<HctlFormula> {
        let formula_native = HctlTreeNode::mk_unary_node(inner.as_native().clone(), UnaryOp::Af);
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that uses the `EG` operator.
    #[staticmethod]
    fn mk_exist_global(inner: &HctlFormula) -> PyResult<HctlFormula> {
        let formula_native = HctlTreeNode::mk_unary_node(inner.as_native().clone(), UnaryOp::Eg);
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that uses the `AG` operator.
    #[staticmethod]
    fn mk_all_global(inner: &HctlFormula) -> PyResult<HctlFormula> {
        let formula_native = HctlTreeNode::mk_unary_node(inner.as_native().clone(), UnaryOp::Ag);
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that uses the `EU` operator.
    #[staticmethod]
    fn mk_exist_until(a: &HctlFormula, b: &HctlFormula) -> PyResult<HctlFormula> {
        let formula_native = HctlTreeNode::mk_binary_node(
            a.as_native().clone(),
            b.as_native().clone(),
            BinaryOp::Eu,
        );
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that uses the `AU` operator.
    #[staticmethod]
    fn mk_all_until(a: &HctlFormula, b: &HctlFormula) -> PyResult<HctlFormula> {
        let formula_native = HctlTreeNode::mk_binary_node(
            a.as_native().clone(),
            b.as_native().clone(),
            BinaryOp::Au,
        );
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that uses the `EW` operator.
    #[staticmethod]
    fn mk_exist_weak_until(a: &HctlFormula, b: &HctlFormula) -> PyResult<HctlFormula> {
        let formula_native = HctlTreeNode::mk_binary_node(
            a.as_native().clone(),
            b.as_native().clone(),
            BinaryOp::Ew,
        );
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that uses the `AW` operator.
    #[staticmethod]
    fn mk_all_weak_until(a: &HctlFormula, b: &HctlFormula) -> PyResult<HctlFormula> {
        let formula_native = HctlTreeNode::mk_binary_node(
            a.as_native().clone(),
            b.as_native().clone(),
            BinaryOp::Aw,
        );
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that represents a quantified state variable.
    #[staticmethod]
    fn mk_state_var(name: String) -> PyResult<HctlFormula> {
        let formula_native = HctlTreeNode::mk_var_node(name);
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that represents a network variable proposition.
    #[staticmethod]
    fn mk_network_var(name: String) -> PyResult<HctlFormula> {
        let formula_native = HctlTreeNode::mk_prop_node(name);
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that represents a Boolean constant.
    #[staticmethod]
    fn mk_const(value: bool) -> PyResult<HctlFormula> {
        let formula_native = HctlTreeNode::mk_constant_node(value);
        Ok(Self::from_native(formula_native))
    }

    /// Create a new `HctlFormula` that represents an extended proposition.
    #[staticmethod]
    fn mk_extended_prop(name: String) -> PyResult<HctlFormula> {
        let formula_native = HctlTreeNode::mk_wild_card_node(name);
        Ok(Self::from_native(formula_native))
    }

    /// Check if this `HctlFormula` represents on of the hybrid operators (see `HybridOperator`).
    fn is_hybrid(&self) -> bool {
        matches!(self.value.node_type, NodeType::HybridNode(_, _, _))
    }

    /// Check if this `HctlFormula` represents on of the temporal operators (see `TemporalUnaryOperator` and `TemporalBinaryOperator`).
    fn is_temporal(&self) -> bool {
        self.is_temporal_unary() || self.is_temporal_binary()
    }

    /// Check if this `HctlFormula` represents on of the unary temporal operators (see `TemporalUnaryOperator`).
    fn is_temporal_unary(&self) -> bool {
        matches!(
            self.value.node_type,
            NodeType::UnaryNode(
                UnaryOp::Ex | UnaryOp::Ax | UnaryOp::Ef | UnaryOp::Af | UnaryOp::Eg | UnaryOp::Ag,
                _
            )
        )
    }

    /// Check if this `HctlFormula` represents on of the binary temporal operators (see `TemporalBinaryOperator`).
    fn is_temporal_binary(&self) -> bool {
        matches!(
            self.value.node_type,
            NodeType::BinaryNode(
                BinaryOp::Eu | BinaryOp::Au | BinaryOp::Ew | BinaryOp::Aw,
                _,
                _
            )
        )
    }

    /// Check if this `HctlFormula` represents on of the binary Boolean operators (see `BinaryOperator`).
    fn is_boolean(&self) -> bool {
        matches!(
            self.value.node_type,
            NodeType::BinaryNode(
                BinaryOp::And | BinaryOp::Or | BinaryOp::Imp | BinaryOp::Iff | BinaryOp::Xor,
                _,
                _
            )
        )
    }

    /// Check if this `HctlFormula` represents a quantified state proposition.
    fn is_state_var(&self) -> bool {
        matches!(self.value.node_type, NodeType::TerminalNode(Atomic::Var(_)))
    }

    /// Check if this `HctlFormula` represents a network variable proposition.
    fn is_network_var(&self) -> bool {
        matches!(
            self.value.node_type,
            NodeType::TerminalNode(Atomic::Prop(_))
        )
    }

    /// Check if this `HctlFormula` represents a constant.
    fn is_const(&self) -> bool {
        matches!(
            self.value.node_type,
            NodeType::TerminalNode(Atomic::True | Atomic::False)
        )
    }

    /// Check if this `HctlFormula` represents an extended proposition.
    fn is_extended_prop(&self) -> bool {
        matches!(
            self.value.node_type,
            NodeType::TerminalNode(Atomic::WildCardProp(_))
        )
    }

    /// Check if this `HctlFormula` represents the `3{x}` operator.
    fn is_exists(&self) -> bool {
        matches!(
            self.value.node_type,
            NodeType::HybridNode(HybridOp::Exists, _, _)
        )
    }

    /// Check if this `HctlFormula` represents the `V{x}` operator.
    fn is_forall(&self) -> bool {
        matches!(
            self.value.node_type,
            NodeType::HybridNode(HybridOp::Forall, _, _)
        )
    }

    /// Check if this `HctlFormula` represents the `!{x}` operator.
    fn is_bind(&self) -> bool {
        matches!(
            self.value.node_type,
            NodeType::HybridNode(HybridOp::Bind, _, _)
        )
    }

    /// Check if this `HctlFormula` represents the `@{x}` operator.
    fn is_jump(&self) -> bool {
        matches!(
            self.value.node_type,
            NodeType::HybridNode(HybridOp::Jump, _, _)
        )
    }

    /// Check if this `HctlFormula` represents the `~` operator.
    fn is_not(&self) -> bool {
        matches!(self.value.node_type, NodeType::UnaryNode(UnaryOp::Not, _))
    }

    /// Check if this `HctlFormula` represents the `&` operator.
    fn is_and(&self) -> bool {
        matches!(
            self.value.node_type,
            NodeType::BinaryNode(BinaryOp::And, _, _)
        )
    }

    /// Check if this `HctlFormula` represents the `|` operator.
    fn is_or(&self) -> bool {
        matches!(
            self.value.node_type,
            NodeType::BinaryNode(BinaryOp::Or, _, _)
        )
    }

    /// Check if this `HctlFormula` represents the `=>` operator.
    fn is_imp(&self) -> bool {
        matches!(
            self.value.node_type,
            NodeType::BinaryNode(BinaryOp::Imp, _, _)
        )
    }

    /// Check if this `HctlFormula` represents the `<=>` operator.
    fn is_iff(&self) -> bool {
        matches!(
            self.value.node_type,
            NodeType::BinaryNode(BinaryOp::Iff, _, _)
        )
    }

    /// Check if this `HctlFormula` represents the `^ (xor)` operator.
    fn is_xor(&self) -> bool {
        matches!(
            self.value.node_type,
            NodeType::BinaryNode(BinaryOp::Xor, _, _)
        )
    }

    /// Check if this `HctlFormula` represents the `EX` operator.
    fn is_exist_next(&self) -> bool {
        matches!(self.value.node_type, NodeType::UnaryNode(UnaryOp::Ex, _))
    }

    /// Check if this `HctlFormula` represents the `AX` operator.
    fn is_all_next(&self) -> bool {
        matches!(self.value.node_type, NodeType::UnaryNode(UnaryOp::Ax, _))
    }

    /// Check if this `HctlFormula` represents the `EF` operator.
    fn is_exist_future(&self) -> bool {
        matches!(self.value.node_type, NodeType::UnaryNode(UnaryOp::Ef, _))
    }

    /// Check if this `HctlFormula` represents the `AF` operator.
    fn is_all_future(&self) -> bool {
        matches!(self.value.node_type, NodeType::UnaryNode(UnaryOp::Af, _))
    }

    /// Check if this `HctlFormula` represents the `EG` operator.
    fn is_exist_global(&self) -> bool {
        matches!(self.value.node_type, NodeType::UnaryNode(UnaryOp::Eg, _))
    }

    /// Check if this `HctlFormula` represents the `AG` operator.
    fn is_all_global(&self) -> bool {
        matches!(self.value.node_type, NodeType::UnaryNode(UnaryOp::Ag, _))
    }

    /// Check if this `HctlFormula` represents the `EU` operator.
    fn is_exist_until(&self) -> bool {
        matches!(
            self.value.node_type,
            NodeType::BinaryNode(BinaryOp::Eu, _, _)
        )
    }

    /// Check if this `HctlFormula` represents the `AU` operator.
    fn is_all_until(&self) -> bool {
        matches!(
            self.value.node_type,
            NodeType::BinaryNode(BinaryOp::Au, _, _)
        )
    }

    /// Check if this `HctlFormula` represents the `EW` operator.
    fn is_exist_weak_until(&self) -> bool {
        matches!(
            self.value.node_type,
            NodeType::BinaryNode(BinaryOp::Ew, _, _)
        )
    }

    /// Check if this `HctlFormula` represents the `AW` operator.
    fn is_all_weak_until(&self) -> bool {
        matches!(
            self.value.node_type,
            NodeType::BinaryNode(BinaryOp::Aw, _, _)
        )
    }

    /// Return the operator, variable and argument if this `HctlFormula` represents a hybrid operator.
    fn as_hybrid(&self) -> Option<(String, String, HctlFormula)> {
        match &self.value.node_type {
            NodeType::HybridNode(op, var, inner) => Some((
                encode_hybrid_operator(op),
                var.clone(),
                self.mk_child_ref(inner),
            )),
            _ => None,
        }
    }

    /// Return the operator and argument if this `HctlFormula` represents a unary temporal operator.
    fn as_temporal_unary(&self) -> Option<(String, HctlFormula)> {
        match &self.value.node_type {
            NodeType::UnaryNode(op, inner) => match op {
                UnaryOp::Ex
                | UnaryOp::Ax
                | UnaryOp::Ef
                | UnaryOp::Af
                | UnaryOp::Eg
                | UnaryOp::Ag => Some((encode_unary_operator(op), self.mk_child_ref(inner))),
                _ => None,
            },
            _ => None,
        }
    }

    /// Return the operator and arguments if this `HctlFormula` represents a binary temporal operator.
    fn as_temporal_binary(&self) -> Option<(String, HctlFormula, HctlFormula)> {
        match &self.value.node_type {
            NodeType::BinaryNode(op, a, b) => match op {
                BinaryOp::Eu | BinaryOp::Au | BinaryOp::Ew | BinaryOp::Aw => Some((
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
            NodeType::BinaryNode(op, a, b) => match op {
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
            NodeType::TerminalNode(Atomic::Var(name)) => Some(name.clone()),
            _ => None,
        }
    }

    /// Return the variable name if this `HctlFormula` represents a network variable proposition.
    fn as_network_var(&self) -> Option<String> {
        match &self.value.node_type {
            NodeType::TerminalNode(Atomic::Prop(name)) => Some(name.clone()),
            _ => None,
        }
    }

    /// Return the Boolean value if this `HctlFormula` represents a constant.
    fn as_const(&self) -> Option<bool> {
        match &self.value.node_type {
            NodeType::TerminalNode(Atomic::True) => Some(true),
            NodeType::TerminalNode(Atomic::False) => Some(false),
            _ => None,
        }
    }

    /// Return the property name if this `HctlFormula` represents an extended proposition (`%name%`).
    fn as_extended_prop(&self) -> Option<String> {
        match &self.value.node_type {
            NodeType::TerminalNode(Atomic::WildCardProp(name)) => Some(name.clone()),
            _ => None,
        }
    }

    /// Return the state variable name and the child formula if this `HctlFormula` represents
    /// the `3{x}` hybrid operator.
    fn as_exists(&self) -> Option<(String, HctlFormula)> {
        match &self.value.node_type {
            NodeType::HybridNode(HybridOp::Exists, name, inner) => {
                Some((name.clone(), self.mk_child_ref(inner)))
            }
            _ => None,
        }
    }

    /// Return the state variable name and the child formula if this `HctlFormula` represents
    /// the `V{x}` hybrid operator.
    fn as_forall(&self) -> Option<(String, HctlFormula)> {
        match &self.value.node_type {
            NodeType::HybridNode(HybridOp::Forall, name, inner) => {
                Some((name.clone(), self.mk_child_ref(inner)))
            }
            _ => None,
        }
    }

    /// Return the state variable name and the child formula if this `HctlFormula` represents
    /// the `!{x}` hybrid operator.
    fn as_bind(&self) -> Option<(String, HctlFormula)> {
        match &self.value.node_type {
            NodeType::HybridNode(HybridOp::Bind, name, inner) => {
                Some((name.clone(), self.mk_child_ref(inner)))
            }
            _ => None,
        }
    }

    /// Return the state variable name and the child formula if this `HctlFormula` represents
    /// the `@{x}` hybrid operator.
    fn as_jump(&self) -> Option<(String, HctlFormula)> {
        match &self.value.node_type {
            NodeType::HybridNode(HybridOp::Jump, name, inner) => {
                Some((name.clone(), self.mk_child_ref(inner)))
            }
            _ => None,
        }
    }

    /// Return the child formula if this `HctlFormula` represents the `~` operator.
    fn as_not(&self) -> Option<HctlFormula> {
        match &self.value.node_type {
            NodeType::UnaryNode(UnaryOp::Not, inner) => Some(self.mk_child_ref(inner)),
            _ => None,
        }
    }

    /// Return the two child formulas if this `HctlFormula` represents the `&` Boolean operator.
    fn as_and(&self) -> Option<(HctlFormula, HctlFormula)> {
        match &self.value.node_type {
            NodeType::BinaryNode(BinaryOp::And, a, b) => {
                Some((self.mk_child_ref(a), self.mk_child_ref(b)))
            }
            _ => None,
        }
    }

    /// Return the two child formulas if this `HctlFormula` represents the `|` Boolean operator.
    fn as_or(&self) -> Option<(HctlFormula, HctlFormula)> {
        match &self.value.node_type {
            NodeType::BinaryNode(BinaryOp::Or, a, b) => {
                Some((self.mk_child_ref(a), self.mk_child_ref(b)))
            }
            _ => None,
        }
    }

    /// Return the two child formulas if this `HctlFormula` represents the `=>` Boolean operator.
    fn as_imp(&self) -> Option<(HctlFormula, HctlFormula)> {
        match &self.value.node_type {
            NodeType::BinaryNode(BinaryOp::Imp, a, b) => {
                Some((self.mk_child_ref(a), self.mk_child_ref(b)))
            }
            _ => None,
        }
    }

    /// Return the two child formulas if this `HctlFormula` represents the `<=>` Boolean operator.
    fn as_iff(&self) -> Option<(HctlFormula, HctlFormula)> {
        match &self.value.node_type {
            NodeType::BinaryNode(BinaryOp::Iff, a, b) => {
                Some((self.mk_child_ref(a), self.mk_child_ref(b)))
            }
            _ => None,
        }
    }

    /// Return the two child formulas if this `HctlFormula` represents the `^` Boolean operator.
    fn as_xor(&self) -> Option<(HctlFormula, HctlFormula)> {
        match &self.value.node_type {
            NodeType::BinaryNode(BinaryOp::Xor, a, b) => {
                Some((self.mk_child_ref(a), self.mk_child_ref(b)))
            }
            _ => None,
        }
    }

    /// Return the child formula if this `HctlFormula` represents the `EX` temporal operator.
    fn as_exist_next(&self) -> Option<HctlFormula> {
        match &self.value.node_type {
            NodeType::UnaryNode(UnaryOp::Ex, inner) => Some(self.mk_child_ref(inner)),
            _ => None,
        }
    }

    /// Return the child formula if this `HctlFormula` represents the `AX` temporal operator.
    fn as_all_next(&self) -> Option<HctlFormula> {
        match &self.value.node_type {
            NodeType::UnaryNode(UnaryOp::Ax, inner) => Some(self.mk_child_ref(inner)),
            _ => None,
        }
    }

    /// Return the child formula if this `HctlFormula` represents the `EF` temporal operator.
    fn as_exist_future(&self) -> Option<HctlFormula> {
        match &self.value.node_type {
            NodeType::UnaryNode(UnaryOp::Ef, inner) => Some(self.mk_child_ref(inner)),
            _ => None,
        }
    }

    /// Return the child formula if this `HctlFormula` represents the `AF` temporal operator.
    fn as_all_future(&self) -> Option<HctlFormula> {
        match &self.value.node_type {
            NodeType::UnaryNode(UnaryOp::Af, inner) => Some(self.mk_child_ref(inner)),
            _ => None,
        }
    }

    /// Return the child formula if this `HctlFormula` represents the `EG` temporal operator.
    fn as_exist_global(&self) -> Option<HctlFormula> {
        match &self.value.node_type {
            NodeType::UnaryNode(UnaryOp::Eg, inner) => Some(self.mk_child_ref(inner)),
            _ => None,
        }
    }

    /// Return the child formula if this `HctlFormula` represents the `AG` temporal operator.
    fn as_all_global(&self) -> Option<HctlFormula> {
        match &self.value.node_type {
            NodeType::UnaryNode(UnaryOp::Ag, inner) => Some(self.mk_child_ref(inner)),
            _ => None,
        }
    }

    /// Return the two child formulas if this `HctlFormula` represents the `EU` temporal operator.
    fn as_exist_until(&self) -> Option<(HctlFormula, HctlFormula)> {
        match &self.value.node_type {
            NodeType::BinaryNode(BinaryOp::Eu, a, b) => {
                Some((self.mk_child_ref(a), self.mk_child_ref(b)))
            }
            _ => None,
        }
    }

    /// Return the two child formulas if this `HctlFormula` represents the `AU` temporal operator.
    fn as_all_until(&self) -> Option<(HctlFormula, HctlFormula)> {
        match &self.value.node_type {
            NodeType::BinaryNode(BinaryOp::Au, a, b) => {
                Some((self.mk_child_ref(a), self.mk_child_ref(b)))
            }
            _ => None,
        }
    }

    /// Return the two child formulas if this `HctlFormula` represents the `EW` temporal operator.
    fn as_exist_weak_until(&self) -> Option<(HctlFormula, HctlFormula)> {
        match &self.value.node_type {
            NodeType::BinaryNode(BinaryOp::Ew, a, b) => {
                Some((self.mk_child_ref(a), self.mk_child_ref(b)))
            }
            _ => None,
        }
    }

    /// Return the two child formulas if this `HctlFormula` represents the `AW` temporal operator.
    fn as_all_weak_until(&self) -> Option<(HctlFormula, HctlFormula)> {
        match &self.value.node_type {
            NodeType::BinaryNode(BinaryOp::Aw, a, b) => {
                Some((self.mk_child_ref(a), self.mk_child_ref(b)))
            }
            _ => None,
        }
    }
}
