use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use biodivine_lib_param_bn::VariableId;

/// Performs one reachability step using the saturation scheme.
///
/// The `universe` is an upper bound on what elements can be added to the `set`. Using `variables`
/// you can restrict the considered transitions. Finally, `step` implements update in one
/// variable.
///
/// Returns `true` if fixpoint has been reached.
pub fn reachability_step<F>(
    set: &mut GraphColoredVertices,
    universe: &GraphColoredVertices,
    variables: &[VariableId],
    step: F,
) -> bool
where
    F: Fn(VariableId, &GraphColoredVertices) -> GraphColoredVertices,
{
    if variables.is_empty() {
        return true;
    }
    for var in variables.iter().rev() {
        let stepped = step(*var, set).minus(set).intersect(universe);

        if !stepped.is_empty() {
            *set = set.union(&stepped);
            return false;
        }
    }
    true
}

/// Fully compute reachable states from `initial` inside `universe` using transitions under
/// `variables`.
///
/// The process is cancellable using the `GraphTaskContext`, in which case the result is valid,
/// but not complete.
pub fn reach_fwd(
    graph: &SymbolicAsyncGraph,
    initial: &GraphColoredVertices,
    universe: &GraphColoredVertices,
    variables: &[VariableId],
) -> GraphColoredVertices {
    let mut set = initial.clone();
    loop {
        if reachability_step(&mut set, universe, variables, |v, s| graph.var_post(v, s)) {
            break;
        }
    }
    set
}

/// Fully compute back-reachable states from `initial` inside `universe` using transitions under
/// `variables`.
///
/// The process is cancellable using the `GraphTaskContext`, in which case the result is valid,
/// but not complete.
pub fn reach_bwd(
    graph: &SymbolicAsyncGraph,
    initial: &GraphColoredVertices,
    universe: &GraphColoredVertices,
    variables: &[VariableId],
) -> GraphColoredVertices {
    let mut set = initial.clone();
    loop {
        if reachability_step(&mut set, universe, variables, |v, s| graph.var_pre(v, s)) {
            break;
        }
    }
    set
}
