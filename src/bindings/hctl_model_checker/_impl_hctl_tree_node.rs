use std::collections::HashSet;
use crate::bindings::hctl_model_checker::PyHctlTreeNode;
use crate::bindings::lib_param_bn::PyBooleanNetwork;
use crate::{throw_runtime_error, AsNative};

use biodivine_hctl_model_checker::preprocessing::parser::{
    parse_and_minimize_hctl_formula, parse_hctl_formula
};
use biodivine_hctl_model_checker::mc_utils::collect_unique_hctl_vars;

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
        match parse_and_minimize_hctl_formula(bn.as_native(), formula.as_str()) {
            Ok(tree) => Ok(PyHctlTreeNode(tree)),
            Err(error) => throw_runtime_error(error)
        }
    }

    /// Collect all unique HCTL variables occurring in the quantifiers in the tree nodes.
    pub fn collect_unique_hctl_vars(&self) -> HashSet<String> {
        collect_unique_hctl_vars(self.as_native().clone(), HashSet::new())
    }

    #[staticmethod]
    /// Generate syntax tree given a HCTL formula and corresponding Boolean network.
    /// Tree is generated exactly according to the formula, and it is not modified. This tree
    /// cannot be used for model checking directly (use `HctlTreeNode::new()` instead).
    /// Validity of the formula is checked during parsing, but not proposition names.
    pub fn build_exact_from_formula(formula: String) -> PyResult<PyHctlTreeNode> {
        match parse_hctl_formula(formula.as_str()) {
            Ok(tree) => Ok(PyHctlTreeNode(tree)),
            Err(error) => throw_runtime_error(error)
        }
    }
}