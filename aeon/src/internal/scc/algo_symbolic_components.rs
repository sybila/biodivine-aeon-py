use crate::scc::algo_effectively_constant::{
    reach_saturated_bwd_excluding, reach_saturated_fwd_excluding,
    remove_effectively_constant_states,
};
use crate::scc::ProgressTracker;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use biodivine_lib_std::param_graph::Params;
use std::sync::atomic::AtomicBool;

pub fn components_2<F>(graph: &SymbolicAsyncGraph, on_component: F)
where
    F: Fn(GraphColoredVertices) -> () + Send + Sync,
{
    let mut universe = graph.unit_vertices().clone();

    let (constrained, variables) = remove_effectively_constant_states(graph, universe);
    universe = constrained.clone();

    while !universe.is_empty() {
        println!("Pick a new pivot branch...");
        let pivot = universe.pick_vertex();

        let bwd_pivot = reach_saturated_bwd_excluding(graph, &pivot, &universe, &variables);
        let component_with_pivot =
            reach_saturated_fwd_excluding(graph, &pivot, &bwd_pivot, &variables);
        let after_component = graph
            .post(&component_with_pivot)
            .minus(&component_with_pivot);
        let is_candidate = component_with_pivot
            .colors()
            .minus(&after_component.colors());
        if !is_candidate.is_empty() {
            on_component(component_with_pivot.intersect_colors(&is_candidate));
        }
        universe = universe.minus(&bwd_pivot);
    }
}

/*
/// Compute the SCCs that are induced by the vertices in `initial`.
///
/// Uses lockstep, so in terms of symbolic steps, this should be optimal.
fn components_with(graph: &SymbolicAsyncGraph, initial: &GraphColoredVertices) -> GraphColoredVertices {
    println!("Lockstep component search.");
    let mut still_running = initial.color_projection(graph);
    let mut fwd = initial.clone();
    let mut bwd = initial.clone();
    let mut fwd_frontier = initial.clone();
    let mut bwd_frontier = initial.clone();
    let mut fwd_unfinished = graph.empty_vertices().clone();
    let mut bwd_unfinished = graph.empty_vertices().clone();
    // First, find fwd/bwd templates using lockstep.
    while !still_running.is_empty() {
        println!("(1) Still running {}; {} {} {} {}", still_running.cardinality(), fwd.as_bdd().size(), bwd.as_bdd().size(), fwd_unfinished.as_bdd().size(), bwd_unfinished.as_bdd().size());
        let post = post(graph, &fwd_frontier).minus(&fwd);
        let pre = pre(graph, &bwd_frontier).minus(&bwd);
        // Compute colours that FINISHED in this iteration.
        let fwd_finished_now = fwd_frontier.color_projection(graph).minus(&post.color_projection(graph));
        let bwd_finished_now = bwd_frontier.color_projection(graph).minus(&pre.color_projection(graph));
        let bwd_finished_now = bwd_finished_now.minus(&fwd_finished_now); // Resolve ties.
        // Remove BWD-FINISHED colours from FWD frontier and move all vertices with these colours to unfinished.
        fwd_frontier = post.minus_colors(&bwd_finished_now);
        fwd_unfinished = fwd_unfinished.union(&fwd.intersect_colors(&bwd_finished_now));
        fwd = fwd.minus_colors(&bwd_finished_now).union(&fwd_frontier);
        // Remove FWD-FINISHED colours from BWD frontier and move all vertices with these colours to unfinished.
        bwd_frontier = pre.minus_colors(&fwd_finished_now);
        bwd_unfinished = bwd_unfinished.union(&bwd.intersect_colors(&fwd_finished_now));
        bwd = bwd.minus_colors(&fwd_finished_now).union(&bwd_frontier);
        // Mark finished colours as done.
        still_running = still_running.minus(&fwd_finished_now.union(&bwd_finished_now));
    }
    // Now fwd and bwd contain VALID forward/backward sets and are colour disjoint.
    // We have to continue computing unfinished within these valid bounds.
    assert!(fwd.color_projection(graph).intersect(&bwd.color_projection(graph)).is_empty());
    assert!(initial.color_projection(graph).minus(&fwd.color_projection(graph)).minus(&bwd.color_projection(graph)).is_empty());

    loop {
        let post = post(graph, &fwd_unfinished).minus(&fwd_unfinished).intersect(&bwd);
        let pre = pre(graph, &bwd_unfinished).minus(&bwd_unfinished).intersect(&fwd);
        fwd_unfinished = fwd_unfinished.union(&post);
        bwd_unfinished = bwd_unfinished.union(&pre);
        println!("(2) Still running {}", post.cardinality() + pre.cardinality());
        if post.is_empty() && pre.is_empty() {
            break;
        }
    }
    // At this point, fwd_unfinished and bwd_unfinished are completely done within the bounds of bwd/fwd.
    // We can therefore join them back
    fwd = fwd.union(&fwd_unfinished);
    bwd = bwd.union(&bwd_unfinished);

    // The final result is the intersection of the two.
    return fwd.intersect(&bwd);
}*/

