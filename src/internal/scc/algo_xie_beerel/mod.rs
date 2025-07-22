use crate::bindings::global_interrupt;
use crate::internal::scc::algo_saturated_reachability::{reach_bwd, reachability_step};
use crate::{log_essential, should_log};
use biodivine_lib_param_bn::VariableId;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use pyo3::PyResult;
use std::io::Write;

/// Uses a simplified Xie-Beerel algorithm adapted to coloured setting to find all bottom
/// SCCs in the given `universe` set. It only tests transitions using `active_variables`.
pub fn xie_beerel_attractors(
    graph: &SymbolicAsyncGraph,
    universe: &GraphColoredVertices,
    active_variables: &[VariableId],
    log_level: usize,
) -> PyResult<Vec<GraphColoredVertices>> {
    if should_log(log_level) {
        println!(
            "Start Xie-Beerel attractor detection on {}[nodes:{}] candidates.",
            universe.approx_cardinality(),
            universe.symbolic_size(),
        );
        std::io::stdout().lock().flush()?;
    }

    let mut universe = universe.clone();
    let mut result = Vec::new();
    while !universe.is_empty() {
        if log_essential(log_level, universe.symbolic_size()) {
            println!(
                " > Start new bottom SCC search. Remaining: {}[nodes:{}].",
                universe.approx_cardinality(),
                universe.symbolic_size()
            );
            std::io::stdout().lock().flush()?;
        }

        let pivots = universe.pick_vertex();

        let pivot_basin = reach_bwd(graph, &pivots, &universe, active_variables, log_level)?;

        let mut pivot_component = pivots.clone();

        // Iteratively compute the pivot component. If some color leaves `pivot_basin`, it is
        // removed from `pivot_component`, as it does not have to be processed anymore.
        //
        // At the end of the loop, `pivot_component` contains only colors for which the component
        // is an attractor (other colors will leave the `pivot_basin` at some point).
        loop {
            let done = reachability_step(
                &mut pivot_component,
                &universe,
                active_variables,
                |var, set| graph.var_post(var, set),
            )?;

            let problem_size = pivot_component.symbolic_size();
            if log_essential(log_level, problem_size) {
                let current = pivot_component.approx_cardinality();
                let max = universe.approx_cardinality();
                println!(
                    " >> [FWD process] Reachability progress: {}[nodes:{}] candidates ({:.2} log-%).",
                    current,
                    problem_size,
                    (current.log2() / max.log2()) * 100.0
                );
                std::io::stdout().lock().flush()?;
            }

            // This ensures `pivot_component` is still a subset of `pivot_basin` even if we do not
            // enforce it explicitly in `reachability_step`, since anything that leaks out
            // is eliminated.
            let escaped_basin = pivot_component.minus(&pivot_basin);
            if !escaped_basin.is_empty() {
                pivot_component = pivot_component.minus_colors(&escaped_basin.colors());
            }

            if done {
                break;
            }
        }

        if !pivot_component.is_empty() {
            if should_log(log_level) {
                println!(
                    " > Found a bottom SCC: {}x{}[nodes:{}].",
                    pivot_component.vertices().approx_cardinality(),
                    pivot_component.colors().approx_cardinality(),
                    pivot_component.symbolic_size(),
                );
                std::io::stdout().lock().flush()?;
            }
            result.push(pivot_component);
        }

        universe = universe.minus(&pivot_basin);
        global_interrupt()?;
    }

    if should_log(log_level) {
        println!(
            "Attractor detection finished with {} results.",
            result.len(),
        );
        std::io::stdout().lock().flush()?;
    }
    Ok(result)
}
