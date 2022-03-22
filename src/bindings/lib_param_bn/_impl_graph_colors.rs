use crate::bindings::lib_bdd::PyBdd;
use crate::bindings::lib_param_bn::{PyGraphColors, PySymbolicAsyncGraph};
use crate::AsNative;
use biodivine_lib_bdd::Bdd;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::GraphColors;
use pyo3::prelude::*;

impl From<PyGraphColors> for GraphColors {
    fn from(value: PyGraphColors) -> Self {
        value.0
    }
}

impl From<GraphColors> for PyGraphColors {
    fn from(value: GraphColors) -> Self {
        PyGraphColors(value)
    }
}

impl AsNative<GraphColors> for PyGraphColors {
    fn as_native(&self) -> &GraphColors {
        &self.0
    }

    fn as_native_mut(&mut self) -> &mut GraphColors {
        &mut self.0
    }
}

#[pymethods]
impl PyGraphColors {
    /// Get a copy of this set in the form of a `Bdd` object.
    pub fn to_bdd(&self) -> PyBdd {
        self.as_native().as_bdd().clone().into()
    }

    /// Populate a new `ColorSet` using a raw `Bdd`.
    pub fn copy_with(&self, bdd: PyBdd) -> Self {
        self.as_native().copy(bdd.into()).into()
    }

    /// Populate a new `ColorSet` using a raw `Bdd` represented as a string.
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

    /// Return a `ColorSet` with a single element picked from this set, or an empty set if this
    /// set is empty.
    pub fn pick_singleton(&self) -> Self {
        self.as_native().pick_singleton().into()
    }

    /// Compute a union of two `ColorSet` objects.
    pub fn union(&self, other: &Self) -> Self {
        self.as_native().union(other.as_native()).into()
    }

    /// Compute an intersection of two `ColorSet` objects.
    pub fn intersect(&self, other: &Self) -> Self {
        self.as_native().intersect(other.as_native()).into()
    }

    /// Compute a difference of two `ColorSet` objects.
    pub fn minus(&self, other: &Self) -> Self {
        self.as_native().minus(other.as_native()).into()
    }

    /// Returns true if this set is empty.
    pub fn is_empty(&self) -> bool {
        self.as_native().is_empty()
    }

    /// Returns true if this `ColorSet` is a subset the argument `ColorSet`.
    pub fn is_subset(&self, other: &Self) -> bool {
        self.as_native().is_subset(&other.0)
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!(
            "ColorSet(cardinality={})",
            self.0.approx_cardinality()
        ))
    }

    fn __repr__(&self) -> PyResult<String> {
        self.__str__()
    }
}