pub fn components<F>(
    graph: &SymbolicAsyncGraph,
    _progress: &ProgressTracker,
    _cancelled: &AtomicBool,
    on_component: F,
) where
    F: Fn(GraphColoredVertices) -> () + Send + Sync,
{
    components_2(graph, on_component);
    /*
    crossbeam::thread::scope(|scope| {
        println!("Detect eventually stable...");
        // TODO: This is not correct, because for parametrisations can_move will never be empty...
        /*let mut without_fixed = graph.unit_vertices().clone();
        for variable in graph.network().graph().variable_ids() {
            let true_states = graph.state_variable_true(variable).intersect(&without_fixed);
            let false_states = graph.state_variable_false(variable).intersect(&without_fixed);
            let can_move_true = graph.has_any_post(variable, &true_states);
            let can_move_false = graph.has_any_post(variable, &false_states);
            if can_move_true.is_empty() {
                // Every transition for this variable is 0 -> 1, hence states that have this
                // transition enabled cannot be in attractor because it would never reverse...
                without_fixed = without_fixed.minus(&can_move_false)

                // At this point, we also know that the two sets (true states and false states)
                // are independent and can be processed in parallel! We should use that! TODO...
            }
            if can_move_false.is_empty() {
                without_fixed = without_fixed.minus(&can_move_true)
            }
        }
        println!("Fixed {}/{}", without_fixed.cardinality(), graph.unit_vertices().cardinality());*/

        println!("Start detecting sinks");


        let mut can_be_sink = graph.unit_vertices().clone();    // intentionally use all vertices
        //panic!("");
        for variable in graph.network().graph().variable_ids() {
            print!("{:?}...", variable);
            io::stdout().flush().unwrap();
            if cancelled.load(Ordering::SeqCst) {
                return ();
            }
            let has_successor = &graph.has_any_post(variable, graph.unit_vertices());
            can_be_sink = can_be_sink.minus(has_successor);
        }
        println!();

        let mut is_sink = can_be_sink.clone();
        /*for sink in is_sink.state_projection(graph).states(graph) {
            let mut valuations = Vec::new();
            for (i_v, v) in graph.network().graph().variable_ids().enumerate() {
                let name = graph.network().graph().get_variable(v).get_name();
                valuations.push((name.clone(), sink.get(i_v)));
            }
            let sink_colors = is_sink.intersect(&graph.vertex(sink.clone())).color_projection();
            let sink_remaining = is_sink.minus(&graph.vertex(sink.clone())).intersect_colors(&sink_colors);
            let sink_rank = if sink_remaining.is_empty() { 1 } else { 2 };

            println!("========================= Sink state (Rank {}) {:?} =========================", sink_rank, sink.values());
            println!("{:?}", valuations);
            println!("========================= Witness network =========================");
            let witness = graph.make_witness(&sink_colors);
            println!("{}", witness.to_string());
        }*/
        let sinks = is_sink.clone();
        // Now we have to report sinks, but we have to satisfy that every reported set has only one component:
        while !is_sink.is_empty() {
            let to_report = is_sink.pivots(graph);
            is_sink = is_sink.minus(&to_report);
            on_component(to_report);
        }

        println!("Sinks detected: {}", sinks.cardinality());

        /*let has_successors: Vec<GraphColoredVertices> = graph.network().graph().variable_ids()
            .collect::<Vec<VariableId>>()
            .into_par_iter()
            .map(|variable: VariableId| {
                graph.has_any_post(variable, graph.unit_vertices())
            })
            .collect();
        let has_successors = par_fold(has_successors, |a, b| a.union(b));*/

        let (not_constant, _) = remove_effectively_constant_states(graph, graph.unit_vertices().clone());
        println!("Not constant: {}/{}", not_constant.cardinality(), graph.unit_vertices().cardinality());

        if cancelled.load(Ordering::SeqCst) {
            return ();
        }

        let can_reach_sink =
            guarded_reach_bwd(&graph, &sinks, &not_constant, cancelled, progress);

        if cancelled.load(Ordering::SeqCst) {
            return ();
        }

        let initial = not_constant.minus(&can_reach_sink);

        println!("Initial: {}", initial.cardinality());

        if initial.is_empty() {
            return ();
        }

        let mut queue: Vec<(f64, GraphColoredVertices)> = Vec::new();
        queue.push((initial.cardinality(), initial));

        while let Some((universe_cardinality, universe)) = queue.pop() {
            if cancelled.load(Ordering::SeqCst) {
                return ();
            }

            println!(
                "Universe cardinality: {} Remaining work queue: {}",
                universe_cardinality,
                queue.len()
            );
            let remaining: f64 = queue.iter().map(|(a, _)| *a).sum::<f64>() + universe_cardinality;
            progress.update_remaining(remaining);
            println!("Look for pivots...");
            let pivots = universe.pivots(graph);
            let backward = guarded_reach_bwd(&graph, &pivots, &universe, cancelled, progress);
            let pivots = improve_pivots(graph, pivots.clone(), &(universe.minus(&backward).union(&pivots)));
            println!("Pivots cardinality: {}", pivots.cardinality());
            let backward = guarded_reach_bwd(&graph, &backward.union(&pivots), &universe, cancelled, progress);
            let component_with_pivots = guarded_reach_fwd(&graph, &pivots, &backward, cancelled, progress);

            let mut is_terminal = component_with_pivots.color_projection(graph);
            for v in graph.network().graph().variable_ids() {
                let successors = graph.any_post(v, &component_with_pivots).minus(&component_with_pivots).color_projection(graph);
                if !successors.is_empty() {
                    is_terminal = is_terminal.minus(&successors);
                }
            }

            if !is_terminal.is_empty() {
                let terminal = component_with_pivots.intersect_colors(&is_terminal);
                scope.spawn(|_| {
                    on_component(terminal);
                });
            }

            let remaining = universe.minus(&backward);
            if !remaining.is_empty() {
                queue.push((remaining.cardinality(), remaining));
            }

            /*
            let forward = guarded_reach_fwd(&graph, &pivots, &universe, cancelled, progress);

            if cancelled.load(Ordering::SeqCst) {
                return ();
            }

            let component_with_pivots =
                guarded_reach_bwd(&graph, &pivots, &forward, cancelled, progress);

            if cancelled.load(Ordering::SeqCst) {
                return ();
            }

            let reachable_terminals = forward.minus(&component_with_pivots);

            let leaves_current = reachable_terminals.color_projection();
            let is_terminal = graph.unit_colors().minus(&leaves_current);

            if !is_terminal.is_empty() {
                let terminal = component_with_pivots.intersect_colors(&is_terminal);
                scope.spawn(|_| {
                    on_component(terminal);
                });
            }

            let basins_of_reachable_terminals =
                guarded_reach_bwd(&graph, &forward, &universe, cancelled, progress);

            if cancelled.load(Ordering::SeqCst) {
                return ();
            }

            let unreachable_terminals = universe.minus(&basins_of_reachable_terminals);

            if !leaves_current.is_empty() {
                queue.push((reachable_terminals.cardinality(), reachable_terminals));
            }
            if !unreachable_terminals.is_empty() {
                queue.push((unreachable_terminals.cardinality(), unreachable_terminals));
            }*/
        }

        println!("Main component loop done...");
    })
    .unwrap();*/
}

