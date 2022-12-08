use crate::bindings::lib_param_bn::{
    PyFixedPoints, PyGraphColoredVertices, PyGraphColors, PyGraphVertices, PySymbolicAsyncGraph,
};
use crate::AsNative;
use biodivine_lib_bdd::BddPartialValuation;
use biodivine_lib_param_bn::biodivine_std::bitvector::BitVector;
use biodivine_lib_param_bn::fixed_points::FixedPoints;
use biodivine_lib_param_bn::solver_context::BnSolverContext;
use biodivine_lib_param_bn::{ExtendedBoolean, Space, VariableId};
use pyo3::types::PyDict;
use pyo3::{pymethods, PyResult};

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
    /// this as an iterator, and instead you can use `limit` argument to set a limit on the number
    /// of returned items.
    #[staticmethod]
    #[args(restriction = "None")]
    pub fn symbolic_list(
        stg: &PySymbolicAsyncGraph,
        size_limit: usize,
        limit: Option<usize>,
        restriction: Option<&PyGraphColoredVertices>,
    ) -> Vec<PyGraphColoredVertices> {
        let limit = limit.unwrap_or(usize::MAX);
        let it = if let Some(restriction) = restriction {
            FixedPoints::symbolic_iterator(stg.as_native(), restriction.as_native(), size_limit)
        } else {
            let restriction = stg.as_native().unit_colored_vertices();
            FixedPoints::symbolic_iterator(stg.as_native(), restriction, size_limit)
        };
        it.take(limit).map(PyGraphColoredVertices::from).collect()
    }

    /// Compute the fixed points (or rather, vertex-color pairs) of the provided asynchronous
    /// graph, but using the Z3 solver.
    ///
    /// If the number of fixed-points is high (which it usually is), you can use `limit` argument
    /// to stop the enumeration.
    ///
    /// You can provide a list of subspaces (dictionaries mapping network variables to values)
    /// as a `positive_restriction` (in which case results will be only from these spaces) and
    /// `negative_restriction` (in which case results will exclude these spaces).
    ///
    /// WARNING: Due to technical issues, right now this is only a list, not an iterator.
    #[staticmethod]
    #[args(limit = "None")]
    #[args(positive_restriction = "None")]
    #[args(negative_restriction = "None")]
    pub fn solver_list(
        stg: &PySymbolicAsyncGraph,
        limit: Option<usize>,
        positive_restriction: Option<Vec<&PyDict>>,
        negative_restriction: Option<Vec<&PyDict>>,
    ) -> PyResult<Vec<PyGraphColoredVertices>> {
        // If no restriction is provided, we include the space of all states.
        let mut positive_spaces = Vec::new();
        if let Some(positive_restriction) = positive_restriction {
            for space in positive_restriction {
                positive_spaces.push(read_space(stg, space)?);
            }
        } else {
            positive_spaces.push(Space::new(stg.as_native().as_network()));
        }
        // A negative restriction is simply empty if not provided.
        let negative_restriction = negative_restriction.unwrap_or_default();
        let mut negative_spaces = Vec::new();
        for space in negative_restriction {
            negative_spaces.push(read_space(stg, space)?);
        }

        let z3_config = z3::Config::new();
        let z3 = z3::Context::new(&z3_config);
        let network_copy = stg.as_native().as_network().clone();
        let context = BnSolverContext::new(&z3, network_copy);
        let iterator = FixedPoints::solver_iterator(&context, &positive_spaces, &negative_spaces);
        let limit = limit.unwrap_or(usize::MAX);
        let results: Vec<PyGraphColoredVertices> = iterator
            .take(limit)
            .map(|it| {
                it.get_symbolic_model(stg.as_native().symbolic_context())
                    .into()
            })
            .collect();
        Ok(results)
    }

    /// Compute the fixed point vertices of the provided asynchronous
    /// graph, but using the Z3 solver.
    ///
    /// If the number of fixed-points is high (which it usually is), you can use `limit` argument
    /// to stop the enumeration.
    ///
    /// You can provide a list of subspaces (dictionaries mapping network variables to values)
    /// as a `positive_restriction` (in which case results will be only from these spaces) and
    /// `negative_restriction` (in which case results will exclude these spaces).
    ///
    /// WARNING: Due to technical issues, right now this is only a list, not an iterator.
    #[staticmethod]
    #[args(limit = "None")]
    #[args(positive_restriction = "None")]
    #[args(negative_restriction = "None")]
    pub fn solver_vertex_list(
        stg: &PySymbolicAsyncGraph,
        limit: Option<usize>,
        positive_restriction: Option<Vec<&PyDict>>,
        negative_restriction: Option<Vec<&PyDict>>,
    ) -> PyResult<Vec<PyGraphVertices>> {
        // If no restriction is provided, we include the space of all states.
        let mut positive_spaces = Vec::new();
        if let Some(positive_restriction) = positive_restriction {
            for space in positive_restriction {
                positive_spaces.push(read_space(stg, space)?);
            }
        } else {
            positive_spaces.push(Space::new(stg.as_native().as_network()));
        }
        // A negative restriction is simply empty if not provided.
        let negative_restriction = negative_restriction.unwrap_or_default();
        let mut negative_spaces = Vec::new();
        for space in negative_restriction {
            negative_spaces.push(read_space(stg, space)?);
        }

        let z3_config = z3::Config::new();
        let z3 = z3::Context::new(&z3_config);
        let network_copy = stg.as_native().as_network().clone();
        let context = BnSolverContext::new(&z3, network_copy);
        let iterator =
            FixedPoints::solver_vertex_iterator(&context, &positive_spaces, &negative_spaces);
        let limit = limit.unwrap_or(usize::MAX);

        // This is a slightly complicated way to convert the vertex
        // back into a symbolic object, but it's faster than calling
        // `stg.as_native().vertex(it).vertices()`.
        let bdd_vars = stg.as_native().symbolic_context().bdd_variable_set();
        let symbolic_variables = stg.as_native().symbolic_context().state_variables();
        let empty_vertices = stg.as_native().unit_colored_vertices().vertices();
        let mut p_val = BddPartialValuation::empty();
        let results: Vec<PyGraphVertices> = iterator
            .take(limit)
            .map(|it| {
                for (k, v) in symbolic_variables.iter().zip(it.values().iter()) {
                    p_val.set_value(*k, *v);
                }
                let bdd = bdd_vars.mk_conjunctive_clause(&p_val);
                let singleton = empty_vertices.copy(bdd);
                singleton.into()
            })
            .collect();
        Ok(results)
    }

    /// Compute the fixed point colors of the provided asynchronous
    /// graph, but using the Z3 solver.
    ///
    /// If the number of fixed-points is high (which it usually is), you can use `limit` argument
    /// to stop the enumeration.
    ///
    /// You can provide a list of subspaces (dictionaries mapping network variables to values)
    /// as a `positive_restriction` (in which case results will be only from these spaces) and
    /// `negative_restriction` (in which case results will exclude these spaces).
    ///
    /// WARNING: Due to technical issues, right now this is only a list, not an iterator.
    #[staticmethod]
    #[args(limit = "None")]
    #[args(positive_restriction = "None")]
    #[args(negative_restriction = "None")]
    pub fn solver_color_list(
        stg: &PySymbolicAsyncGraph,
        limit: Option<usize>,
        positive_restriction: Option<Vec<&PyDict>>,
        negative_restriction: Option<Vec<&PyDict>>,
    ) -> PyResult<Vec<PyGraphColors>> {
        // If no restriction is provided, we include the space of all states.
        let mut positive_spaces = Vec::new();
        if let Some(positive_restriction) = positive_restriction {
            for space in positive_restriction {
                positive_spaces.push(read_space(stg, space)?);
            }
        } else {
            positive_spaces.push(Space::new(stg.as_native().as_network()));
        }
        // A negative restriction is simply empty if not provided.
        let negative_restriction = negative_restriction.unwrap_or_default();
        let mut negative_spaces = Vec::new();
        for space in negative_restriction {
            negative_spaces.push(read_space(stg, space)?);
        }

        let z3_config = z3::Config::new();
        let z3 = z3::Context::new(&z3_config);
        let network_copy = stg.as_native().as_network().clone();
        let context = BnSolverContext::new(&z3, network_copy);
        let iterator =
            FixedPoints::solver_color_iterator(&context, &positive_spaces, &negative_spaces);
        let limit = limit.unwrap_or(usize::MAX);
        let results: Vec<PyGraphColors> = iterator
            .take(limit)
            .map(|it| {
                it.get_symbolic_colors(stg.as_native().symbolic_context())
                    .into()
            })
            .collect();
        Ok(results)
    }
}

fn read_space(stg: &PySymbolicAsyncGraph, py_space: &PyDict) -> PyResult<Space> {
    let mut space = Space::new(stg.as_native().as_network());
    for (k, v) in py_space {
        let key: VariableId = stg.resolve_variable(k)?.into();
        let value: ExtendedBoolean = v.extract::<bool>()?.into();
        space[key] = value;
    }
    Ok(space)
}
