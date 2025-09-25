use std::collections::{HashMap, HashSet};

use biodivine_lib_bdd::{Bdd, BddVariable};
use biodivine_lib_param_bn::VariableId;
use pyo3::prelude::*;

use crate::AsNative;
use crate::bindings::lib_param_bn::symbolic::asynchronous_graph::AsynchronousGraph;
use crate::bindings::lib_param_bn::variable_id::VariableId as VariableIdBinding;

#[pyclass(module = "biodivine_aeon", frozen)]
pub struct Percolation {
    _dummy: (),
}

#[pymethods]
impl Percolation {
    /// **Deprecated**: Use `PecolationComp.percolate_subspace()` instead.
    /// Performs a percolation of a single subspace.
    ///
    /// Percolation propagates the values of variables that are guaranteed to be constant in the
    /// given subspace. Note that this function will not overwrite values fixed in the original
    /// space if they percolate to a conflicting value. Also note that the result is a subspace
    /// of the original space, i.e., it does not just contain the newly propagated variables.
    ///
    /// This method should technically work on parametrized networks as well, but the constant
    /// check is performed across all interpretations, hence a lot of subspaces will not
    /// percolate meaningfully. We recommend using other symbolic methods for such systems.
    #[staticmethod]
    pub fn percolate_subspace(
        py: Python,
        graph: &AsynchronousGraph,
        space: &Bound<'_, PyAny>,
    ) -> PyResult<HashMap<VariableIdBinding, bool>> {
        let native_graph = graph.as_native();
        let state_variables = native_graph.symbolic_context().state_variables();
        let mut network_variables: Vec<Option<VariableId>> = vec![
            None;
            native_graph
                .symbolic_context()
                .bdd_variable_set()
                .num_vars()
                as usize
        ];
        for var in native_graph.variables() {
            let bdd_var = state_variables[var.to_index()];
            network_variables[bdd_var.to_index()] = Some(var);
        }

        let initial_space = graph.resolve_subspace_valuation(space)?;

        // Variables that have a known fixed value.
        let mut fixed: Vec<Option<bool>> = vec![None; native_graph.num_vars()];
        for (var, v) in &initial_space {
            fixed[var.to_index()] = Some(*v);
        }

        let mut fns: Vec<Option<Bdd>> = vec![None; native_graph.num_vars()];
        let mut fn_inputs: Vec<Option<HashSet<BddVariable>>> = vec![None; native_graph.num_vars()];

        let mut restriction = Vec::new();

        let mut done = false;
        while !done {
            done = true;
            for i in 0..native_graph.num_vars() {
                if fixed[i].is_some() {
                    continue;
                }

                py.check_signals()?;

                if fns[i].is_none() {
                    let fn_bdd = native_graph.get_symbolic_fn_update(VariableId::from_index(i));
                    fns[i] = Some(fn_bdd.clone());
                }

                let fn_bdd = fns[i].as_mut().unwrap();

                let value = if fn_bdd.is_false() {
                    false
                } else if fn_bdd.is_true() {
                    true
                } else {
                    if fn_inputs[i].is_none() {
                        let inputs = fn_bdd.support_set();
                        fn_inputs[i] = Some(inputs);
                    }

                    let inputs = fn_inputs[i].as_mut().unwrap();

                    restriction.clear();
                    for input in inputs.clone() {
                        let Some(var) = network_variables[input.to_index()] else {
                            // This input corresponds to some network parameter. Parameters cannot
                            // be fixed by subspace percolation.
                            continue;
                        };
                        if let Some(value) = fixed[var.to_index()] {
                            restriction.push((input, value));
                            inputs.remove(&input);
                        }
                    }

                    if restriction.is_empty() {
                        continue;
                    }

                    *fn_bdd = fn_bdd.restrict(&restriction);
                    if fn_bdd.is_true() {
                        true
                    } else if fn_bdd.is_false() {
                        false
                    } else {
                        continue;
                    }
                };

                done = false;
                fixed[i] = Some(value);
            }
        }

        let mut result = HashMap::new();
        for (i, v) in fixed.iter().enumerate() {
            if let Some(v) = v.as_ref() {
                let var = VariableIdBinding::new(i);
                result.insert(var, *v);
            }
        }

        Ok(result)
    }
}
