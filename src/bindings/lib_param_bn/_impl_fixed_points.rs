use crate::bindings::lib_param_bn::{
    PyFixedPoints, PyGraphColoredVertices, PyGraphColors, PyGraphVertices, PySymbolicAsyncGraph,
};
use crate::AsNative;
use biodivine_lib_param_bn::fixed_points::FixedPoints;
use pyo3::pymethods;

#[pymethods]
impl PyFixedPoints {
    /// A naive symbolic algorithm that computes the fixed points by gradual
    /// elimination of all states with outgoing transitions.
    #[staticmethod]
    #[args(restriction = "None")]
    pub fn naive_symbolic(
        stg: &PySymbolicAsyncGraph,
        restriction: Option<&PyGraphColoredVertices>,
    ) -> PyGraphColoredVertices {
        if let Some(restriction) = restriction {
            FixedPoints::naive_symbolic(stg.as_native(), restriction.as_native()).into()
        } else {
            let restriction = stg.as_native().unit_colored_vertices();
            FixedPoints::naive_symbolic(stg.as_native(), restriction).into()
        }
    }

    /// A better version of the `Self::naive_symbolic` algorithm that can actually scale to
    /// reasonably sized networks (e.g. 100-200 variables + parameters).
    ///
    // Only fixed-points from the restriction set are returned. However, the state has to be a
    // global fixed point, not just a fixed-point within the restriction set.
    #[staticmethod]
    #[args(restriction = "None")]
    pub fn symbolic(
        stg: &PySymbolicAsyncGraph,
        restriction: Option<&PyGraphColoredVertices>,
    ) -> PyGraphColoredVertices {
        if let Some(restriction) = restriction {
            FixedPoints::symbolic(stg.as_native(), restriction.as_native()).into()
        } else {
            let restriction = stg.as_native().unit_colored_vertices();
            FixedPoints::symbolic(stg.as_native(), restriction).into()
        }
    }

    /// The result of the function are all vertices that can appear as fixed-points for some
    /// parameter valuation. That is, for every returned vertex, there is at least one color
    /// for which the vertex is a fixed-point.
    #[staticmethod]
    #[args(restriction = "None")]
    pub fn symbolic_vertices(
        stg: &PySymbolicAsyncGraph,
        restriction: Option<&PyGraphColoredVertices>,
    ) -> PyGraphVertices {
        if let Some(restriction) = restriction {
            FixedPoints::symbolic_vertices(stg.as_native(), restriction.as_native()).into()
        } else {
            let restriction = stg.as_native().unit_colored_vertices();
            FixedPoints::symbolic_vertices(stg.as_native(), restriction).into()
        }
    }

    /// Similar to `Self::symbolic_vertices`, but only returns colors for which there exists
    /// at least one fixed-point within restriction.
    #[staticmethod]
    #[args(restriction = "None")]
    pub fn symbolic_colors(
        stg: &PySymbolicAsyncGraph,
        restriction: Option<&PyGraphColoredVertices>,
    ) -> PyGraphColors {
        if let Some(restriction) = restriction {
            FixedPoints::symbolic_colors(stg.as_native(), restriction.as_native()).into()
        } else {
            let restriction = stg.as_native().unit_colored_vertices();
            FixedPoints::symbolic_colors(stg.as_native(), restriction).into()
        }
    }

    /// This function creates an iterator that yields symbolic sets of fixed-point states, such
    /// that eventually, all fixed-points are returned (and there are no duplicates).
    ///
    /// As with `Self::symbolic`, you can use the restriction set to limit the search to
    /// a particular subset of states. Furthermore, you can use size_limit to give the algorithm
    /// a hint as to how large (in terms of BDD nodes) you want the resulting sets to be.
    /// This value depends on how much memory and time you have, but a good starting value
    /// tends to be somewhere between 1_000_000 and 10_000_000. If you'd rather want more smaller
    /// sets, but get them quickly, you can go as low as 10_000 or 100_000 (given the model
    /// is not too large).
    ///
    /// It is not strictly guaranteed that the results are within this size limit, but the
    /// algorithm makes a reasonable effort to achieve this.
    ///
    /// WARNING: Due to technical difficulties, right now it is not possible to actually use
    /// this as an iterator, and instead you can use `item_limit` to set a limit on the number
    /// of returned items.
    #[staticmethod]
    #[args(restriction = "None")]
    pub fn symbolic_list(
        stg: &PySymbolicAsyncGraph,
        size_limit: usize,
        item_limit: Option<usize>,
        restriction: Option<&PyGraphColoredVertices>,
    ) -> Vec<PyGraphColoredVertices> {
        let item_limit = item_limit.unwrap_or(usize::MAX);
        let it = if let Some(restriction) = restriction {
            FixedPoints::symbolic_iterator(stg.as_native(), restriction.as_native(), size_limit)
        } else {
            let restriction = stg.as_native().unit_colored_vertices();
            FixedPoints::symbolic_iterator(stg.as_native(), restriction, size_limit)
        };
        it.take(item_limit)
            .map(PyGraphColoredVertices::from)
            .collect()
    }
}
