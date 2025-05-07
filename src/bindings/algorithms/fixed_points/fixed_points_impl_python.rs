use pyo3::{pyclass, pymethods, PyResult};

use crate::bindings::{
    algorithms::graph_representation::PyGraphRepresentation,
    lib_param_bn::symbolic::{
        set_color::ColorSet, set_colored_vertex::ColoredVertexSet, set_vertex::VertexSet,
    },
};

use super::PyFixedPointsConfig;

/// Implements fixed point search over an `AsynchronousGraph`
#[pyclass(module = "biodivine_aeon", frozen)]
#[pyo3(name = "FixedPointsComp")]
pub struct PyFixedPoints(PyFixedPointsConfig);

/// These methods are Python facing wrappers of native methods and thus should not be used from
/// within Rust.
#[pymethods]
impl PyFixedPoints {
    /// Create a new `FixedPointsComp` instance with the given `AsynchronousGraph` or
    /// `BooleanNetwork` and otherwise default configuration.
    #[staticmethod]
    pub fn create_from(graph_representation: PyGraphRepresentation) -> PyResult<Self> {
        Ok(PyFixedPoints(PyFixedPointsConfig::try_from(
            graph_representation,
        )?))
    }

    /// Create a new `FixedPointsComp` instance with the given `FixedPointsConfig`.
    #[staticmethod]
    pub fn with_config(config: PyFixedPointsConfig) -> Self {
        PyFixedPoints(config)
    }

    /// A naive symbolic algorithm that computes the fixed points by gradual elimination of
    /// all states with outgoing transitions.
    ///
    /// Only fixed-points from the `restriction` set are returned. However, the state has to
    /// be a *global* fixed point, not just a fixed-point within the `restriction` set.
    ///
    /// **Characteristics:** As the name suggests, this algorithm is not really suited for
    /// processing complex networks. However, we provide it as a "baseline" for testing other
    /// algorithms. In theory, due to its simplicity, it could be faster on some of the smaller
    /// networks where the symbolic explosion is not severe.
    pub fn naive_symbolic(&self) -> PyResult<ColoredVertexSet> {
        Ok(ColoredVertexSet::mk_native(
            self.0.ctx.clone(),
            self.0.inner.naive_symbolic()?,
        ))
    }

    /// Iteratively compute the colored set of fixed-points in an `AsynchronousGraph` that are the
    /// subset of the `restriction` set.
    ///
    /// This is a better version of the `naive_symbolic()` algorithm that can actually scale to
    /// reasonably sized networks (e.g. 100-200 variables + parameters).
    pub fn symbolic(&self) -> PyResult<ColoredVertexSet> {
        Ok(ColoredVertexSet::mk_native(
            self.0.ctx.clone(),
            self.0.inner.symbolic()?,
        ))
    }

    /// Iteratively compute the set of fixed-point vertices in an `AsynchronousGraph`.
    ///
    /// This is equivalent to `FixedPointsComp.symbolic(graph, set).vertices()`, but can be
    /// significantly faster because the projection is applied on-demand within the algorithm.
    ///
    /// The result of the function are all vertices that can appear as fixed-points for **some**
    /// parameter valuation. That is, for every returned vertex, there is at least one color
    /// for which the vertex is a fixed-point.
    pub fn symbolic_vertices(&self) -> PyResult<VertexSet> {
        Ok(VertexSet::mk_native(
            self.0.ctx.clone(),
            self.0.inner.symbolic_vertices()?,
        ))
    }

    /// Iteratively compute the set of fixed-point colors in an `AsynchronousGraph`.
    ///
    /// This is equivalent to `FixedPointsComp.symbolic(graph, set).colors()`, but can be
    /// significantly faster because the projection is applied on-demand within the algorithm.
    ///
    /// Similar to `symbolic_vertices()`, but only returns colors for which there exists
    /// at least one fixed-point within `restriction`.
    pub fn symbolic_colors(&self) -> PyResult<ColorSet> {
        Ok(ColorSet::mk_native(
            self.0.ctx.clone(),
            self.0.inner.symbolic_colors()?,
        ))
    }
}
