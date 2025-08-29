use std::collections::HashSet;

use biodivine_lib_bdd::bdd;
use biodivine_lib_param_bn::{
    BooleanNetwork,
    biodivine_std::traits::Set,
    symbolic_async_graph::SymbolicAsyncGraph,
    trap_spaces::{NetworkColoredSpaces, SymbolicSpaceContext},
};
use log::info;
use macros::Configurable;

use crate::{
    debug_with_limit,
    internal::algorithms::{
        configurable::{Config as _, Configurable},
        fixed_points::{FixedPoints, FixedPointsConfig},
    },
    is_cancelled,
};

use super::{SymbolicSpaceContextExt as _, TrapSpacesConfig, TrapSpacesError};

const TARGET_ESSENTIAL_SYMBOLIC: &str = "TrapSpaces::essential_symbolic";
const TARGET_MINIMAL_SYMBOLIC: &str = "TrapSpaces::minimal_symbolic";
const TARGET_MINIMIZE: &str = "TrapSpaces::minimize";
const TARGET_MAXIMIZE: &str = "TrapSpaces::maximize";

#[derive(Clone, Configurable)]
pub struct TrapSpaces(TrapSpacesConfig);

impl From<(SymbolicAsyncGraph, SymbolicSpaceContext)> for TrapSpaces {
    /// Create a new "default" [TrapSpaces] from the given [SymbolicAsyncGraph] and
    /// [SymbolicSpaceContext].
    fn from((graph, ctx): (SymbolicAsyncGraph, SymbolicSpaceContext)) -> Self {
        TrapSpaces(TrapSpacesConfig::from((graph, ctx)))
    }
}

impl TryFrom<&BooleanNetwork> for TrapSpaces {
    type Error = TrapSpacesError;

    /// Create a new "default" [TrapSpaces] for the given [BooleanNetwork].
    fn try_from(boolean_network: &BooleanNetwork) -> Result<Self, Self::Error> {
        Ok(TrapSpaces(TrapSpacesConfig::try_from(boolean_network)?))
    }
}

impl TrapSpaces {
    /// Computes the coloured subset of "essential" trap spaces of a Boolean network.
    ///
    /// A trap space is essential if it cannot be reduced through percolation. In general, every
    /// minimal trap space is always essential.
    pub fn essential_symbolic(&self) -> Result<NetworkColoredSpaces, TrapSpacesError> {
        let ctx = &self.config().ctx;
        let graph = &self.config().graph;
        let restriction = &self.config().restriction;

        info!(
            target: TARGET_ESSENTIAL_SYMBOLIC,
            "Start symbolic essential trap space search with {}[nodes:{}] candidates.",
            restriction.approx_cardinality(),
            restriction.symbolic_size()
        );

        let bdd_ctx = ctx.bdd_variable_set();

        // We always start with the restriction set, because it should carry the information
        // about valid encoding of spaces.
        let mut to_merge = vec![restriction.as_bdd().clone()];
        let mut combined_bdd_size = restriction.as_bdd().size();
        for var in graph.variables() {
            if combined_bdd_size >= self.config().bdd_size_limit {
                return Err(TrapSpacesError::BddSizeLimitExceeded(
                    restriction.as_bdd().clone(),
                ));
            }

            let update_bdd = graph.get_symbolic_fn_update(var);
            let not_update_bdd = update_bdd.not();
            is_cancelled!(self, || restriction.as_bdd().clone())?;

            let has_up_transition = &ctx.mk_can_go_to_true_ext(update_bdd, self)?;
            is_cancelled!(self, || restriction.as_bdd().clone())?;

            let has_down_transition = &ctx.mk_can_go_to_true_ext(&not_update_bdd, self)?;
            is_cancelled!(self, || restriction.as_bdd().clone())?;

            let true_var = ctx.get_positive_variable(var);
            let false_var = ctx.get_negative_variable(var);

            let is_trap =
                bdd!(bdd_ctx, (has_up_transition => true_var) & (has_down_transition => false_var));
            is_cancelled!(self, || restriction.as_bdd().clone())?;

            let is_essential =
                bdd!(bdd_ctx, (true_var & false_var) => (has_up_transition & has_down_transition));
            is_cancelled!(self, || restriction.as_bdd().clone())?;

            debug_with_limit!(
                target: TARGET_ESSENTIAL_SYMBOLIC,
                size: is_trap.size() + is_essential.size(),
                " > Created initial sets for {:?} using {}+{} BDD nodes.",
                var,
                is_trap.size(),
                is_essential.size(),
            );

            let to_push = is_trap.and(&is_essential);
            combined_bdd_size += to_push.size();

            to_merge.push(to_push);
        }

        let trap_spaces = FixedPoints::with_config(
            FixedPointsConfig::from(graph.clone())
                .with_cancellation_nowrap(self.config().cancellation.clone())
                .with_bdd_size_limit(self.config().bdd_size_limit),
        )
        .symbolic_merge(to_merge, HashSet::new(), TARGET_ESSENTIAL_SYMBOLIC)?;

        let trap_spaces = NetworkColoredSpaces::new(trap_spaces, ctx);
        is_cancelled!(self, || trap_spaces.as_bdd().clone())?;

        info!(
            target: TARGET_ESSENTIAL_SYMBOLIC,
            "Found {}x{}[nodes:{}] essential trap spaces.",
            trap_spaces.colors().approx_cardinality(),
            trap_spaces.spaces().approx_cardinality(),
            trap_spaces.symbolic_size(),
        );

        Ok(trap_spaces)
    }

