use std::collections::HashSet;

use biodivine_lib_bdd::bdd;
use biodivine_lib_param_bn::{
    biodivine_std::traits::Set,
    symbolic_async_graph::SymbolicAsyncGraph,
    trap_spaces::{NetworkColoredSpaces, SymbolicSpaceContext},
    BooleanNetwork,
};
use log::{debug, info};
use pyo3::{pyclass, pymethods, Py, PyResult, Python};

use crate::{
    bindings::{
        algorithms::{
            configurable::{Config as _, Configurable},
            fixed_points::{
                fixed_points_config::FixedPointsConfig, fixed_points_impl::FixedPoints,
            },
            trap_spaces::{
                trap_spaces_config::{PyTrapSpacesConfig, TrapSpacesConfig},
                trap_spaces_error::TrapSpacesError,
            },
        },
        lib_param_bn::{
            boolean_network::BooleanNetwork as BooleanNetworkBinding,
            symbolic::{
                asynchronous_graph::AsynchronousGraph, set_colored_space::ColoredSpaceSet,
                symbolic_space_context::SymbolicSpaceContext as SymbolicSpaceContextBinding,
            },
        },
    },
    is_cancelled, AsNative as _,
};

#[derive(Clone)]
pub struct TrapSpaces(TrapSpacesConfig);

impl Configurable for TrapSpaces {
    type ConfigType = TrapSpacesConfig;

    /// Retrieve the internal [TrapSpacesConfig] of this instance.
    fn config(&self) -> &Self::ConfigType {
        &self.0
    }

    /// Create a new [TrapSpaces] instance with the given [TrapSpacesConfig].
    fn with_config(config: Self::ConfigType) -> Self {
        TrapSpaces(config)
    }
}

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
            "Start symbolic essential trap space search with {}[nodes:{}] candidates.",
            restriction.approx_cardinality(),
            restriction.symbolic_size()
        );

        // TODO: discuss - what to do with this, it has to be passed to symbolic merge, but it was
        // removed from the arguments
        let bdd_ctx = ctx.bdd_variable_set();

        // We always start with the restriction set, because it should carry the information
        // about valid encoding of spaces.
        let mut to_merge = vec![restriction.as_bdd().clone()];
        for var in graph.variables() {
            let update_bdd = graph.get_symbolic_fn_update(var);
            let not_update_bdd = update_bdd.not();
            // is_cancelled!(self)?;

            // TODO: ohtenkay - rewrite _mk_can_go_to_true
            let has_up_transition = &ctx.mk_can_go_to_true(update_bdd);
            // is_cancelled!(self)?;

            // TODO: ohtenkay - rewrite _mk_can_go_to_true
            let has_down_transition = &ctx.mk_can_go_to_true(&not_update_bdd);
            // is_cancelled!(self)?;

            let true_var = ctx.get_positive_variable(var);
            let false_var = ctx.get_negative_variable(var);

            let is_trap =
                bdd!(bdd_ctx, (has_up_transition => true_var) & (has_down_transition => false_var));
            // is_cancelled!(self)?;

            let is_essential =
                bdd!(bdd_ctx, (true_var & false_var) => (has_up_transition & has_down_transition));
            // is_cancelled!(self)?;

            debug!(
                " > Created initial sets for {:?} using {}+{} BDD nodes.",
                var,
                is_trap.size(),
                is_essential.size(),
            );

            to_merge.push(is_trap.and(&is_essential));
        }

        // TODO: discuss - this is the new version, does it have to use symbolic merge directly?
        let trap_spaces = FixedPoints::with_config(
            FixedPointsConfig::from(graph.clone())
                .with_cancellation_nowrap(self.config().cancellation.clone())
                // TODO: ohtenkay - adjust the limit
                .with_bdd_size_limit(self.config().bdd_size_limit),
        )
        // TODO: discuss - this is where the bdd_ctx is used
        .symbolic_merge(to_merge, HashSet::new(), "TODO")?;

        let trap_spaces = NetworkColoredSpaces::new(trap_spaces, ctx);
        // is_cancelled!(self)?;

        info!(
            "Found {}x{}[nodes:{}] essential trap spaces.",
            trap_spaces.colors().approx_cardinality(),
            trap_spaces.spaces().approx_cardinality(),
            trap_spaces.symbolic_size(),
        );

        Ok(trap_spaces)
    }

    /// Computes the minimal coloured trap spaces of the provided `network` within the specified
    /// `restriction` set.
    ///
    /// This method currently uses [Self::essential_symbolic], hence is always slower than
    /// this method.
    pub fn minimal_symbolic(&self) -> Result<NetworkColoredSpaces, TrapSpacesError> {
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
            "Start minimal subspace search with {}x{}[nodes:{}] candidates.",
            original.colors().approx_cardinality(),
            original.spaces().approx_cardinality(),
            original.symbolic_size()
        );

        while !original.is_empty() {
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
            // is_cancelled!(self)?;

            // Compute the set of strict super spaces.
            // TODO:
            //  This can take a long time if there are colors and a lot of traps, e.g.
            //  fixed-points, even though individual colors are easy. We should probably
            //  find a way to get rid of fixed points and any related super-spaces first,
            //  as these are clearly minimal. The other option would be to tune the super
            //  space enumeration to avoid spaces that are clearly irrelevant anyway.
            // TODO: ohtenkay - rewrite _mk_super_spaces, _impl_symbolic_space_context
            let super_spaces = ctx.mk_super_spaces(minimum_candidate.as_bdd());
            let super_spaces = NetworkColoredSpaces::new(super_spaces, ctx);
            // is_cancelled!(self)?;

            original = original.minus(&super_spaces);
            minimal = minimal.minus(&super_spaces).union(&minimum_candidate);
            // is_cancelled!(self)?;

            debug!(
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
            "Start maximal subspace search with {}x{}[nodes:{}] candidates.",
            original.colors().approx_cardinality(),
            original.spaces().approx_cardinality(),
            original.symbolic_size()
        );

        while !original.is_empty() {
            let maximum_candidate = original.pick_space();
            // is_cancelled!(self)?;

            // Compute the set of strict sub spaces.
            let super_spaces = ctx.mk_sub_spaces(maximum_candidate.as_bdd());
            let super_spaces = NetworkColoredSpaces::new(super_spaces, ctx);
            // is_cancelled!(self)?;

            original = original.minus(&super_spaces);
            maximal = maximal.minus(&super_spaces).union(&maximum_candidate);
            // is_cancelled!(self)?;

            // TODO: ohtenkay - implement debug with size limit, new macro, use log!(), check for
            // usages
            debug!(
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
            "Found {}[nodes:{}] maximal spaces.",
            maximal.approx_cardinality(),
            maximal.symbolic_size(),
        );

        Ok(maximal)
    }
}

