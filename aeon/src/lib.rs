use crate::internal::scc::{Behaviour, Classifier};
use biodivine_bdd::{Bdd, BddVariable, BddVariableSet, BddVariableSetBuilder, BooleanExpression};
use biodivine_boolean_networks::{
    BooleanNetwork, ColorSet, ColoredVertexSet, ParameterId, RegulatoryGraph, SymbolicAsyncGraph,
    VariableId, VertexSet,
};
use pyo3::prelude::*;
use pyo3::types::PyDict;

mod internal;

/// Remove states from the given set that are guaranteed to not contain any attractors.
///
/// The result is a set of states that way still contain some non-attractor states, but should be
/// largely pruned.
#[pyfunction]
pub fn transition_guided_reduction(
    graph: &SymbolicAsyncGraph,
    states: &ColoredVertexSet,
) -> ColoredVertexSet {
    let (states, _) = internal::scc::algo_interleaved_transition_guided_reduction::interleaved_transition_guided_reduction(
        &graph.0, states.0.clone()
    );
    states.into()
}

/// Compute the attractors from the given set using the xie-beerel algorithm.
#[pyfunction]
pub fn xie_beerel_attractors(
    graph: &SymbolicAsyncGraph,
    states: &ColoredVertexSet,
) -> Vec<ColoredVertexSet> {
    let variables = graph.0.as_network().variables().collect::<Vec<_>>();
    let result =
        internal::scc::algo_xie_beerel::xie_beerel_attractors(&graph.0, &states.0, &variables);
    result.into_iter().map(|it| it.into()).collect()
}

/// Compute the forward reachable vertices. If a universe is given, the search is restricted to
/// this subset.
#[pyfunction]
pub fn reach_fwd(
    graph: &SymbolicAsyncGraph,
    states: &ColoredVertexSet,
    universe: Option<&ColoredVertexSet>,
) -> ColoredVertexSet {
    let variables = graph.0.as_network().variables().collect::<Vec<_>>();
    if let Some(universe) = universe.as_ref() {
        internal::scc::algo_saturated_reachability::reach_fwd(
            &graph.0,
            &states.0,
            &universe.0,
            &variables,
        )
        .into()
    } else {
        internal::scc::algo_saturated_reachability::reach_fwd(
            &graph.0,
            &states.0,
            graph.0.unit_colored_vertices(),
            &variables,
        )
        .into()
    }
}

/// Compute the backward reachable vertices. If a universe is given, the search is restricted to
/// this subset.
#[pyfunction]
pub fn reach_bwd(
    graph: &SymbolicAsyncGraph,
    states: &ColoredVertexSet,
    universe: Option<&ColoredVertexSet>,
) -> ColoredVertexSet {
    let variables = graph.0.as_network().variables().collect::<Vec<_>>();
    if let Some(universe) = universe {
        internal::scc::algo_saturated_reachability::reach_bwd(
            &graph.0,
            &states.0,
            &universe.0,
            &variables,
        )
        .into()
    } else {
        internal::scc::algo_saturated_reachability::reach_bwd(
            &graph.0,
            &states.0,
            graph.0.unit_colored_vertices(),
            &variables,
        )
        .into()
    }
}

/// Find attractors of a particular graph.
#[pyfunction]
pub fn find_attractors(graph: &SymbolicAsyncGraph) -> Vec<ColoredVertexSet> {
    let (states, transitions) = internal::scc::algo_interleaved_transition_guided_reduction::interleaved_transition_guided_reduction(
        &graph.0, graph.0.mk_unit_colored_vertices()
    );
    let result =
        internal::scc::algo_xie_beerel::xie_beerel_attractors(&graph.0, &states, &transitions);
    result.into_iter().map(|it| it.into()).collect()
}

/// Analyze a particular attractor and separate its color set into different classes
/// based on whether the attractor is stable, oscillating or disordered.
#[pyfunction]
pub fn classify_attractor(
    py: Python,
    graph: &SymbolicAsyncGraph,
    attractor: &ColoredVertexSet,
) -> PyResult<PyObject> {
    let mut classes = Classifier::classify_component(&attractor.0, &graph.0);
    let result = PyDict::new(py);
    if let Some(stability) = classes.remove(&Behaviour::Stability) {
        let stability: ColorSet = stability.into();
        result.set_item("stability", stability.into_py(py))?;
    }
    if let Some(oscillation) = classes.remove(&Behaviour::Oscillation) {
        let oscillation: ColorSet = oscillation.into();
        result.set_item("oscillation", oscillation.into_py(py))?;
    }
    if let Some(disorder) = classes.remove(&Behaviour::Disorder) {
        let disorder: ColorSet = disorder.into();
        result.set_item("disorder", disorder.into_py(py))?;
    }

    Ok(result.to_object(py))
}

/// A Python module implemented in Rust.
#[pymodule]
fn biodivine_aeon(_py: Python, module: &PyModule) -> PyResult<()> {
    module.add_class::<VariableId>()?;
    module.add_class::<ParameterId>()?;
    module.add_class::<RegulatoryGraph>()?;
    module.add_class::<BooleanNetwork>()?;
    module.add_class::<SymbolicAsyncGraph>()?;
    module.add_class::<ColorSet>()?;
    module.add_class::<VertexSet>()?;
    module.add_class::<ColoredVertexSet>()?;
    // Re-export everything here as well, because the types are incompatible in Python :/
    module.add_class::<Bdd>()?;
    module.add_class::<BddVariable>()?;
    module.add_class::<BddVariableSet>()?;
    module.add_class::<BddVariableSetBuilder>()?;
    module.add_class::<BooleanExpression>()?;
    module.add_function(wrap_pyfunction!(classify_attractor, module)?)?;
    module.add_function(wrap_pyfunction!(find_attractors, module)?)?;
    module.add_function(wrap_pyfunction!(reach_bwd, module)?)?;
    module.add_function(wrap_pyfunction!(reach_fwd, module)?)?;
    module.add_function(wrap_pyfunction!(xie_beerel_attractors, module)?)?;
    module.add_function(wrap_pyfunction!(transition_guided_reduction, module)?)?;
    Ok(())
}
