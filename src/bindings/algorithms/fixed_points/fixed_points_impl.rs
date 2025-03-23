use std::collections::{HashMap, HashSet};

use biodivine_lib_bdd::{Bdd, BddVariable, BddVariableSet};
use biodivine_lib_param_bn::{
    biodivine_std::traits::Set,
    symbolic_async_graph::{GraphColoredVertices, GraphColors, GraphVertices},
};
use log::{debug, info, trace};
use pyo3::pyclass;

use crate::bindings::algorithms::fixed_points::{
    fixed_points_config::FixedPointsConfig, fixed_points_error::FixedPointsError,
};

const TARGET_NAIVE_SYMBOLIC: &str = "FixedPoints::naive_symbolic";
const TARGET_SYMBOLIC: &str = "FixedPoints::symbolic";
const TARGET_SYMBOLIC_VERTICES: &str = "FixedPoints::symbolic_vertices";
const TARGET_SYMBOLIC_COLORS: &str = "FixedPoints::symbolic_colors";

#[pyclass(module = "biodivine_aeon", frozen)]
pub struct FixedPoints(FixedPointsConfig);

impl FixedPoints {
    /// Retrieve the internal [FixedPointsConfig] of this instance.
    pub fn config(&self) -> &FixedPointsConfig {
        &self.0
    }
}

impl FixedPoints {
    // TODO: ohtenkay - document this, discuss whether to take the documentation from lib_param_bn
    pub fn naive_symbolic(&self) -> Result<GraphColoredVertices, FixedPointsError> {
        let stg = &self.config().graph;
        let restriction = &self.config().restriction;

        info!(
            target: TARGET_NAIVE_SYMBOLIC,
            "Started search with {}[nodes:{}] candidates.",
            restriction.approx_cardinality(),
            restriction.symbolic_size()
        );

        let mut to_merge: Vec<GraphColoredVertices> = stg
            .variables()
            .map(|var| {
                let can_step = stg.var_can_post(var, stg.unit_colored_vertices());
                let is_stable = restriction.minus(&can_step);

                // interrupt()?;

                trace!(
                    target: TARGET_NAIVE_SYMBOLIC,
                    " > Created initial set for {:?} using {} BDD nodes.",
                    var,
                    is_stable.symbolic_size()
                );

                is_stable
            })
            .collect();

        while to_merge.len() > 1 {
            to_merge.sort_by_key(|it| -(it.symbolic_size() as isize));

            // TODO: ohtenkay - ask what should be debug and what slould be trace
            debug!(
                target: TARGET_NAIVE_SYMBOLIC,
                " > Merging {} sets using {} BDD nodes.",
                to_merge.len(),
                to_merge.iter().map(|it| it.symbolic_size()).sum::<usize>(),
            );

            // interrupt()?;

            let x = to_merge.pop().unwrap();
            let y = to_merge.pop().unwrap();
            to_merge.push(x.intersect(&y));
        }

        // TODO: ohtenkay - discuss this error
        let Some(fixed_points) = to_merge.pop() else {
            info!(
                target: TARGET_NAIVE_SYMBOLIC,
                "No fixed points found using {} BDD nodes.",
                restriction.symbolic_size()
            );
            return Err(FixedPointsError::NoFixedPointsFound);
        };

        info!(
            target: TARGET_NAIVE_SYMBOLIC,
            "Found {}[nodes:{}] fixed-points.",
            fixed_points.approx_cardinality(),
            fixed_points.symbolic_size(),
        );

        Ok(fixed_points)
    }

