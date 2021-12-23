use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use biodivine_lib_param_bn::VariableId;

/// This routine removes vertices which can never appear in an attractor by detecting parameter values
/// for which the variable jumps only in one direction.
///
/// If such one-directional jump is detected, then all states that can reach it are naturally
/// not in any attractor (since in an attractor, that jump would have to be reversed eventually).
///
/// Note that this does not mean the variable has to strictly always jump - that is why we need the
/// backward reachability to detect states that can actually achieve irreversible jump.
pub fn remove_effectively_constant_states(
    graph: &SymbolicAsyncGraph,
    set: GraphColoredVertices,
) -> (GraphColoredVertices, Vec<VariableId>) {
    println!("Remove effectively constant states.");
    let original_size = set.approx_cardinality();
    let mut universe = set;
    let mut stop = false;
    let mut variables: Vec<VariableId> = graph.network().variables().collect();
    while !stop {
        stop = true;
        let mut to_remove = graph.empty_vertices().clone();
        for variable in graph.network().variables() {
            let active_variables: Vec<VariableId> = variables
                .iter()
                .cloned()
                .filter(|it| *it != variable)
                .collect();
            let vertices_where_var_can_jump = graph.var_can_post(variable, &universe);
            let vertices_where_var_jumped = graph.var_post(variable, &universe);
            let reachable_after_jump = reach_saturated_fwd_excluding(
                graph,
                &vertices_where_var_jumped,
                &universe,
                &active_variables,
            );
            let can_jump_again = reachable_after_jump.intersect(&vertices_where_var_can_jump);
            let will_never_jump_again = vertices_where_var_can_jump.minus(&can_jump_again);
            if !will_never_jump_again.is_empty() {
                stop = false;
                let to_remove_for_var = reach_saturated_bwd_excluding(
                    graph,
                    &will_never_jump_again,
                    &universe,
                    &active_variables,
                );
                to_remove = to_remove.union(&to_remove_for_var);
                //universe = universe.minus(&to_remove); THIS IS A BAD IDEA...
                println!(
                    "{:?} will never jump again: {}",
                    variable,
                    will_never_jump_again.approx_cardinality()
                );
                println!(
                    "Eliminated {}/{} {:+e}%",
                    to_remove.approx_cardinality(),
                    universe.approx_cardinality(),
                    (to_remove.approx_cardinality() / universe.approx_cardinality()) * 100.0
                );
            }
        }
        universe = universe.minus(&to_remove);
        let original_vars = variables.len();
        variables = variables
            .into_iter()
            .filter(|v| !graph.var_can_post(*v, &universe).is_empty())
            .collect();
        println!(
            "Universe now has {} nodes. Eliminated {} variables.",
            universe.clone().into_bdd().size(),
            original_vars - variables.len()
        );
    }

    println!("Final active variables: {}", variables.len());
    println!(
        "Removed {}/{} {:+e}%; {} nodes.",
        universe.approx_cardinality(),
        original_size,
        (universe.approx_cardinality() / original_size) * 100.0,
        universe.clone().into_bdd().size()
    );

    for v in &variables {
        let vertices_where_var_can_jump = graph.var_can_post(*v, &universe);
        let reachable_before_jump = reach_saturated_bwd_excluding(
            graph,
            &vertices_where_var_can_jump,
            &universe,
            &variables,
        );
        let reachable_after_jump = reach_saturated_fwd_excluding(
            graph,
            &vertices_where_var_can_jump,
            &universe,
            &variables,
        );
        let components = reachable_before_jump.intersect(&reachable_after_jump);
        let below = reachable_after_jump.minus(&components);
        let can_reach_below =
            reach_saturated_bwd_excluding(graph, &below, &universe, &variables).minus(&below);
        println!(
            "({:?}) Below: {} Can reach below: {}",
            v,
            below.approx_cardinality(),
            can_reach_below.approx_cardinality()
        );
        universe = universe.minus(&can_reach_below);
    }

    println!("Final active variables: {}", variables.len());
    println!(
        "Removed {}/{} {:+e}%; {} nodes.",
        universe.approx_cardinality(),
        original_size,
        (universe.approx_cardinality() / original_size) * 100.0,
        universe.clone().into_bdd().size()
    );
    return (universe, variables);
}

pub fn reach_saturated_fwd_excluding(
    graph: &SymbolicAsyncGraph,
    initial: &GraphColoredVertices,
    guard: &GraphColoredVertices,
    variables: &Vec<VariableId>,
) -> GraphColoredVertices {
    if variables.is_empty() {
        return initial.clone();
    }
    let mut result = initial.clone();
    let last_variable = variables.len() - 1;
    let mut active_variable = last_variable;
    loop {
        let variable = variables[active_variable];
        let post = graph
            .var_post(variable, &result)
            .intersect(guard)
            .minus(&result);
        result = result.union(&post);

        if !post.is_empty() {
            active_variable = last_variable;
        } else {
            if active_variable == 0 {
                break;
            } else {
                active_variable -= 1;
            }
        }
    }
    return result;
}

pub fn reach_saturated_bwd_excluding(
    graph: &SymbolicAsyncGraph,
    initial: &GraphColoredVertices,
    guard: &GraphColoredVertices,
    variables: &Vec<VariableId>,
) -> GraphColoredVertices {
    if variables.is_empty() {
        return initial.clone();
    }
    let mut result = initial.clone();
    let last_variable = variables.len() - 1;
    let mut active_variable = last_variable;
    loop {
        let variable = variables[active_variable];
        let post = graph
            .var_pre(variable, &result)
            .intersect(guard)
            .minus(&result);
        result = result.union(&post);

        if !post.is_empty() {
            active_variable = last_variable;
        } else {
            if active_variable == 0 {
                break;
            } else {
                active_variable -= 1;
            }
        }
    }
    return result;
}
