use crate::bindings::lib_bdd::PyBdd;
use crate::bindings::lib_param_bn::{
    PyGraphVertexIterator, PyGraphVertices, PySymbolicAsyncGraph, PyVariableId,
};
use crate::{throw_runtime_error, AsNative};
use biodivine_lib_bdd::{Bdd, BddVariable};
use biodivine_lib_param_bn::biodivine_std::bitvector::BitVector;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::GraphVertices;
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use std::collections::HashSet;

#[pymethods]
impl PyGraphVertices {
    fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
        let a = self.as_native().as_bdd();
        let b = other.as_native().as_bdd();
        match op {
            CompareOp::Lt => throw_runtime_error("Unsupported operation."),
            CompareOp::Le => throw_runtime_error("Unsupported operation."),
            CompareOp::Eq => Ok(a.xor(b).is_false()),
            CompareOp::Ne => Ok(a.iff(b).is_false()),
            CompareOp::Gt => throw_runtime_error("Unsupported operation."),
            CompareOp::Ge => throw_runtime_error("Unsupported operation."),
        }
    }

    #[new]
    pub fn new(graph: &PySymbolicAsyncGraph, bdd: PyBdd) -> PyGraphVertices {
        let ctx = graph.as_native().symbolic_context();
        assert_eq!(
            ctx.bdd_variable_set().num_vars(),
            bdd.as_native().num_vars()
        );
        let support = bdd.as_native().support_set();
        let expected: HashSet<BddVariable> = ctx.state_variables().iter().cloned().collect();
        assert!(support.is_subset(&expected));
        PyGraphVertices(GraphVertices::new(
            bdd.into(),
            graph.as_native().symbolic_context(),
        ))
    }

    pub fn to_bdd(&self) -> PyBdd {
        self.as_native().as_bdd().clone().into()
    }

    pub fn copy_with(&self, bdd: PyBdd) -> Self {
        self.as_native().copy(bdd.into()).into()
    }

    pub fn copy_with_raw_string(&self, bdd: String) -> PyResult<Self> {
        Ok(self.as_native().copy(Bdd::from_string(bdd.as_str())).into())
    }

    pub fn cardinality(&self) -> f64 {
        self.as_native().approx_cardinality()
    }

    pub fn symbolic_size(&self) -> usize {
        self.as_native().symbolic_size() * 10
    }

    pub fn to_dot(&self, graph: &PySymbolicAsyncGraph) -> String {
        self.as_native().to_dot_string(graph.0.symbolic_context())
    }

    pub fn union(&self, other: &Self) -> Self {
        self.as_native().union(other.as_native()).into()
    }

    pub fn intersect(&self, other: &Self) -> Self {
        self.as_native().intersect(other.as_native()).into()
    }

    pub fn minus(&self, other: &Self) -> Self {
        self.as_native().minus(other.as_native()).into()
    }

    pub fn is_empty(&self) -> bool {
        self.as_native().is_empty()
    }

    pub fn is_subset(&self, other: &Self) -> bool {
        self.as_native().is_subset(other.as_native())
    }

    pub fn pick_singleton(&self) -> Self {
        self.as_native().pick_singleton().into()
    }

    pub fn is_subspace(&self) -> bool {
        self.as_native().is_subspace()
    }

    pub fn is_singleton(&self) -> bool {
        self.as_native().is_singleton()
    }

    pub fn fix_network_variable(&self, variable: PyVariableId, value: bool) -> PyGraphVertices {
        self.as_native()
            .fix_network_variable(variable.into(), value)
            .into()
    }

    pub fn restrict_network_variable(
        &self,
        variable: PyVariableId,
        value: bool,
    ) -> PyGraphVertices {
        self.as_native()
            .restrict_network_variable(variable.into(), value)
            .into()
    }

    pub fn iterator(&self) -> PyGraphVertexIterator {
        PyGraphVertexIterator(self.as_native().iter())
    }

    pub fn __iter__(&self) -> PyGraphVertexIterator {
        self.iterator()
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("VertexSet(cardinality = {})", self.cardinality()))
    }

    fn __repr__(&self) -> PyResult<String> {
        self.__str__()
    }
}

#[pymethods]
impl PyGraphVertexIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<Vec<bool>> {
        slf.0.next().map(|it| it.values())
    }
}
