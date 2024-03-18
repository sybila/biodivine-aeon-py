//! Interleaved transition guided reduction algorithm.
//!
//! Implements interleaved transition guided reduction. This technique does not remove
//! all non-attractor states, but can very significantly prune the state space in
//! a very reasonable amount of time.
//!

use crate::bindings::global_interrupt;
use crate::should_log;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use biodivine_lib_param_bn::VariableId;
use pyo3::PyResult;

mod _impl_extended_component_process;
mod _impl_fwd_bwd_process;
mod _impl_reachable_process;
mod _impl_scheduler;

/// Removes from `initial` as many non-attractor states as possible
/// using interleaved transition guided reduction.
///
/// It also returns a list of system variables for which there are still
/// transitions in the graph (other variables are effectively constant).
///
/// If cancelled, the result is still valid, but not necessarily complete.
pub fn interleaved_transition_guided_reduction(
    graph: &SymbolicAsyncGraph,
    initial: GraphColoredVertices,
    to_reduce: &[VariableId],
    log_level: usize,
) -> PyResult<(GraphColoredVertices, Vec<VariableId>)> {
    if should_log(log_level) {
        println!(
            "Start interleaved transition guided reduction with {}[nodes:{}] candidates.",
            initial.approx_cardinality(),
            initial.symbolic_size()
        );
    }

    let variables = graph.variables().collect::<Vec<_>>();
    let mut scheduler = Scheduler::new(initial, variables, log_level);
    for variable in to_reduce {
        global_interrupt()?;
        scheduler.spawn(ReachableProcess::new(
            *variable,
            graph,
            scheduler.get_universe().clone(),
        ));
    }

    while !scheduler.is_done() {
        global_interrupt()?;
        scheduler.step(graph)?;
    }

    let result = scheduler.finalize();

    if should_log(log_level) {
        println!(
            "Interleaved transition guided reduction finished with {}[nodes:{}] candidates.",
            result.0.approx_cardinality(),
            result.0.symbolic_size()
        );
    }

    Ok(result)
}

/// **(internal)** A process trait is a unit of work that is managed by a `Scheduler`.
/// Process has a *weight* that approximates how symbolically hard is to work with
/// its intermediate representation.
trait Process {
    /// Perform one step in the process. This can perform multiple symbolic operations,
    /// but should be fairly simple (i.e. does not need interrupting).
    ///
    /// If you still need to run a complex operation, you should check `GraphTaskContext`
    /// provided by `Scheduler` for cancellation.
    ///
    /// Returns true if the process cannot perform more steps.
    fn step(&mut self, scheduler: &mut Scheduler, graph: &SymbolicAsyncGraph) -> PyResult<bool>;

    /// Approximate symbolic complexity of the process.
    fn weight(&self) -> usize;

    /// Mark the given set of states as eliminated - i.e. they can be disregarded by this process.
    fn discard_states(&mut self, set: &GraphColoredVertices);
}

/// **(internal)** Scheduler manages work divided into `Processes`. It keeps a `universe`
/// of unprocessed vertices and a list of remaining active variables.
struct Scheduler {
    active_variables: Vec<VariableId>,
    universe: GraphColoredVertices,
    processes: Vec<(usize, Box<dyn Process>)>,
    to_discard: Option<GraphColoredVertices>,
    log_level: usize,
}

/// **(internal)** Basic backward reachability process.
struct BwdProcess {
    bwd: GraphColoredVertices,
    universe: GraphColoredVertices,
}

/// **(internal)** Basic forward reachability process.
struct FwdProcess {
    fwd: GraphColoredVertices,
    universe: GraphColoredVertices,
}

/// **(internal)** Computes the set of vertices reachable from states that can perform `var_post`.
///
/// When reachable set is computed, it automatically starts the extended component process.
struct ReachableProcess {
    variable: VariableId,
    fwd: FwdProcess,
}

/// **(internal)** Computes the extended component of a forward-reachable set.
struct ExtendedComponentProcess {
    variable: VariableId,
    fwd_set: GraphColoredVertices,
    bwd: BwdProcess,
}
