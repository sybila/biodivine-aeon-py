use biodivine_lib_bdd::Bdd;
use biodivine_lib_param_bn::{
    biodivine_std::traits::Set, symbolic_async_graph::GraphColoredVertices,
};
use log::{debug, info, trace};
use pyo3::pyclass;

use crate::bindings::algorithms::fixed_points::{
    fixed_points_config::FixedPointsConfig, fixed_points_error::FixedPointsError,
};

const TARGET_NAIVE_SYMBOLIC: &str = "FixedPoints::naive_symbolic";
const TARGET_SYMBOLIC: &str = "FixedPoints::symbolic";

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
    pub fn naive_symbolic(
        &self,
        restriction: &GraphColoredVertices,
    ) -> Result<GraphColoredVertices, FixedPointsError> {
        info!(
            target: TARGET_NAIVE_SYMBOLIC,
            "Started search with {}[nodes:{}] candidates.",
            restriction.approx_cardinality(),
            restriction.symbolic_size()
        );

        let stg = &self.config().graph;

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

    pub fn symbolic(
        &self,
        restriction: &GraphColoredVertices,
    ) -> Result<GraphColoredVertices, FixedPointsError> {
        info!(
            target: TARGET_SYMBOLIC,
            "Started search with {}[nodes:{}] candidates.",
            restriction.approx_cardinality(),
            restriction.symbolic_size()
        );

        let stg = &self.config().graph;
        let mut to_merge = self.prepare_to_merge(TARGET_SYMBOLIC)?;

        // Finally add the global requirement on the whole state space, if it is relevant.
        if !stg.unit_colored_vertices().is_subset(restriction) {
            to_merge.push(restriction.as_bdd().clone());
        }

        // interrupt()?;

        Ok(restriction.clone())
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
}
