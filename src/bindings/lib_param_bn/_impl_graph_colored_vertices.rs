use crate::bindings::lib_bdd::PyBdd;
use crate::bindings::lib_param_bn::{
    PyGraphColoredVertices, PyGraphColors, PyGraphVertices, PySymbolicAsyncGraph,
};
use crate::AsNative;
use biodivine_lib_bdd::Bdd;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices;
use pyo3::prelude::*;

impl From<PyGraphColoredVertices> for GraphColoredVertices {
    fn from(value: PyGraphColoredVertices) -> Self {
        value.0
    }
}

impl From<GraphColoredVertices> for PyGraphColoredVertices {
    fn from(value: GraphColoredVertices) -> Self {
        PyGraphColoredVertices(value)
    }
}

impl AsNative<GraphColoredVertices> for PyGraphColoredVertices {
    fn as_native(&self) -> &GraphColoredVertices {
        &self.0
    }

    fn as_native_mut(&mut self) -> &mut GraphColoredVertices {
        &mut self.0
    }
}

#[pymethods]
impl PyGraphColoredVertices {
    /// Get a copy of this set in the form of a `Bdd` object.
    pub fn to_bdd(&self) -> PyBdd {
        self.as_native().as_bdd().clone().into()
    }

    /// Populate a new `ColorSet` using a raw `Bdd`.
    pub fn copy_with(&self, bdd: PyBdd) -> Self {
        self.as_native().copy(bdd.into()).into()
    }

    /// Populate a new `ColoredVertexSet` using a raw `Bdd` represented as a string.
    pub fn copy_with_raw_string(&self, bdd: String) -> PyResult<Self> {
        Ok(self
            .as_native()
            .copy(Bdd::from_string(bdd.as_str()).into())
            .into())
    }

    /// Get an approximate memory consumption of this symbolic set in bytes.
    ///
    /// (real value may be different due to underlying details of OS and allocation mechanisms)
    pub fn symbolic_size(&self) -> usize {
        self.0.symbolic_size() * 10 // There are 10 bytes in a single BDD node.
    }

    /// Compute a `.dot` string representing the underlying BDD graph.
    ///
    /// Needs a reference to the underlying symbolic graph to resolve variable names.
    /// If you are ok with an anonymous BDD graph, convert the set to `Bdd` and then use
    /// anonymous `.dot` conversion function on the result.
    pub fn to_dot(&self, graph: &PySymbolicAsyncGraph) -> String {
        self.as_native()
            .to_dot_string(graph.as_native().symbolic_context())
    }

    /// Get an approximate count of elements in this set.
    pub fn cardinality(&self) -> f64 {
        self.as_native().approx_cardinality()
    }

    /// Compute a union of two `ColoredVertexSet` objects.
    pub fn union(&self, other: &Self) -> Self {
        self.as_native().union(other.as_native()).into()
    }

    /// Compute an intersection of two `ColoredVertexSet` objects.
    pub fn intersect(&self, other: &Self) -> Self {
        self.as_native().intersect(other.as_native()).into()
    }

    /// Compute a difference of two `ColoredVertexSet` objects.
    pub fn minus(&self, other: &Self) -> Self {
        self.as_native().minus(other.as_native()).into()
    }

    /// Returns true if this `ColoredVertexSet` is empty.
    pub fn is_empty(&self) -> bool {
        self.as_native().is_empty()
    }

    /// Returns true if this `ColoredVertexSet` is a subset of the argument `ColoredVertexSet`.
    pub fn is_subset(&self, other: &Self) -> bool {
        self.as_native().is_subset(other.as_native())
    }

    /// Return a `VertexSet` of vertices that appear in this
    /// `ColoredVertexSet` for *at least* one color.
    pub fn vertices(&self) -> PyGraphVertices {
        self.as_native().vertices().into()
    }

    /// Return a `ColorSet` of colors that appear in this
    /// `ColoredVertexSet` for *at least* one vertex.
    pub fn colors(&self) -> PyGraphColors {
        self.as_native().colors().into()
    }

    /// Pick a single vertex-color pair from this `ColoredVertexSet` and return this pair
    /// as a singleton set.
    pub fn pick_singleton(&self) -> Self {
        self.as_native().pick_singleton().into()
    }

    /// Compute a `ColoredVertexSet` that contains exactly one color for every vertex in this set.
    ///
    /// That is, the resulting set contains one *witness* color per vertex in the original set.
    pub fn pick_color(&self) -> Self {
        self.as_native().pick_color().into()
    }

    /// Compute a `ColoredVertexSet` that contains exactly one vertex for every color in this set.
    ///
    /// That is, the resulting set contains one *witness* vertex per colour in the original set.
    pub fn pick_vertex(&self) -> Self {
        self.as_native().pick_vertex().into()
    }

    /// Remove all colors in the given `ColorSet` from this `ColoredVertexSet`,
    /// regardless of their associated vertex.
    pub fn minus_colors(&self, other: &PyGraphColors) -> Self {
        self.as_native().minus_colors(other.as_native()).into()
    }

    /// Compute a `ColoredVertexSet` that only retains colors from the given `ColorSet`,
    /// regardless of their associated vertex.
    pub fn intersect_colors(&self, other: &PyGraphColors) -> Self {
        self.as_native().intersect_colors(other.as_native()).into()
    }

    /// Remove all vertices in the given `VertexSet` from this `ColoredVertexSet`,
    /// regardless of their associated color.
    pub fn minus_vertices(&self, other: &PyGraphVertices) -> Self {
        self.as_native().minus_vertices(other.as_native()).into()
    }

    /// Compute a `ColoredVertexSet` that only retains vertices from the given `VertexSet`,
    /// regardless of their associated color.
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