    pub fn symbolic(&self) -> Result<GraphColoredVertices, FixedPointsError> {
        let stg = &self.config().graph;
        let restriction = &self.config().restriction;

        info!(
            target: TARGET_SYMBOLIC,
            "Started search with {}[nodes:{}] candidates.",
            restriction.approx_cardinality(),
            restriction.symbolic_size()
        );

        let mut to_merge = self.prepare_to_merge(TARGET_SYMBOLIC)?;

        // TODO: ohtenkay - deleted note to self

        // Finally add the global requirement on the whole state space, if it is relevant.
        if !stg.unit_colored_vertices().is_subset(restriction) {
            to_merge.push(restriction.as_bdd().clone());
        }

        // interrupt()?;

        let fixed_points = Self::symbolic_merge(
            stg.symbolic_context().bdd_variable_set(),
            to_merge,
            HashSet::default(),
            TARGET_SYMBOLIC,
        )?;

        // interrupt()?;

        let fixed_points = stg.unit_colored_vertices().copy(fixed_points);

        info!(
            target: TARGET_SYMBOLIC,
            "Found {}[nodes:{}] fixed-points.",
            fixed_points.approx_cardinality(),
            fixed_points.symbolic_size(),
        );

        Ok(fixed_points)
    }

    pub fn symbolic_vertices(&self) -> Result<GraphVertices, FixedPointsError> {
        let stg = &self.config().graph;
        let restriction = &self.config().restriction;

        info!(
            target: TARGET_SYMBOLIC_VERTICES,
            "Started search with {}[nodes:{}] candidates.",
            restriction.approx_cardinality(),
            restriction.symbolic_size()
        );

        let mut to_merge = self.prepare_to_merge(TARGET_SYMBOLIC_VERTICES)?;

        // Finally add the global requirement on the whole state space, if it is relevant.
        if !stg.unit_colored_vertices().is_subset(restriction) {
            to_merge.push(restriction.as_bdd().clone());
        }

        let projections: HashSet<BddVariable> = stg
            .symbolic_context()
            .parameter_variables()
            .iter()
            .cloned()
            .collect();

        // interrupt()?;

        let bdd = Self::symbolic_merge(
            stg.symbolic_context().bdd_variable_set(),
            to_merge,
            projections,
            TARGET_SYMBOLIC_VERTICES,
        )?;

        let vertices = stg.empty_colored_vertices().vertices().copy(bdd);

        // interrupt()?;

        info!(
            target: TARGET_SYMBOLIC_VERTICES,
            "Found {}[nodes:{}] fixed-point vertices.",
            vertices.approx_cardinality(),
            vertices.symbolic_size(),
        );

        Ok(vertices)
    }

    pub fn symbolic_colors(&self) -> Result<GraphColors, FixedPointsError> {
        let stg = &self.config().graph;
        let restriction = &self.config().restriction;

        info!(
            target: TARGET_SYMBOLIC_COLORS,
            "Started search with {}[nodes:{}] candidates.",
            restriction.approx_cardinality(),
            restriction.symbolic_size()
        );

        let mut to_merge = self.prepare_to_merge(TARGET_SYMBOLIC_COLORS)?;

        // Finally add the global requirement on the whole state space, if it is relevant.
        if !stg.unit_colored_vertices().is_subset(restriction) {
            to_merge.push(restriction.as_bdd().clone());
        }

        let projections: HashSet<BddVariable> = stg
            .symbolic_context()
            .state_variables()
            .iter()
            .cloned()
            .collect();

        // interrupt()?;

        let bdd = Self::symbolic_merge(
            stg.symbolic_context().bdd_variable_set(),
            to_merge,
            projections,
            TARGET_SYMBOLIC_COLORS,
        )?;

        let colors = stg.empty_colored_vertices().colors().copy(bdd);

        // interrupt()?;

        info!(
            target: TARGET_SYMBOLIC_COLORS,
            "Found {}[nodes:{}] fixed-point colors.",
            colors.approx_cardinality(),
            colors.symbolic_size(),
        );

        Ok(colors)
    }
}

