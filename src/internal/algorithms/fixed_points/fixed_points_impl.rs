use std::collections::{HashMap, HashSet};

use biodivine_lib_bdd::{Bdd, BddVariable};
use biodivine_lib_param_bn::{
    biodivine_std::traits::Set,
    symbolic_async_graph::{GraphColoredVertices, GraphColors, GraphVertices, SymbolicAsyncGraph},
    BooleanNetwork,
};
use log::{debug, info, trace};
use macros::Configurable;

use crate::{
    debug_with_limit,
    internal::algorithms::{cancellation::CancellationHandler, configurable::Configurable},
    is_cancelled,
};

use super::{FixedPointsConfig, FixedPointsError};

const TARGET_NAIVE_SYMBOLIC: &str = "FixedPoints::naive_symbolic";
const TARGET_SYMBOLIC: &str = "FixedPoints::symbolic";
const TARGET_SYMBOLIC_VERTICES: &str = "FixedPoints::symbolic_vertices";
const TARGET_SYMBOLIC_COLORS: &str = "FixedPoints::symbolic_colors";

/// Implements fixed-point search over a [SymbolicAsyncGraph].
///
/// See [FixedPointsConfig] and [FixedPointsError] for more info.
#[derive(Clone, Configurable)]
pub struct FixedPoints(FixedPointsConfig);

impl From<SymbolicAsyncGraph> for FixedPoints {
    /// Create a new [FixedPoints] instance with the given [SymbolicAsyncGraph]
    /// and otherwise default configuration.
    fn from(graph: SymbolicAsyncGraph) -> Self {
        FixedPoints(FixedPointsConfig::from(graph))
    }
}

impl TryFrom<&BooleanNetwork> for FixedPoints {
    type Error = FixedPointsError;

    /// Create a new [FixedPoints] instance with the given [BooleanNetwork]
    /// and otherwise default configuration.
    fn try_from(boolean_network: &BooleanNetwork) -> Result<Self, Self::Error> {
        Ok(FixedPoints(FixedPointsConfig::try_from(boolean_network)?))
    }
}

impl FixedPoints {
    /// A naive symbolic algorithm that computes the fixed points by gradual elimination of
    /// all states with outgoing transitions.
    ///
    /// Only fixed-points from the `restriction` set are returned. However, the state has to
    /// be a *global* fixed point, not just a fixed-point within the `restriction` set.
    ///
    /// **Characteristics:** As the name suggests, this algorithm is not really suited for
    /// processing complex networks. However, we provide it as a "baseline" for testing other
    /// algorithms. In theory, due to its simplicity, it could be faster on some of the smaller
    /// networks where the symbolic explosion is not severe.
    pub fn naive_symbolic(&self) -> Result<GraphColoredVertices, FixedPointsError> {
        self.start_timer();
        let stg = &self.config().graph;
        let restriction = &self.config().restriction;

        info!(
            target: TARGET_NAIVE_SYMBOLIC,
            "Started search with {}[nodes:{}] candidates.",
            restriction.approx_cardinality(),
            restriction.symbolic_size()
        );

        let mut combined_bdd_size = 0;
        let mut to_merge: Vec<GraphColoredVertices> = stg
            .variables()
            .map(|var| {
                if combined_bdd_size > self.config().bdd_size_limit {
                    return Err(FixedPointsError::BddSizeLimitExceeded(
                        restriction.as_bdd().clone(),
                    ));
                }

                let can_step = stg.var_can_post(var, stg.unit_colored_vertices());
                let is_stable = restriction.minus(&can_step);

                is_cancelled!(self, || { restriction.as_bdd().clone() })?;

                trace!(
                    target: TARGET_NAIVE_SYMBOLIC,
                    " > Created initial set for {:?} using {} BDD nodes.",
                    var,
                    is_stable.symbolic_size()
                );

                combined_bdd_size += is_stable.as_bdd().size();
                Ok(is_stable)
            })
            .collect::<Result<Vec<_>, FixedPointsError>>()?;

        while to_merge.len() > 1 {
            to_merge.sort_by_key(|it| -(it.symbolic_size() as isize));

            debug!(
                target: TARGET_NAIVE_SYMBOLIC,
                " > Merging {} sets using {} BDD nodes.",
                to_merge.len(),
                to_merge.iter().map(|it| it.symbolic_size()).sum::<usize>(),
            );

            is_cancelled!(self, || { to_merge.pop().unwrap().as_bdd().clone() })?;

            let x = to_merge.pop().unwrap();
            let y = to_merge.pop().unwrap();
            to_merge.push(x.intersect(&y));
        }

        let fixed_points = to_merge
            .pop()
            .expect("SymbolicAsyncGraph has no variables, invalid state.");

        info!(
            target: TARGET_NAIVE_SYMBOLIC,
            "Found {}[nodes:{}] fixed-points.",
            fixed_points.approx_cardinality(),
            fixed_points.symbolic_size(),
        );

        Ok(fixed_points)
    }

