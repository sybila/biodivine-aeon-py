use crate::bindings::lib_param_bn::{PyGraphColoredVertices, PyGraphColors, PySymbolicAsyncGraph};
use crate::internal::scc::algo_interleaved_transition_guided_reduction::interleaved_transition_guided_reduction;
use crate::internal::scc::{Behaviour, Classifier};
use crate::AsNative;
use pyo3::prelude::*;
use pyo3::types::PyDict;

pub(crate) fn register(module: &PyModule) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(classify_attractor, module)?)?;
    module.add_function(wrap_pyfunction!(find_attractors, module)?)?;
    module.add_function(wrap_pyfunction!(reach_bwd, module)?)?;
    module.add_function(wrap_pyfunction!(reach_fwd, module)?)?;
    module.add_function(wrap_pyfunction!(xie_beerel_attractors, module)?)?;
    module.add_function(wrap_pyfunction!(transition_guided_reduction, module)?)?;
    Ok(())
}

/// Remove states from the given `ColoredVertexSet` that are guaranteed to not appear in
/// any attractors.
///
/// The result is a colored set of states that may still contain some non-attractor states,
/// but should be significantly reduced.
#[pyfunction]
pub fn transition_guided_reduction(
    graph: &PySymbolicAsyncGraph,
    states: &PyGraphColoredVertices,
) -> PyGraphColoredVertices {
    let (states, _) =
        interleaved_transition_guided_reduction(graph.as_native(), states.as_native().clone());
    states.into()
}

/// Compute a list of `ColoredVertexSet` objects that together describe the attractors
/// of the given `SymbolicAsyncGraph`. Optionally, an initial `ColoredVertexSet` can be provided,
/// in which case the results are restricted to this set.
///
/// Note that the returned sets do not represent the attractors canonically. That is, there
/// are multiple ways of combining the attractors for different colors into a list of colored sets.
#[pyfunction]
pub fn xie_beerel_attractors(
    graph: &PySymbolicAsyncGraph,
    states: Option<&PyGraphColoredVertices>,
) -> Vec<PyGraphColoredVertices> {
    let variables = graph
        .as_native()
        .as_network()
        .variables()
        .collect::<Vec<_>>();
    let states = states
        .map(|it| it.clone())
        .unwrap_or_else(|| graph.unit_colored_vertices());
    let result = crate::internal::scc::algo_xie_beerel::xie_beerel_attractors(
        graph.as_native(),
        states.as_native(),
        &variables,
    );
    result.into_iter().map(|it| it.into()).collect()
}

/// Compute the `ColoredVertexSet` containing the forward reachable vertices from the given
/// initial `ColoredVertexSet`. Optionally, a "universe" `ColoredVertexSet` can be also provided,
/// in which case the search is restricted to this "universe" set.
#[pyfunction]
pub fn reach_fwd(
    graph: &PySymbolicAsyncGraph,
    states: &PyGraphColoredVertices,
    universe: Option<&PyGraphColoredVertices>,
) -> PyGraphColoredVertices {
    let variables = graph
        .as_native()
        .as_network()
        .variables()
        .collect::<Vec<_>>();
    let universe = universe
        .map(|it| it.clone())
        .unwrap_or_else(|| graph.unit_colored_vertices());

    crate::internal::scc::algo_saturated_reachability::reach_fwd(
        graph.as_native(),
        states.as_native(),
        universe.as_native(),
        &variables,
    )
    .into()
}

/// Compute the `ColoredVertexSet` containing the backward reachable vertices from the given
/// initial `ColoredVertexSet`. Optionally, a "universe" `ColoredVertexSet` can be also provided,
/// in which case the search is restricted to this "universe" set.
#[pyfunction]
pub fn reach_bwd(
    graph: &PySymbolicAsyncGraph,
    states: &PyGraphColoredVertices,
    universe: Option<&PyGraphColoredVertices>,
) -> PyGraphColoredVertices {
    let variables = graph
        .as_native()
        .as_network()
        .variables()
        .collect::<Vec<_>>();
    let universe = universe
        .map(|it| it.clone())
        .unwrap_or_else(|| graph.unit_colored_vertices());

    crate::internal::scc::algo_saturated_reachability::reach_bwd(
        graph.as_native(),
        states.as_native(),
        universe.as_native(),
        &variables,
    )
    .into()
}

/// Efficiently extract attractors of a particular `SymbolicAsyncGraph`.
///
/// Internally, this combines interleaved transition guided reduction and Xie-Beerel attractor
/// detection algorithm into a single procedure.
#[pyfunction]
pub fn find_attractors(graph: &PySymbolicAsyncGraph) -> Vec<PyGraphColoredVertices> {
    let (states, transitions) = interleaved_transition_guided_reduction(
        graph.as_native(),
        graph.as_native().mk_unit_colored_vertices(),
    );
    let result = crate::internal::scc::algo_xie_beerel::xie_beerel_attractors(
        graph.as_native(),
        &states,
        &transitions,
    );
    result.into_iter().map(|it| it.into()).collect()
}

/// Analyze a particular attractor (represented by a `ColoredVertexSet`) and separate its
/// associated `ColorSet` into three different subsets based on whether the attractor is
/// stable, oscillating or disordered for each color.
///
/// The result is a dictionary containing keys `stability`, `oscillation` and `disorder`
/// if the associated behavior was detected in the set.
#[pyfunction]
pub fn classify_attractor(
    py: Python,
    graph: &PySymbolicAsyncGraph,
    attractor: &PyGraphColoredVertices,
) -> PyResult<PyObject> {
    let mut classes = Classifier::classify_component(attractor.as_native(), graph.as_native());
    let result = PyDict::new(py);
    if let Some(stability) = classes.remove(&Behaviour::Stability) {
        let stability: PyGraphColors = stability.into();
        result.set_item("stability", stability.into_py(py))?;
    }
    if let Some(oscillation) = classes.remove(&Behaviour::Oscillation) {
        let oscillation: PyGraphColors = oscillation.into();
        result.set_item("oscillation", oscillation.into_py(py))?;
    }
    if let Some(disorder) = classes.remove(&Behaviour::Disorder) {
        let disorder: PyGraphColors = disorder.into();
        result.set_item("disorder", disorder.into_py(py))?;
    }

    Ok(result.to_object(py))
}