#[pyclass(module = "biodivine_aeon", frozen)]
#[pyo3(name = "TrapSpaces")]
pub struct PyTrapSpaces(PyTrapSpacesConfig);

#[pymethods]
impl PyTrapSpaces {
    #[staticmethod]
    pub fn from_boolean_network(py: Python, bn: Py<BooleanNetworkBinding>) -> PyResult<Self> {
        Ok(PyTrapSpaces(PyTrapSpacesConfig::from_boolean_network(
            py, bn,
        )?))
    }

    #[staticmethod]
    pub fn from_graph_with_context(
        graph: &AsynchronousGraph,
        ctx: Py<SymbolicSpaceContextBinding>,
    ) -> Self {
        PyTrapSpaces(PyTrapSpacesConfig::from_graph_with_context(graph, ctx))
    }

    /// Create a new [TrapSpaces] instance with the given [TrapSpacesConfig].
    #[staticmethod]
    pub fn with_config(config: PyTrapSpacesConfig) -> Self {
        PyTrapSpaces(config)
    }

    /// Computes the coloured set of "essential" trap spaces of a Boolean network.
    ///
    /// A trap space is essential if it cannot be further reduced through percolation. In general, every
    /// minimal trap space is always essential.
    pub fn essential_symbolic(&self) -> PyResult<ColoredSpaceSet> {
        Ok(ColoredSpaceSet::wrap_native(
            self.0.symbolic_space_context(),
            self.0.inner().essential_symbolic()?,
        ))
    }

    /// Computes the minimal coloured trap spaces of the provided `network` within the specified
    /// `restriction` set.
    ///
    /// Currently, this method always slower than [Self::essential_symbolic], because it first has to compute
    /// the essential set.
    pub fn minimal_symbolic(&self) -> PyResult<ColoredSpaceSet> {
        Ok(ColoredSpaceSet::wrap_native(
            self.0.symbolic_space_context(),
            self.0.inner().minimal_symbolic()?,
        ))
    }

    /// Compute the inclusion-minimal spaces within a particular subset.
    pub fn minimize(&self, set: &ColoredSpaceSet) -> PyResult<ColoredSpaceSet> {
        Ok(ColoredSpaceSet::wrap_native(
            self.0.symbolic_space_context(),
            self.0.inner().minimize(set.as_native())?,
        ))
    }

    /// Compute the inclusion-maximal spaces within a particular subset.
    pub fn maximize(&self, set: &ColoredSpaceSet) -> PyResult<ColoredSpaceSet> {
        Ok(ColoredSpaceSet::wrap_native(
            self.0.symbolic_space_context(),
            self.0.inner().maximize(set.as_native())?,
        ))
    }
}
