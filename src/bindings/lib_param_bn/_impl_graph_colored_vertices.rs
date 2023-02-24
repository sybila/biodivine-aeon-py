use crate::bindings::lib_bdd::PyBdd;
use crate::bindings::lib_param_bn::{
    PyGraphColoredVertices, PyGraphColors, PyGraphVertices, PySymbolicAsyncGraph, PyVariableId,
};
use crate::AsNative;
use biodivine_lib_bdd::Bdd;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use pyo3::prelude::*;

#[pymethods]
impl PyGraphColoredVertices {
    #[new]
    pub fn new(graph: &PySymbolicAsyncGraph, bdd: PyBdd) -> PyGraphColoredVertices {
        let ctx = graph.as_native().symbolic_context();
        assert_eq!(
            ctx.bdd_variable_set().num_vars(),
            bdd.as_native().num_vars()
        );
        PyGraphColoredVertices(graph.as_native().empty_vertices().copy(bdd.into()))
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

    pub fn symbolic_size(&self) -> usize {
        self.0.symbolic_size() * 10 // There are 10 bytes in a single BDD node.
    }

    pub fn to_dot(&self, graph: &PySymbolicAsyncGraph) -> String {
        self.as_native()
            .to_dot_string(graph.as_native().symbolic_context())
    }

    pub fn cardinality(&self) -> f64 {
        self.as_native().approx_cardinality()
    }

    pub fn pick_singleton(&self) -> Self {
        self.as_native().pick_singleton().into()
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

    pub fn is_subspace(&self) -> bool {
        self.as_native().is_subspace()
    }

    pub fn is_singleton(&self) -> bool {
        self.as_native().is_singleton()
    }

    pub fn fix_network_variable(
        &self,
        variable: PyVariableId,
        value: bool,
    ) -> PyGraphColoredVertices {
        self.as_native()
            .fix_network_variable(variable.into(), value)
            .into()
    }

    pub fn restrict_network_variable(
        &self,
        variable: PyVariableId,
        value: bool,
    ) -> PyGraphColoredVertices {
        self.as_native()
            .restrict_network_variable(variable.into(), value)
            .into()
    }

    pub fn vertices(&self) -> PyGraphVertices {
        self.as_native().vertices().into()
    }

    pub fn colors(&self) -> PyGraphColors {
        self.as_native().colors().into()
    }

    pub fn pick_color(&self) -> Self {
        self.as_native().pick_color().into()
    }

    pub fn pick_vertex(&self) -> Self {
        self.as_native().pick_vertex().into()
    }

    pub fn minus_colors(&self, other: &PyGraphColors) -> Self {
        self.as_native().minus_colors(other.as_native()).into()
    }

    pub fn intersect_colors(&self, other: &PyGraphColors) -> Self {
        self.as_native().intersect_colors(other.as_native()).into()
    }

    pub fn minus_vertices(&self, other: &PyGraphVertices) -> Self {
        self.as_native().minus_vertices(other.as_native()).into()
    }

    pub fn intersect_vertices(&self, other: &PyGraphVertices) -> Self {
        self.as_native()
            .intersect_vertices(other.as_native())
            .into()
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!(
            "ColoredVertexSet(cardinality = {}, unique vertices = {}, unique colors = {})",
            self.as_native().approx_cardinality(),
            self.as_native().vertices().approx_cardinality(),
            self.as_native().colors().approx_cardinality()
        ))
    }

    fn __repr__(&self) -> PyResult<String> {
        self.__str__()
    }
}
