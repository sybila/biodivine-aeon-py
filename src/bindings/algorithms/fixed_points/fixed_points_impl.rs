use std::collections::{HashMap, HashSet};

use biodivine_lib_bdd::{Bdd, BddVariable, BddVariableSet};
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
                fixed_points_config::FixedPointsConfig, fixed_points_error::FixedPointsError,
            },
        },
        lib_param_bn::symbolic::{
            asynchronous_graph::AsynchronousGraph, set_color::ColorSet,
            set_colored_vertex::ColoredVertexSet, set_vertex::VertexSet,
        },
    },
    is_cancelled,
};

const TARGET_NAIVE_SYMBOLIC: &str = "FixedPoints::naive_symbolic";
const TARGET_SYMBOLIC: &str = "FixedPoints::symbolic";
const TARGET_SYMBOLIC_VERTICES: &str = "FixedPoints::symbolic_vertices";
const TARGET_SYMBOLIC_COLORS: &str = "FixedPoints::symbolic_colors";

#[pyclass(module = "biodivine_aeon", frozen)]
pub struct FixedPoints(FixedPointsConfig);

impl FixedPoints {
    /// Create a new [FixedPoints] instance with the given [SymbolicAsyncGraph]
    /// and otherwise default configuration.
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

impl FixedPoints {
    // TODO: docs - document these methods using the lib_param_bn docs
    pub fn naive_symbolic(&self) -> Result<GraphColoredVertices, FixedPointsError> {
        self.config().start_timer();
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

                is_cancelled!(self.config())?;

                trace!(
                    target: TARGET_NAIVE_SYMBOLIC,
                    " > Created initial set for {:?} using {} BDD nodes.",
                    var,
                    is_stable.symbolic_size()
                );

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
            is_cancelled!(self.config())?;

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
        self.config().start_timer();
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

        is_cancelled!(self.config())?;

        let fixed_points = self.symbolic_merge(
            stg.symbolic_context().bdd_variable_set(),
            to_merge,
            HashSet::default(),
            TARGET_SYMBOLIC,
        )?;

        is_cancelled!(self.config())?;

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
        self.config().start_timer();
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

        is_cancelled!(self.config())?;

        let bdd = self.symbolic_merge(
            stg.symbolic_context().bdd_variable_set(),
            to_merge,
            projections,
            TARGET_SYMBOLIC_VERTICES,
        )?;

        let vertices = stg.empty_colored_vertices().vertices().copy(bdd);

        is_cancelled!(self.config())?;

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

        is_cancelled!(self.config())?;

        let bdd = self.symbolic_merge(
            stg.symbolic_context().bdd_variable_set(),
            to_merge,
            projections,
            TARGET_SYMBOLIC_COLORS,
        )?;

        let colors = stg.empty_colored_vertices().colors().copy(bdd);

        is_cancelled!(self.config())?;

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

                is_cancelled!(self.config())?;

                trace!(
                    target: target,
                    " > Created initial set for {:?} using {} BDD nodes.",
                    var,
                    is_stable.symbolic_size()
                );

                Ok(is_stable.into_bdd())
            })
            .collect::<Result<Vec<_>, FixedPointsError>>()?;

        Ok(result)
    }

    fn symbolic_merge(
        &self,
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

        is_cancelled!(self.config())?;

        // TODO: ohtenkay - deleted note to self

        while !to_merge.is_empty() || !projections.is_empty() {
            for p_var in projections.clone() {
                let dependencies = dependency_map.get(&p_var).unwrap();
                if dependencies.is_subset(&merged) {
                    result = result.var_exists(p_var);
                    projections.remove(&p_var);

                    is_cancelled!(self.config())?;

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

                is_cancelled!(self.config())?;

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

        is_cancelled!(self.config())?;

        info!(target: target, "Merge finished with {} BDD nodes.", result.size());

        // All projection variables are indeed gone.
        assert!(projections.is_empty());
        // And everything was merged.
        assert_eq!(merged.len(), support_sets.len());

        Ok(result)
    }
}

// TODO: finalize - make this optional with a feature flag
#[pymethods]
impl FixedPoints {
    /// Create a new [Reachability] instance with the given [AsynchronousGraph]
    /// and otherwise default configuration.
    #[staticmethod]
    #[pyo3(name = "with_graph")]
    pub fn with_graph_py(graph: Py<AsynchronousGraph>) -> Self {
        FixedPoints(FixedPointsConfig::with_graph_py(graph))
    }

    /// Create a new [Reachability] instance with the given [ReachabilityConfig].
    #[staticmethod]
    #[pyo3(name = "with_config")]
    pub fn with_config_py(config: Py<FixedPointsConfig>) -> Self {
        FixedPoints(config.get().clone())
    }

    #[pyo3(name = "naive_symbolic")]
    pub fn naive_symbolic_py(&self) -> PyResult<ColoredVertexSet> {
        let result_set = self.naive_symbolic()?;
        Ok(ColoredVertexSet::mk_native(
            self.config().symbolic_context(),
            result_set,
        ))
    }

    #[pyo3(name = "symbolic")]
    pub fn symbolic_py(&self) -> PyResult<ColoredVertexSet> {
        let result_set = self.symbolic()?;
        Ok(ColoredVertexSet::mk_native(
            self.config().symbolic_context(),
            result_set,
        ))
    }

    #[pyo3(name = "symbolic_vertices")]
    pub fn symbolic_vertices_py(&self) -> PyResult<VertexSet> {
        let result_set = self.symbolic_vertices()?;
        Ok(VertexSet::mk_native(
            self.config().symbolic_context(),
            result_set,
        ))
    }

    #[pyo3(name = "symbolic_colors")]
    pub fn symbolic_colors_py(&self) -> PyResult<ColorSet> {
        let result_set = self.symbolic_colors()?;
        Ok(ColorSet::mk_native(
            self.config().symbolic_context(),
            result_set,
        ))
    }
}
