use std::collections::{HashMap, HashSet};

use biodivine_lib_bdd::{Bdd, BddVariable};
use biodivine_lib_param_bn::{
    biodivine_std::traits::Set,
    symbolic_async_graph::{GraphColoredVertices, GraphColors, GraphVertices, SymbolicAsyncGraph},
};
use log::{debug, info, trace};
use pyo3::{pyclass, pymethods, Py, PyResult};

use crate::{
    bindings::{
        algorithms::{
            cancellation::CancellationHandler,
            fixed_points::{
                fixed_points_config::{FixedPointsConfig, FixedPointsConfigPython},
                fixed_points_error::FixedPointsError,
            },
        },
        lib_param_bn::symbolic::{
            asynchronous_graph::AsynchronousGraph, set_color::ColorSet,
            set_colored_vertex::ColoredVertexSet, set_vertex::VertexSet,
            symbolic_context::SymbolicContext,
        },
    },
    is_cancelled,
};

const TARGET_NAIVE_SYMBOLIC: &str = "FixedPoints::naive_symbolic";
const TARGET_SYMBOLIC: &str = "FixedPoints::symbolic";
const TARGET_SYMBOLIC_VERTICES: &str = "FixedPoints::symbolic_vertices";
const TARGET_SYMBOLIC_COLORS: &str = "FixedPoints::symbolic_colors";

#[derive(Clone)]
pub struct FixedPoints(FixedPointsConfig);

impl FixedPoints {
    /// Create a new [FixedPoints] instance with the given [SymbolicAsyncGraph]
    /// and otherwise default configuration.
    #[allow(dead_code)]
    pub fn with_graph(graph: SymbolicAsyncGraph) -> Self {
        FixedPoints(FixedPointsConfig::with_graph(graph))
    }

    /// Create a new [FixedPoints] instance with the given [FixedPointsConfig].
    pub fn with_config(config: FixedPointsConfig) -> Self {
        FixedPoints(config)
    }

    /// Retrieve the internal [FixedPointsConfig] of this instance.
    pub fn config(&self) -> &FixedPointsConfig {
        &self.0
    }
}

impl CancellationHandler for FixedPoints {
    fn is_cancelled(&self) -> bool {
        self.config().cancellation.is_cancelled()
    }

    fn start_timer(&self) {
        self.config().cancellation.start_timer()
    }
}

impl FixedPoints {
    // TODO: docs - document these methods using the lib_param_bn docs
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

        // TODO: discuss - is this correct
        let mut combined_bdd_size = 0;
        let mut to_merge: Vec<GraphColoredVertices> = stg
            .variables()
            .map(|var| {
                if combined_bdd_size > self.config().bdd_size_limit {
                    return Err(FixedPointsError::BddSizeLimitExceeded(
                        self.config().restriction.clone(),
                    ));
                }

                let can_step = stg.var_can_post(var, stg.unit_colored_vertices());
                let is_stable = restriction.minus(&can_step);

                is_cancelled!(self)?;

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

            // TODO: ohtenkay - is there a partial result in any of the algorithms?
            is_cancelled!(self)?;

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

        is_cancelled!(self)?;

        let fixed_points = self.symbolic_merge(to_merge, HashSet::default(), TARGET_SYMBOLIC)?;

        is_cancelled!(self)?;

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

        is_cancelled!(self)?;

        let bdd = self.symbolic_merge(to_merge, projections, TARGET_SYMBOLIC_VERTICES)?;

        let vertices = stg.empty_colored_vertices().vertices().copy(bdd);

        is_cancelled!(self)?;

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

        is_cancelled!(self)?;

        let bdd = self.symbolic_merge(to_merge, projections, TARGET_SYMBOLIC_COLORS)?;

        let colors = stg.empty_colored_vertices().colors().copy(bdd);

        is_cancelled!(self)?;

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

        let mut combined_bdd_size = 0;
        let result = stg
            .variables()
            .map(|var| {
                if combined_bdd_size > self.config().bdd_size_limit {
                    return Err(FixedPointsError::BddSizeLimitExceeded(
                        self.config().restriction.clone(),
                    ));
                }

                let can_step = stg.var_can_post(var, stg.unit_colored_vertices());
                let is_stable = stg.unit_colored_vertices().minus(&can_step);

                is_cancelled!(self)?;

                // TODO: discuss - if i change this to trace or disable logging, cancellation does
                // not happen
                // TODO: discuss - python does not have a TRACE level, so mbe use debug everywhere?
                debug!(
                    target: target,
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

    fn symbolic_merge(
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

        is_cancelled!(self)?;

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

                    is_cancelled!(self)?;

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

                is_cancelled!(self)?;

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
            // TODO: discuss - changed this to guard condition
            if best_result_size == usize::MAX {
                continue;
            }

            result = best_result;
            to_merge.remove(&best_index);
            merged.insert(best_index);

            let sum_to_merge_bdd_sizes = to_merge.values().map(|set| set.size()).sum::<usize>();
            if sum_to_merge_bdd_sizes + best_result_size > self.config().bdd_size_limit {
                return Err(FixedPointsError::BddSizeLimitExceeded(
                    self.config().graph.unit_colored_vertices().copy(result),
                ));
            }

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

        is_cancelled!(self)?;

        info!(target: target, "Merge finished with {} BDD nodes.", result.size());

        // All projection variables are indeed gone.
        assert!(projections.is_empty());
        // And everything was merged.
        assert_eq!(merged.len(), support_sets.len());

        Ok(result)
    }
}

// TODO: finalize - make this optional with a feature flag
#[pyclass(module = "biodivine_aeon", frozen)]
#[pyo3(name = "FixedPoints")]
pub struct FixedPointsPython {
    inner: FixedPoints,
    symbolic_context: Py<SymbolicContext>,
}

#[pymethods]
impl FixedPointsPython {
    /// Create a new [FixedPoints] instance with the given [AsynchronousGraph]
    /// and otherwise default configuration.
    #[staticmethod]
    pub fn with_graph(graph: Py<AsynchronousGraph>) -> Self {
        let config = FixedPointsConfigPython::with_graph(graph);

        FixedPointsPython {
            inner: FixedPoints(config.inner()),
            symbolic_context: config.symbolic_context(),
        }
    }

    /// Create a new [FixedPoints] instance with the given [FixedPointsConfig].
    #[staticmethod]
    pub fn with_config(config: Py<FixedPointsConfigPython>) -> Self {
        FixedPointsPython {
            inner: FixedPoints::with_config(config.get().inner()),
            symbolic_context: config.get().symbolic_context(),
        }
    }

    pub fn naive_symbolic(&self) -> PyResult<ColoredVertexSet> {
        let result_set = self.inner.naive_symbolic()?;

        Ok(ColoredVertexSet::mk_native(
            self.symbolic_context.clone(),
            result_set,
        ))
    }

    pub fn symbolic(&self) -> PyResult<ColoredVertexSet> {
        let result_set = self.inner.symbolic()?;

        Ok(ColoredVertexSet::mk_native(
            self.symbolic_context.clone(),
            result_set,
        ))
    }

    pub fn symbolic_vertices(&self) -> PyResult<VertexSet> {
        let result_set = self.inner.symbolic_vertices()?;
        Ok(VertexSet::mk_native(
            self.symbolic_context.clone(),
            result_set,
        ))
    }

    pub fn symbolic_colors(&self) -> PyResult<ColorSet> {
        let result_set = self.inner.symbolic_colors()?;
        Ok(ColorSet::mk_native(
            self.symbolic_context.clone(),
            result_set,
        ))
    }
}