    /// Computes the minimal coloured trap spaces of the underlying `graph` within the configured
    /// `restriction` set.
    ///
    /// This method currently uses [Self::essential_symbolic], hence is always slower than
    /// this method.
    pub fn minimal_symbolic(&self) -> Result<NetworkColoredSpaces, TrapSpacesError> {
        info!(
            target: TARGET_MINIMAL_SYMBOLIC,
            "Start symbolic minimal trap space search."
        );

        self.essential_symbolic()
            .and_then(|essential| self.minimize(&essential))
    }

    /// Compute the minimal spaces within a particular subset.
    pub fn minimize(
        &self,
        spaces: &NetworkColoredSpaces,
    ) -> Result<NetworkColoredSpaces, TrapSpacesError> {
        let ctx = &self.config().ctx;

        let mut original = spaces.clone();
        let mut minimal = ctx.mk_empty_colored_spaces();

        info!(
            target: TARGET_MINIMIZE,
            "Start minimal subspace search with {}x{}[nodes:{}] candidates.",
            original.colors().approx_cardinality(),
            original.spaces().approx_cardinality(),
            original.symbolic_size()
        );

        while !original.is_empty() {
            if minimal.as_bdd().size() >= self.config().bdd_size_limit {
                return Err(TrapSpacesError::BddSizeLimitExceeded(
                    minimal.as_bdd().clone(),
                ));
            }

            // TODO:
            //  The pick-space process could probably be optimized somewhat to prioritize
            //  the most specific trap spaces (most "false" dual variables) instead of any
            //  just any trap space. On the other hand, the pick method already favors "lower"
            //  valuations, so there might not be that much space for improvement.

            // TODO:
            //  The other option would be to also consider sub-spaces and basically do something
            //  like normal attractor search, where next candidate is picked only from the
            //  sub-spaces of the original pick. This would guarantee that every iteration always
            //  discovers a minimal trap space, but it could just mean extra overhead if the
            //  "greedy" method using pick is good enough. Initial tests indicate that the
            //  greedy approach is enough.
            let minimum_candidate = original.pick_space();
            is_cancelled!(self, || minimal.as_bdd().clone())?;

            // Compute the set of strict super spaces.
            // TODO:
            //  This can take a long time if there are colors and a lot of traps, e.g.
            //  fixed-points, even though individual colors are easy. We should probably
            //  find a way to get rid of fixed points and any related super-spaces first,
            //  as these are clearly minimal. The other option would be to tune the super
            //  space enumeration to avoid spaces that are clearly irrelevant anyway.
            let super_spaces = ctx.mk_super_spaces_ext(minimum_candidate.as_bdd(), self)?;
            let super_spaces = NetworkColoredSpaces::new(super_spaces, ctx);
            is_cancelled!(self, || minimal.as_bdd().clone())?;

            original = original.minus(&super_spaces);
            minimal = minimal.minus(&super_spaces).union(&minimum_candidate);
            is_cancelled!(self, || minimal.as_bdd().clone())?;

            debug_with_limit!(
                target: TARGET_MINIMIZE,
                size: original.symbolic_size() + minimal.symbolic_size(),
                "Minimization in progress: {}x{}[nodes:{}] unprocessed, {}x{}[nodes:{}] candidates.",
                original.colors().approx_cardinality(),
                original.spaces().approx_cardinality(),
                original.symbolic_size(),
                minimal.colors().approx_cardinality(),
                minimal.spaces().approx_cardinality(),
                minimal.symbolic_size(),
            );
        }

        info!(
            target: TARGET_MINIMIZE,
            "Found {}[nodes:{}] minimal spaces.",
            minimal.approx_cardinality(),
            minimal.symbolic_size(),
        );

        Ok(minimal)
    }

    /// The same as [Self::minimize], but searches for maximal spaces within `spaces`.
    pub fn maximize(
        &self,
        spaces: &NetworkColoredSpaces,
    ) -> Result<NetworkColoredSpaces, TrapSpacesError> {
        let ctx = &self.config().ctx;

        let mut original = spaces.clone();
        let mut maximal = ctx.mk_empty_colored_spaces();

        info!(
            target: TARGET_MAXIMIZE,
            "Start maximal subspace search with {}x{}[nodes:{}] candidates.",
            original.colors().approx_cardinality(),
            original.spaces().approx_cardinality(),
            original.symbolic_size()
        );

        while !original.is_empty() {
            if maximal.as_bdd().size() >= self.config().bdd_size_limit {
                return Err(TrapSpacesError::BddSizeLimitExceeded(
                    maximal.as_bdd().clone(),
                ));
            }

            let maximum_candidate = original.pick_space();
            is_cancelled!(self, || maximal.as_bdd().clone())?;

            // Compute the set of strict sub spaces.
            let sub_spaces = ctx.mk_sub_spaces_ext(maximum_candidate.as_bdd(), self)?;
            let sub_spaces = NetworkColoredSpaces::new(sub_spaces, ctx);
            is_cancelled!(self, || maximal.as_bdd().clone())?;

            original = original.minus(&sub_spaces);
            maximal = maximal.minus(&sub_spaces).union(&maximum_candidate);
            is_cancelled!(self, || maximal.as_bdd().clone())?;

            debug_with_limit!(
                target: TARGET_MAXIMIZE,
                size: original.symbolic_size() + maximal.symbolic_size(),
                "Maximization in progress: {}x{}[nodes:{}] unprocessed, {}x{}[nodes:{}] candidates.",
                original.colors().approx_cardinality(),
                original.spaces().approx_cardinality(),
                original.symbolic_size(),
                maximal.colors().approx_cardinality(),
                maximal.spaces().approx_cardinality(),
                maximal.symbolic_size(),
            );
        }

        info!(
            target: TARGET_MAXIMIZE,
            "Found {}[nodes:{}] maximal spaces.",
            maximal.approx_cardinality(),
            maximal.symbolic_size(),
        );

        Ok(maximal)
    }
}
