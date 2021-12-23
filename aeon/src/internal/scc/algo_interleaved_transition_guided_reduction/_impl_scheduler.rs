use crate::internal::scc::algo_interleaved_transition_guided_reduction::{Process, Scheduler};
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use biodivine_lib_param_bn::VariableId;

impl Scheduler {
    /// Create a new `Scheduler` with initial universe and active variables.
    pub fn new(initial: GraphColoredVertices, variables: Vec<VariableId>) -> Scheduler {
        Scheduler {
            active_variables: variables,
            universe: initial,
            processes: Vec::new(),
            to_discard: None,
        }
    }

    /// Finalize this scheduler, returning the current universe and active variables.
    pub fn finalize(self) -> (GraphColoredVertices, Vec<VariableId>) {
        (self.universe, self.active_variables)
    }

    /// Remove given `var` from the list of active variables.
    pub fn discard_variable(&mut self, var: VariableId) {
        self.active_variables
            .iter()
            .position(|v| *v == var)
            .into_iter()
            .for_each(|index| {
                self.active_variables.remove(index);
            });
    }

    /// Remove given `set` from the universe of this scheduler.
    pub fn discard_vertices(&mut self, set: &GraphColoredVertices) {
        self.universe = self.universe.minus(set);
        if let Some(to_discard) = self.to_discard.as_mut() {
            *to_discard = to_discard.union(set);
        } else {
            self.to_discard = Some(set.clone());
        }
    }

    /// Add a new process into this scheduler.
    pub fn spawn<P: 'static + Process>(&mut self, process: P) {
        self.processes.push((process.weight(), Box::new(process)));
    }

    /// Get the current universe set of the scheduler.
    pub fn get_universe(&self) -> &GraphColoredVertices {
        &self.universe
    }

    /// Get the list of currently active variables.
    pub fn get_active_variables(&self) -> &[VariableId] {
        &self.active_variables
    }

    /// True if all processes are finished.
    pub fn is_done(&self) -> bool {
        self.processes.is_empty()
    }

    /// If possible, perform one computational step for one of the processes.
    pub fn step(&mut self, graph: &SymbolicAsyncGraph) {
        if self.is_done() {
            return;
        }

        // First, apply to_discard in all processes:
        if let Some(to_discard) = self.to_discard.as_ref() {
            for (w, process) in self.processes.iter_mut() {
                process.discard_states(to_discard);
                *w = process.weight();
            }
            self.to_discard = None;
        }

        // Second, put the best process in the last place
        self.processes.sort_by_key(|(w, _)| usize::MAX - (*w));

        // Perform one step in a process
        if let Some((_, mut process)) = self.processes.pop() {
            let is_done = process.step(self, graph);
            if !is_done {
                self.processes.push((process.weight(), process))
            }
        }
    }
}
