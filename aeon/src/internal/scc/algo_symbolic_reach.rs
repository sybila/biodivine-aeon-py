use crate::scc::ProgressTracker;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use std::io;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};

pub fn guarded_reach_fwd(
    graph: &SymbolicAsyncGraph,
    initial: &GraphColoredVertices,
    guard: &GraphColoredVertices,
    cancelled: &AtomicBool,
    progress: &ProgressTracker,
) -> GraphColoredVertices {
    /*let mut result = initial.clone();
    let mut frontier = initial.clone();

    println!("Reach fwd...");
    while !frontier.is_empty() {
        if cancelled.load(Ordering::SeqCst) {
            return result; // result is incorrect, but we are cancelled so we don't care...
        }

        progress.update_last_wave(frontier.cardinality());

        println!("{}/{} ({:+e}%, nodes result({}), frontier({}))",
                 result.cardinality(),
                 guard.cardinality(),
                 (result.cardinality()/guard.cardinality()) * 100.0,
                 result.clone().into_bdd().size(),
                 frontier.clone().into_bdd().size()
        );
        print!("{} || ", frontier.cardinality());
        let mut successors = graph.empty_vertices().clone();
        for variable in graph.network().graph().variable_ids() {
            print!("{:?}...", variable);
            io::stdout().flush().unwrap();
            if cancelled.load(Ordering::SeqCst) {
                return result; // result is incorrect, but we are cancelled so we don't care...
            }
            successors = successors.union(&graph.post(variable, &frontier, guard));
        }
        successors = successors.minus(&result);
        result = result.union(&successors);
        println!();
        frontier = successors;
    }

    progress.update_last_wave(0.0);
    return result;*/
    let mut result = initial.clone();

    println!("Reach fwd...");
    loop {
        if cancelled.load(Ordering::SeqCst) {
            return result; // result is incorrect, but we are cancelled so we don't care...
        }

        progress.update_last_wave(result.cardinality());

        println!(
            "{}/{} ({:+e}%, nodes result({}))",
            result.cardinality(),
            guard.cardinality(),
            (result.cardinality() / guard.cardinality()) * 100.0,
            result.clone().into_bdd().size()
        );
        let mut successors = graph.empty_vertices().clone();
        for variable in graph.network().graph().variable_ids() {
            io::stdout().flush().unwrap();
            if cancelled.load(Ordering::SeqCst) {
                return result; // result is incorrect, but we are cancelled so we don't care...
            }
            let s = graph.post(variable, &result, guard);
            successors = successors.union(&s);
            /*if !s.is_empty() {
                println!("{:?} -> {}", variable, s.into_bdd().size());
            }*/
        }
        print!(" || {}", successors.clone().into_bdd().size());
        println!();
        successors = successors.minus(&result);
        if successors.is_empty() {
            break;
        }
        result = result.union(&successors);
    }

    progress.update_last_wave(0.0);
    return result;
}

pub fn guarded_reach_bwd(
    graph: &SymbolicAsyncGraph,
    initial: &GraphColoredVertices,
    guard: &GraphColoredVertices,
    cancelled: &AtomicBool,
    progress: &ProgressTracker,
) -> GraphColoredVertices {
    /*let mut result = initial.clone();
    let mut frontier = initial.clone();

    println!("Reach bwd...");
    while !frontier.is_empty() {
        if cancelled.load(Ordering::SeqCst) {
            return result; // result is incorrect, but we are cancelled so we don't care...
        }

        progress.update_last_wave(frontier.cardinality());

        println!("{}/{} ({:+e}%, nodes result({}), frontier({}))",
                 result.cardinality(),
                 guard.cardinality(),
                 (result.cardinality()/guard.cardinality()) * 100.0,
                 result.clone().into_bdd().size(),
                 frontier.clone().into_bdd().size()
        );
        print!("{} || ", frontier.cardinality());
        /*let var_predecessors: Vec<GraphColoredVertices> = graph.network().graph().variable_ids().collect::<Vec<VariableId>>()
            .into_par_iter()
            .map(|variable| {
                let mut predecessors = graph.empty_vertices().clone();
                let mut sub_frontier = graph.pre(variable, &frontier, guard);
                while !sub_frontier.is_empty() {
                    sub_frontier = sub_frontier.minus(&predecessors);
                    predecessors = predecessors.union(&sub_frontier);
                    sub_frontier = graph.pre(variable, &sub_frontier, guard);
                }
                predecessors
            })
            .collect();
        let predecessors = par_fold(var_predecessors, |a, b| a.union(b));
        frontier = predecessors.minus(&result);
        result = result.union(&predecessors);*/
        let mut predecessors = graph.empty_vertices().clone();
        for variable in graph.network().graph().variable_ids() {
            print!("{:?}...", variable);
            io::stdout().flush().unwrap();
            if cancelled.load(Ordering::SeqCst) {
                return result; // result is incorrect, but we are cancelled so we don't care...
            }
            predecessors = predecessors.union(&graph.pre(variable, &frontier, guard));
        }
        predecessors = predecessors.minus(&result);
        result = result.union(&predecessors);
        println!();
        frontier = predecessors;
    }

    progress.update_last_wave(0.0);*/
    let mut result = initial.clone();

    println!("Reach bwd...");
    loop {
        if cancelled.load(Ordering::SeqCst) {
            return result; // result is incorrect, but we are cancelled so we don't care...
        }

        progress.update_last_wave(result.cardinality());

        println!(
            "{}/{} ({:+e}%, nodes result({}))",
            result.cardinality(),
            guard.cardinality(),
            (result.cardinality() / guard.cardinality()) * 100.0,
            result.clone().into_bdd().size()
        );
        let mut predecessors = graph.empty_vertices().clone();
        for variable in graph.network().graph().variable_ids() {
            io::stdout().flush().unwrap();
            if cancelled.load(Ordering::SeqCst) {
                return result; // result is incorrect, but we are cancelled so we don't care...
            }
            let s = graph.pre(variable, &result, guard);
            predecessors = predecessors.union(&s);
            /*if !s.is_empty() {
                println!("{:?} -> {}", variable, s.into_bdd().size());
            }*/
        }
        print!(" || {}", predecessors.clone().into_bdd().size());
        println!();
        predecessors = predecessors.minus(&result);
        if predecessors.is_empty() {
            break;
        }
        result = result.union(&predecessors);
    }

    progress.update_last_wave(0.0);
    return result;
}