/*fn improve_pivots(graph: &SymbolicAsyncGraph, pivots: GraphColoredVertices, universe: &GraphColoredVertices) -> GraphColoredVertices {
    let mut discovered = pivots.clone();
    let mut final_pivots = graph.empty_vertices().clone();
    let mut frontier = pivots.clone();
    println!("Pivot optimisation...");
    while !frontier.is_empty() {
        if discovered.as_bdd().size() > 10_000 {
            println!("{}/{} ({:+e}%, nodes result({}))",
                     discovered.cardinality(),
                     universe.cardinality(),
                     (discovered.cardinality() / universe.cardinality()) * 100.0,
                     discovered.as_bdd().size()
            );
        }
        let mut new_successor = graph.empty_vertices().clone();
        for v in graph.network().graph().variable_ids() {
            new_successor = new_successor.union(&graph.any_post(v, &frontier));
        }
        new_successor = new_successor.minus(&discovered);
        let stopped_in_this_step = frontier.color_projection(graph).minus(&new_successor.color_projection(graph));
        if !stopped_in_this_step.is_empty() {
            // For colours that stopped in this iteration
            final_pivots = final_pivots.union(&frontier.intersect_colors(&stopped_in_this_step).pivots(graph))
        }
        frontier = new_successor;
        discovered = discovered.union(&frontier);
    }
    return final_pivots;
}*/
