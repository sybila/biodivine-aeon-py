use crate::bindings::hctl_model_checker::PyHctlTreeNode;
use crate::bindings::lib_param_bn::PyBooleanNetwork;
use crate::{throw_runtime_error, AsNative};
use std::collections::HashSet;

use biodivine_hctl_model_checker::mc_utils::collect_unique_hctl_vars;
use biodivine_hctl_model_checker::preprocessing::node::NodeType;
use biodivine_hctl_model_checker::preprocessing::parser::{
    parse_and_minimize_extended_formula, parse_and_minimize_hctl_formula, parse_hctl_formula,
};
use biodivine_lib_param_bn::symbolic_async_graph::SymbolicContext;

use pyo3::basic::CompareOp;
use pyo3::prelude::*;

#[pymethods]
impl PyHctlTreeNode {
    fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
        let a = self.as_native();
        let b = other.as_native();
        match op {
            CompareOp::Lt => throw_runtime_error("Unsupported operation."),
            CompareOp::Le => throw_runtime_error("Unsupported operation."),
            CompareOp::Eq => Ok(a == b),
            CompareOp::Ne => Ok(a != b),
            CompareOp::Gt => throw_runtime_error("Unsupported operation."),
            CompareOp::Ge => throw_runtime_error("Unsupported operation."),
        }
    }

    fn __str__(&self) -> String {
        self.as_native().subform_str.clone()
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }

    #[new]
    /// Generate syntax tree given a HCTL formula and corresponding Boolean network.
    /// Tree is slightly modified after the parsing (variable renaming, etc.) due to optimizations.
    /// Validity of the formula is checked during parsing, including proposition names.
    pub fn new(formula: String, bn: &PyBooleanNetwork) -> PyResult<PyHctlTreeNode> {
        let ctx = SymbolicContext::new(bn.as_native()).unwrap();
        match parse_and_minimize_hctl_formula(&ctx, formula.as_str()) {
            Ok(tree) => Ok(PyHctlTreeNode(tree)),
            Err(error) => throw_runtime_error(error),
        }
    }

    /// Collect all unique HCTL variables occurring in the quantifiers in the tree nodes.
    pub fn collect_unique_hctl_vars(&self) -> HashSet<String> {
        collect_unique_hctl_vars(self.as_native().clone())
    }

    /// Return child node(s). If there are two children, returns left first.
    pub fn get_children(&self) -> Vec<PyHctlTreeNode> {
        let mut children = Vec::new();
        match self.0.node_type.clone() {
            NodeType::TerminalNode(_) => (),
            NodeType::UnaryNode(_, child) => {
                children.push(PyHctlTreeNode(*child));
            }
            NodeType::BinaryNode(_, left, right) => {
                children.push(PyHctlTreeNode(*left));
                children.push(PyHctlTreeNode(*right));
            }
            NodeType::HybridNode(_, _, child) => {
                children.push(PyHctlTreeNode(*child));
            }
        }

        children
    }

    /// Return 'operator' that is represented by the node, in a string form.
    /// For unary/binary nodes, simply returns the operator, such as "|".
    /// For terminal nodes, returns name of the var/prop/constant, such as "{x}".
    /// For hybrid nodes, returns operator string + name of the var, like "Bind {x}:".
    pub fn get_operator(&self) -> String {
        match self.0.node_type.clone() {
            NodeType::TerminalNode(atom) => format!("{atom}"),
            NodeType::UnaryNode(op, _) => format!("{op}"),
            NodeType::BinaryNode(op, _, _) => format!("{op}"),
            NodeType::HybridNode(op, var, _) => format!("{op} {{{var}}}:"),
        }
    }

    #[staticmethod]
    /// Parse an extended HCTL formula string representation into an actual formula tree
    /// with renamed (minimized) set of variables. Validity of the formula is checked during
    /// parsing, including proposition names.
    /// Extended formulae can include `wild-card propositions` in form `%proposition%`.
    pub fn new_from_extended(formula: String, bn: &PyBooleanNetwork) -> PyResult<PyHctlTreeNode> {
        let ctx = SymbolicContext::new(bn.as_native()).unwrap();
        match parse_and_minimize_extended_formula(&ctx, formula.as_str()) {
            Ok(tree) => Ok(PyHctlTreeNode(tree)),
            Err(error) => throw_runtime_error(error),
        }
    }

    #[staticmethod]
    /// Generate syntax tree given a HCTL formula. Tree is generated exactly according to
    /// the formula, and it is not modified.
    /// This tree cannot be used for model checking directly (use `HctlTreeNode::new()` instead).
    /// Validity of the formula is checked during parsing, but not proposition names.
    pub fn build_exact_from_formula(formula: String) -> PyResult<PyHctlTreeNode> {
        match parse_hctl_formula(formula.as_str()) {
            Ok(tree) => Ok(PyHctlTreeNode(tree)),
            Err(error) => throw_runtime_error(error),
        }
    }
}
