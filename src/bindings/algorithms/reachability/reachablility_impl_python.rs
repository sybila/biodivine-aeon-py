use pyo3::{pymethods, PyResult};

use crate::{
    bindings::{
        algorithms::graph_representation::PyGraphRepresentation,
        lib_param_bn::symbolic::set_colored_vertex::ColoredVertexSet,
    },
    internal::algorithms::reachability::{Reachability, ReachabilityConfig},
    AsNative as _,
};

/// These methods are Python facing wrappers of native methods and thus should not be used from
/// within Rust.
#[pymethods]
impl Reachability {
    /// Create a new `ReachabilityComp` instance from the given `AsynchronousGraph` or `BooleanNetwork`,
    /// with otherwise default configuration.
    #[staticmethod]
    #[pyo3(name = "create_from")]
    pub fn python_create_from(graph_representation: PyGraphRepresentation) -> PyResult<Self> {
        Ok(Reachability(ReachabilityConfig::python_create_from(
            graph_representation,
        )?))
    }

    /// Create a new `ReachabilityComp` instance with the given `ReachabilityConfig`.
    #[staticmethod]
    #[pyo3(name = "with_config")]
    pub fn python_with_config(config: ReachabilityConfig) -> Self {
        Reachability(config)
    }

    /// Compute the *greatest superset* of the given `initial` set that is forward closed.
    ///
    /// Intuitively, these are all the vertices that are reachable from the `initial` set.
    #[pyo3(name = "forward_closed_superset")]
    pub fn python_forward_closed_superset(
        &self,
        initial: &ColoredVertexSet,
    ) -> PyResult<ColoredVertexSet> {
        Ok(ColoredVertexSet::mk_native(
            initial.ctx(),
            self.forward_closed_superset(initial.as_native())?,
        ))
    }

    /// Compute the *greatest superset* of the given `initial` set that is backward closed.
    ///
    /// Intuitively, these are all the vertices that can reach a vertex in the `initial` set.
    #[pyo3(name = "backward_closed_superset")]
    pub fn python_backward_closed_superset(
        &self,
        initial: &ColoredVertexSet,
    ) -> PyResult<ColoredVertexSet> {
        Ok(ColoredVertexSet::mk_native(
            initial.ctx(),
            self.backward_closed_superset(initial.as_native())?,
        ))
    }

    /// Compute the *greatest subset* of the given `initial` set that is forward closed.
    ///
    /// Intuitively, this removes all vertices that can reach a vertex outside the `initial`
    /// set.
    #[pyo3(name = "forward_closed_subset")]
    pub fn python_forward_closed_subset(
        &self,
        initial: &ColoredVertexSet,
    ) -> PyResult<ColoredVertexSet> {
        Ok(ColoredVertexSet::mk_native(
            initial.ctx(),
            self.forward_closed_subset(initial.as_native())?,
        ))
    }

    /// Compute the *greatest subset* of the given `initial` set that is backward closed.
    ///
    /// Intuitively, this removes all vertices that can be reached by a vertex outside
    /// the `initial` set.
    #[pyo3(name = "backward_closed_subset")]
    pub fn python_backward_closed_subset(
        &self,
        initial: &ColoredVertexSet,
    ) -> PyResult<ColoredVertexSet> {
        Ok(ColoredVertexSet::mk_native(
            initial.ctx(),
            self.backward_closed_subset(initial.as_native())?,
        ))
    }
}
