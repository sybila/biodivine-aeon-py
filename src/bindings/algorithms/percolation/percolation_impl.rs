use std::collections::HashSet;

use biodivine_lib_bdd::{Bdd, BddVariable};
use biodivine_lib_param_bn::{
    symbolic_async_graph::SymbolicAsyncGraph, BooleanNetwork, VariableId,
};
use log::{debug, info, trace};
use macros::Configurable;
use pyo3::pyclass;

use crate::{
    bindings::algorithms::{cancellation::CancellationHandler, configurable::Configurable},
    is_cancelled,
};

use super::{PercolationConfig, PercolationError};

const TARGET_PERCOLATE_SUBSPACE: &str = "Percolation::percolate_subspace";

/// Implements subspace percolation over a [SymbolicAsyncGraph].
///
/// See [PercolationConfig] and [PercolationError] for more info.
#[pyclass(module = "biodivine_aeon", frozen)]
#[pyo3(name = "PercolationComp")]
#[derive(Clone, Configurable)]
pub struct Percolation(pub PercolationConfig);

impl From<SymbolicAsyncGraph> for Percolation {
    /// Create a new [Percolation] instance from the given [SymbolicAsyncGraph]
    /// and otherwise default configuration.
    fn from(graph: SymbolicAsyncGraph) -> Self {
        Percolation(PercolationConfig::from(graph))
    }
}

impl TryFrom<&BooleanNetwork> for Percolation {
    type Error = PercolationError;

    /// Create a new [Percolation] instance from the given [BooleanNetwork]
    /// and otherwise default configuration.
    fn try_from(boolean_network: &BooleanNetwork) -> Result<Self, Self::Error> {
        Ok(Percolation(PercolationConfig::try_from(boolean_network)?))
    }
}

impl Percolation {
    /// Performs a percolation of a single subspace.
    ///
    /// Percolation propagates the values of variables that are guaranteed to be constant in the
    /// given subspace. Note that this function will not overwrite values fixed in the original
    /// space if they percolate to a conflicting value. Also note that the result is a subspace
    /// of the original space, i.e. it does not just contain the newly propagated variables.
    ///
    /// This method should technically work on parametrized networks as well, but the constant
    /// check is performed across all interpretations, hence a lot of sub-spaces will not
    /// percolate meaningfully. We recommend using other symbolic methods for such systems.
    pub fn percolate_subspace(
        &self,
        subspace: Vec<(VariableId, bool)>,
    ) -> Result<Vec<(VariableId, bool)>, PercolationError> {
        self.start_timer();
        info!(
            target: TARGET_PERCOLATE_SUBSPACE,
            "Started with {} variables in the subspace.",
            subspace.len()
        );

        let graph = &self.config().graph;

        let mut network_variables = vec![
            VariableId::from_index(0);
            graph.symbolic_context().bdd_variable_set().num_vars()
                as usize
        ];

        let state_variables = graph.symbolic_context().state_variables();
        for var in graph.variables() {
            let bdd_var = state_variables[var.to_index()];
            network_variables[bdd_var.to_index()] = var;
        }

        // Variables that have a known fixed value.
        let mut fixed: Vec<Option<bool>> = vec![None; graph.num_vars()];
        for (var, v) in &subspace {
            fixed[var.to_index()] = Some(*v);
        }

        let mut fns: Vec<Option<Bdd>> = vec![None; graph.num_vars()];
        let mut fn_inputs: Vec<Option<HashSet<BddVariable>>> = vec![None; graph.num_vars()];

        let mut restriction = Vec::new();

        let mut done = false;
        while !done {
            debug!(
                target: TARGET_PERCOLATE_SUBSPACE,
                "Currently found {} fixed variables.",
                fixed.iter().filter(|v| v.is_some()).count()
            );

            done = true;
            for i in 0..graph.num_vars() {
                if fixed[i].is_some() {
                    continue;
                }

                is_cancelled!(self, || { fixed.clone() })?;

                if fns[i].is_none() {
                    let fn_bdd = graph.get_symbolic_fn_update(VariableId::from_index(i));
                    fns[i] = Some(fn_bdd.clone());
                }

                let fn_bdd = fns[i].as_mut().unwrap();

                trace!(
                    target: TARGET_PERCOLATE_SUBSPACE,
                    "Checking fn_bdd with index {} and value: ",
                    i,
                );

                let value = match (fn_bdd.is_true(), fn_bdd.is_false()) {
                    (true, _) => true,
                    (_, true) => false,
                    _ => {
                        if fn_inputs[i].is_none() {
                            let inputs = fn_bdd.support_set();
                            fn_inputs[i] = Some(inputs);
                        }

                        let inputs = fn_inputs[i].as_mut().unwrap();

                        restriction.clear();
                        for input in inputs.clone() {
                            let var = network_variables[input.to_index()];
                            if let Some(value) = fixed[var.to_index()] {
                                restriction.push((input, value));
                                inputs.remove(&input);
                            }
                        }

                        if restriction.is_empty() {
                            trace!(target: TARGET_PERCOLATE_SUBSPACE, " > skipped");
                            continue;
                        }

                        *fn_bdd = fn_bdd.restrict(&restriction);
                        match (fn_bdd.is_true(), fn_bdd.is_false()) {
                            (true, _) => true,
                            (_, true) => false,
                            _ => {
                                trace!(target: TARGET_PERCOLATE_SUBSPACE, " > skipped");
                                continue;
                            }
                        }
                    }
                };

                trace!(target: TARGET_PERCOLATE_SUBSPACE, " > {}", value);

                done = false;
                fixed[i] = Some(value);
            }
        }

        let result: Vec<_> = fixed
            .iter()
            .enumerate()
            .filter_map(|(i, v)| {
                if let Some(v) = v.as_ref() {
                    let var = VariableId::from_index(i);
                    Some((var, *v))
                } else {
                    None
                }
            })
            .collect();

        info!(target: TARGET_PERCOLATE_SUBSPACE, "Done. Result: {} fixed variables.", result.len());
        Ok(result)
    }
}