impl FixedPoints {
    fn prepare_to_merge(&self, target: &str) -> Result<Vec<Bdd>, FixedPointsError> {
        let stg = &self.config().graph;

        let result = stg
            .variables()
            .map(|var| {
                let can_step = stg.var_can_post(var, stg.unit_colored_vertices());
                let is_stable = stg.unit_colored_vertices().minus(&can_step);

                // interrupt()?;

                trace!(
                    target: target,
                    " > Created initial set for {:?} using {} BDD nodes.",
                    var,
                    is_stable.symbolic_size()
                );

                is_stable.into_bdd()
            })
            .collect();

        Ok(result)
    }

    pub fn symbolic_merge(
        // TODO: ohtenkay - discuss this argument being removed and the function taking &self instead
        universe: &BddVariableSet,
        to_merge: Vec<Bdd>,
        // The set of variables that will be eliminated from the result.
        mut projections: HashSet<BddVariable>,
        target: &str,
    ) -> Result<Bdd, FixedPointsError> {
        // First, assign each merge item a unique integer identifier.
        let mut to_merge: HashMap<usize, Bdd> = to_merge.into_iter().enumerate().collect();

        // And compute the support set for each of them.
        let support_sets: HashMap<usize, HashSet<BddVariable>> = to_merge
            .iter()
            .map(|(id, set)| (*id, set.support_set()))
            .collect();

        // Now, for every projection variable, determine which merge items depend on said variable.
        // Once all of these items appear in the final result, the variable can be removed.
        let dependency_map: HashMap<BddVariable, HashSet<usize>> = projections
            .iter()
            .map(|bdd_var| {
                let dependencies = support_sets
                    .iter()
                    .filter(|(_, set)| set.contains(bdd_var))
                    .map(|(id, _)| *id)
                    .collect::<HashSet<usize>>();
                (*bdd_var, dependencies)
            })
            .collect();

        let mut result = universe.mk_true();
        let mut merged = HashSet::new();

        // interrupt()?;

        // TODO: ohtenkay - deleted note to self

        while !to_merge.is_empty() || !projections.is_empty() {
            for p_var in projections.clone() {
                let dependencies = dependency_map.get(&p_var).unwrap();
                if dependencies.is_subset(&merged) {
                    result = result.var_exists(p_var);
                    projections.remove(&p_var);

                    // interrupt()?;

                    trace!(
                        target: target,
                        " > Projection. New result has {} BDD nodes. Remaining projections: {}.",
                        result.size(),
                        projections.len()
                    );
                }
            }

            let mut best_result = universe.mk_true();
            let mut best_result_size = usize::MAX;
            let mut best_index = 0;

            // Ensure deterministic iteration of results.
            let mut merge_iter: Vec<usize> = to_merge.keys().cloned().collect();
            merge_iter.sort_by_cached_key(|it| to_merge[it].size());
            merge_iter.reverse();

            for i in merge_iter {
                let set = &to_merge[&i];
                let bdd = Bdd::binary_op_with_limit(
                    best_result_size,
                    set,
                    &result,
                    biodivine_lib_bdd::op_function::and,
                );

                // interrupt()?;

                if let Some(bdd) = bdd {
                    // At this point, the size of the BDD should be smaller or equal to the
                    // best result, so we can just update it.
                    assert!(bdd.size() <= best_result_size);
                    best_result = bdd;
                    best_result_size = best_result.size();
                    best_index = i;
                }
            }

            // This may not be true in the last iteration if the only thing left to do
            // are projections.
            if best_result_size != usize::MAX {
                result = best_result;
                to_merge.remove(&best_index);
                merged.insert(best_index);

                if result.is_false() {
                    return Ok(universe.mk_false());
                }

                trace!(
                    target: target,
                    " > Merge. New result has {} BDD nodes. Remaining constraints: {}.",
                    result.size(),
                    to_merge.len(),
                );
            }
        }

        // interrupt()?;

        info!(target: target, "Merge finished with {} BDD nodes.", result.size());

        // All projection variables are indeed gone.
        assert!(projections.is_empty());
        // And everything was merged.
        assert_eq!(merged.len(), support_sets.len());

        Ok(result)
    }
}