    /// A better version of the [Self::naive_symbolic] algorithm that can actually scale to
    /// reasonably sized networks (e.g. 100-200 variables + parameters).
    ///
    /// Only fixed-points from the `restriction` set are returned. However, the state has to
    /// be a *global* fixed point, not just a fixed-point within the `restriction` set.
    ///
    /// **Characteristics:** Instead of merging individual constraints one by one, this algorithm
    /// greedily selects the constraint that leads to the smallest intermediate BDD. This requires
    /// more symbolic operations, but can work better as the intermediate BDDs tend to be smaller.
    ///
    /// In particular, this tends to work better for cases where no fixed points exist, because
    /// the process will often quickly find a combination on constraints that prevent
    /// the fixed point from existing (whereas for [Self::naive_symbolic], this process is much more
    /// random).
    ///
    /// Also note that the algorithm can benefit from parallelization, but we do not implement
    /// it here, as it is a bit problematic to implement in a platform-independent manner.
    ///
    /// You can often scale the algorithm to very large networks as well, but the hardest
    /// bottleneck seems to be the total number of fixed points. As such, if the network is large
    /// (e.g. 1000 variables) but has only a few fixed points, it can often still be solved by this
    /// method. However, if there are many parameters (e.g. >50) and the number of fixed points
    /// is proportional to the number of parameters, you will be bounded by the inherent
    /// combinatorial complexity of the resulting set of states.
    pub fn symbolic(&self) -> Result<GraphColoredVertices, FixedPointsError> {
        self.start_timer();
        let stg = &self.config().graph;
        let restriction = &self.config().restriction;

        info!(
            target: TARGET_SYMBOLIC,
            "Started search with {}[nodes:{}] candidates.",
            restriction.approx_cardinality(),
            restriction.symbolic_size()
        );

        let mut to_merge = self.prepare_to_merge(TARGET_SYMBOLIC)?;

        /*
           Note to self: There is actually a marginally faster version of this algorithm that
           does not throw away the intermediate results but instead carries them over to the
           next iteration. Nevertheless, this version also wastes much more memory, as all
           results have to be preserved, so I ultimately decided not to use it.
        */

        // Finally add the global requirement on the whole state space, if it is relevant.
        if !stg.unit_colored_vertices().is_subset(restriction) {
            to_merge.push(restriction.as_bdd().clone());
        }

        is_cancelled!(self, || { to_merge.pop().unwrap().clone() })?;

        let fixed_points = self.symbolic_merge(to_merge, HashSet::default(), TARGET_SYMBOLIC)?;

        is_cancelled!(self, || { fixed_points.clone() })?;

        let fixed_points = stg.unit_colored_vertices().copy(fixed_points);

        info!(
            target: TARGET_SYMBOLIC,
            "Found {}[nodes:{}] fixed-points.",
            fixed_points.approx_cardinality(),
            fixed_points.symbolic_size(),
        );

        Ok(fixed_points)
    }

    /// The result of the function are all vertices that can appear as fixed-points for **some**
    /// parameter valuation. That is, for every returned vertex, there is at least one color
    /// for which the vertex is a fixed-point.
    ///
    /// **Characteristics:** If the network has no parameters, the result is equivalent to the
    /// result of [Self::symbolic]. However, if there are parameters in the network, the result
    /// is often much smaller and can be computed much faster.
    ///
    /// In particular, it is possible to use the result (or subsets of the result) as `restriction`
    /// sets for [Self::symbolic] to obtain the full information later. In some cases, this is also
    /// faster than running [Self::symbolic] directly.
    ///
    /// If you are only interested in certain combinations of variables within the fixed-points,
    /// you can also write a custom projection query using [Self::symbolic_merge].
    pub fn symbolic_vertices(&self) -> Result<GraphVertices, FixedPointsError> {
        self.start_timer();
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

        is_cancelled!(self, || { to_merge.pop().unwrap().clone() })?;

        let bdd = self.symbolic_merge(to_merge, projections, TARGET_SYMBOLIC_VERTICES)?;

        is_cancelled!(self, || { bdd.clone() })?;

        let vertices = stg.empty_colored_vertices().vertices().copy(bdd);

        info!(
            target: TARGET_SYMBOLIC_VERTICES,
            "Found {}[nodes:{}] fixed-point vertices.",
            vertices.approx_cardinality(),
            vertices.symbolic_size(),
        );

        Ok(vertices)
    }

