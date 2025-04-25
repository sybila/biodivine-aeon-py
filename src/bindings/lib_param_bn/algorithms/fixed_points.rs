use crate::bindings::lib_param_bn::symbolic::asynchronous_graph::AsynchronousGraph;
use crate::bindings::lib_param_bn::symbolic::set_color::ColorSet;
use crate::bindings::lib_param_bn::symbolic::set_colored_vertex::ColoredVertexSet;
use crate::bindings::lib_param_bn::symbolic::set_vertex::VertexSet;
use crate::{global_log_level, AsNative};
use pyo3::prelude::*;

// TODO: finalize - deprecate this file

#[pyclass(module = "biodivine_aeon", frozen)]
pub struct FixedPointsOld {
    _dummy: (),
}

#[pymethods]
impl FixedPointsOld {
    /// Iteratively compute the colored set of fixed-points in an `AsynchronousGraph` that are the
    /// subset of the `restriction` set.
    #[staticmethod]
    #[pyo3(signature = (stg, restriction = None))]
    pub fn symbolic(
        py: Python,
        stg: &AsynchronousGraph,
        restriction: Option<&ColoredVertexSet>,
    ) -> PyResult<ColoredVertexSet> {
        let restriction = restriction
            .map(|it| it.as_native())
            .unwrap_or(stg.as_native().unit_colored_vertices());
        let result = biodivine_lib_param_bn::fixed_points::FixedPoints::_symbolic(
            stg.as_native(),
            restriction,
            global_log_level(py)?,
            &|| py.check_signals(),
        )?;
        Ok(ColoredVertexSet::mk_native(stg.symbolic_context(), result))
    }

    /// Iteratively compute the set of fixed-point vertices in an `AsynchronousGraph`.
    ///
    /// This is equivalent to `FixedPoints.symbolic(graph, set).vertices()`, but can be
    /// significantly faster because the projection is applied on-demand within the algorithm.
    #[staticmethod]
    #[pyo3(signature = (stg, restriction = None))]
    pub fn symbolic_vertices(
        py: Python,
        stg: &AsynchronousGraph,
        restriction: Option<&ColoredVertexSet>,
    ) -> PyResult<VertexSet> {
        let restriction = restriction
            .map(|it| it.as_native())
            .unwrap_or(stg.as_native().unit_colored_vertices());
        let result = biodivine_lib_param_bn::fixed_points::FixedPoints::_symbolic_vertices(
            stg.as_native(),
            restriction,
            global_log_level(py)?,
            &|| py.check_signals(),
        )?;
        Ok(VertexSet::mk_native(stg.symbolic_context(), result))
    }

    /// Iteratively compute the set of fixed-point vertices in an `AsynchronousGraph`.
    ///
    /// This is equivalent to `FixedPoints.symbolic(graph, set).colors()`, but can be
    /// significantly faster because the projection is applied on-demand within the algorithm.
    #[staticmethod]
    #[pyo3(signature = (stg, restriction = None))]
    pub fn symbolic_colors(
        py: Python,
        stg: &AsynchronousGraph,
        restriction: Option<&ColoredVertexSet>,
    ) -> PyResult<ColorSet> {
        let restriction = restriction
            .map(|it| it.as_native())
            .unwrap_or(stg.as_native().unit_colored_vertices());
        let result = biodivine_lib_param_bn::fixed_points::FixedPoints::_symbolic_colors(
            stg.as_native(),
            restriction,
            global_log_level(py)?,
            &|| py.check_signals(),
        )?;
        Ok(ColorSet::mk_native(stg.symbolic_context(), result))
    }
}