    /// Similar to [Self::symbolic_vertices], but only returns colors for which there exists
    /// at least one fixed-point within `restriction`.
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

        is_cancelled!(self, || { to_merge.pop().unwrap().clone() })?;

        let bdd = self.symbolic_merge(to_merge, projections, TARGET_SYMBOLIC_COLORS)?;

        is_cancelled!(self, || { bdd.clone() })?;

        let colors = stg.empty_colored_vertices().colors().copy(bdd);

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
        let restriction = &self.config().restriction;

        let mut combined_bdd_size = 0;
        let result = stg
            .variables()
            .map(|var| {
                if combined_bdd_size > self.config().bdd_size_limit {
                    return Err(FixedPointsError::BddSizeLimitExceeded(
                        restriction.as_bdd().clone(),
                    ));
                }

                let can_step = stg.var_can_post(var, stg.unit_colored_vertices());
                let is_stable = stg.unit_colored_vertices().minus(&can_step);

                is_cancelled!(self, || { restriction.as_bdd().clone() })?;

                debug_with_limit!(
                    target: target,
                    size: is_stable.symbolic_size(),
                    " > Created initial set for {:?} using {} BDD nodes.",
                    var,
                    is_stable.symbolic_size()
                );

                combined_bdd_size += is_stable.as_bdd().size();
                Ok(is_stable.into_bdd())
            })
            .collect::<Result<Vec<_>, FixedPointsError>>()?;

        Ok(result)
    }

    /// This is a helper method that is used by [Self::symbolic], [Self::symbolic_vertices] and
    /// [Self::symbolic_colors].
    ///
    /// It greedily performs a conjunction of the given BDDs, but eliminates the symbolic
    /// variables given in `projections`. Using this method, you can implement arbitrary projected
    /// fixed-point detection. However, the method is inherently unsafe because currently
    /// there is no way to give a type-safe result for operations other than `symbolic_vertices`
    /// and `symbolic_colors`, so it is up to you to understand whether the result is
    /// actually what you wanted.
    pub(crate) fn symbolic_merge(
        &self,
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

        let universe = self.config().graph.symbolic_context().bdd_variable_set();
        let mut result = universe.mk_true();
        let mut merged = HashSet::new();

        is_cancelled!(self, || { result.clone() })?;

        /*
           Note to self: It seems that not all projections are always beneficial to the BDD size.
           At the same time, a non-optimal merge may enable a very useful projection. It is
           not entirely clear how to greedily apply these two observations. Ideally, we'd like
           to prioritize merges that lead to projections, but this is not universally true.

           Maybe we could at least greedily prefer merging sets that will immediately lead to
           projections? But even this is not an entirely clear win.
        */

        while !to_merge.is_empty() || !projections.is_empty() {
            for p_var in projections.clone() {
                let dependencies = dependency_map.get(&p_var).unwrap();
                if dependencies.is_subset(&merged) {
                    result = result.var_exists(p_var);
                    projections.remove(&p_var);

                    is_cancelled!(self, || { result.clone() })?;

                    debug_with_limit!(
                        target: target,
                        size: result.size(),
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

                is_cancelled!(self, || { result.clone() })?;

                if let Some(bdd) = bdd {
                    // At this point, the size of the BDD should be smaller or equal to the
                    // best result, so we can just update it.
                    assert!(bdd.size() <= best_result_size);
                    best_result = bdd;
                    best_result_size = best_result.size();
                    best_index = i;
                }
            }

            // This may be true in the last iteration if the only thing left to do
            // are projections.
            if best_result_size == usize::MAX {
                continue;
            }

            result = best_result;
            to_merge.remove(&best_index);
            merged.insert(best_index);

            let sum_to_merge_bdd_sizes = to_merge.values().map(|set| set.size()).sum::<usize>();
            if sum_to_merge_bdd_sizes + best_result_size > self.config().bdd_size_limit {
                return Err(FixedPointsError::BddSizeLimitExceeded(result));
            }

            if result.is_false() {
                return Ok(universe.mk_false());
            }

            debug_with_limit!(
                target: target,
                size: result.size(),
                " > Merge. New result has {} BDD nodes. Remaining constraints: {}.",
                result.size(),
                to_merge.len(),
            );
        }

        is_cancelled!(self, || { result.clone() })?;

        info!(target: target, "Merge finished with {} BDD nodes.", result.size());

        // All projection variables are indeed gone.
        assert!(projections.is_empty());
        // And everything was merged.
        assert_eq!(merged.len(), support_sets.len());

        Ok(result)
    }
}
