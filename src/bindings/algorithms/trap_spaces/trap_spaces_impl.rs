use biodivine_lib_param_bn::{
    biodivine_std::traits::Set,
    fixed_points::FixedPoints,
    symbolic_async_graph::SymbolicAsyncGraph,
    trap_spaces::{NetworkColoredSpaces, SymbolicSpaceContext},
    BooleanNetwork,
};
use log::{debug, info};
use pyo3::prelude::*;
use std::collections::HashSet;

use crate::{
    bindings::{
        algorithms::{
            cancellation::CancellationHandler,
            trap_spaces::{
                trap_spaces_config::TrapSpacesConfig, trap_spaces_error::TrapSpacesError,
            },
        },
        lib_param_bn::symbolic::{
            asynchronous_graph::AsynchronousGraph, set_colored_space::ColoredSpaceSet,
            symbolic_space_context::SymbolicSpaceContext as SymbolicSpaceContextPython,
        },
    },
    is_cancelled, AsNative,
};

#[pyclass(module = "biodivine_aeon", frozen)]
#[derive(Clone)]
pub struct TrapSpaces(TrapSpacesConfig);

impl TrapSpaces {
    /// Create a new "default" [TrapSpaces] for the given [BooleanNetwork].
    pub fn from_boolean_network(bn: BooleanNetwork) -> Result<Self, TrapSpacesError> {
        TrapSpacesConfig::from_boolean_network(bn).map(TrapSpaces)
    }

    /// Retrieve the internal [TrapSpacesConfig] of this instance.
    pub fn config(&self) -> &TrapSpacesConfig {
        &self.0
    }
}

impl CancellationHandler for TrapSpaces {
    fn is_cancelled(&self) -> bool {
        self.config().cancellation.is_cancelled()
    }

    fn start_timer(&self) {
        self.config().cancellation.start_timer();
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

        let bdd_ctx = ctx.bdd_variable_set();

        // We always start with the restriction set, because it should carry the information
        // about valid encoding of spaces.
        let mut to_merge = vec![restriction.as_bdd().clone()];
        for var in graph.variables() {
            let update_bdd = graph.get_symbolic_fn_update(var);
            let not_update_bdd = update_bdd.not();
            is_cancelled!(self)?;

            // TODO: discuss - how to rewrite _mk_can_go_to_true?
            let has_up_transition = ctx.mk_can_go_to_true(update_bdd);
            is_cancelled!(self)?;

            // TODO: discuss - how to rewrite _mk_can_go_to_true?
            let has_down_transition = ctx.mk_can_go_to_true(&not_update_bdd);
            is_cancelled!(self)?;

            let true_var = ctx.get_positive_variable(var);
            let false_var = ctx.get_negative_variable(var);

            let is_trap_1 = has_up_transition.imp(&bdd_ctx.mk_var(true_var));
            let is_trap_2 = has_down_transition.imp(&bdd_ctx.mk_var(false_var));
            let is_trap = is_trap_1.and(&is_trap_2);
            is_cancelled!(self)?;

            let is_essential_1 = bdd_ctx.mk_var(true_var).and(&bdd_ctx.mk_var(false_var));
            let is_essential_2 = has_up_transition.and(&has_down_transition);
            let is_essential = is_essential_1.imp(&is_essential_2);
            is_cancelled!(self)?;

            // TODO: discuss - ask about this
            // This will work in next version of lib-bdd:
            // let is_trap = bdd!(bdd_ctx, (has_up_transition => true_var) & (has_down_transition => false_var));
            // let is_essential = bdd!(bdd_ctx, (true_var & false_var) => (has_up_transition & has_down_transition));

            debug!(
                " > Created initial sets for {:?} using {}+{} BDD nodes.",
                var,
                is_trap.size(),
                is_essential.size(),
            );

            to_merge.push(is_trap.and(&is_essential));
        }

        // TODO: discuss - use the new version here, create struct?
        let trap_spaces = FixedPoints::symbolic_merge(bdd_ctx, to_merge, HashSet::default());
        let trap_spaces = NetworkColoredSpaces::new(trap_spaces, ctx);
        is_cancelled!(self)?;

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
    pub fn _minimal_symbolic<E, F: Fn() -> Result<(), E>>(
        ctx: &SymbolicSpaceContext,
        graph: &SymbolicAsyncGraph,
        restriction: &NetworkColoredSpaces,
        log_level: usize,
        interrupt: &F,
    ) -> Result<NetworkColoredSpaces, E> {
        let essential = Self::_essential_symbolic(ctx, graph, restriction, log_level, interrupt)?;
        Self::_minimize(ctx, &essential, log_level, interrupt)
    }

    /// Compute the minimal spaces within a particular subset.
    pub fn _minimize<E, F: Fn() -> Result<(), E>>(
        ctx: &SymbolicSpaceContext,
        spaces: &NetworkColoredSpaces,
        log_level: usize,
        interrupt: &F,
    ) -> Result<NetworkColoredSpaces, E> {
        let mut original = spaces.clone();
        let mut minimal = ctx.mk_empty_colored_spaces();

        if should_log(log_level) {
            println!(
                "Start minimal subspace search with {}x{}[nodes:{}] candidates.",
                original.colors().approx_cardinality(),
                original.spaces().approx_cardinality(),
                original.symbolic_size()
            );
        }

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
            interrupt()?;

            // Compute the set of strict super spaces.
            // TODO:
            //  This can take a long time if there are colors and a lot of traps, e.g.
            //  fixed-points, even though individual colors are easy. We should probably
            //  find a way to get rid of fixed points and any related super-spaces first,
            //  as these are clearly minimal. The other option would be to tune the super
            //  space enumeration to avoid spaces that are clearly irrelevant anyway.
            let super_spaces =
                ctx._mk_super_spaces(minimum_candidate.as_bdd(), log_level, interrupt)?;
            let super_spaces = NetworkColoredSpaces::new(super_spaces, ctx);
            interrupt()?;

            original = original.minus(&super_spaces);
            minimal = minimal.minus(&super_spaces).union(&minimum_candidate);
            interrupt()?;

            if log_essential(
                log_level,
                original.symbolic_size() + minimal.symbolic_size(),
            ) {
                println!(
                    "Minimization in progress: {}x{}[nodes:{}] unprocessed, {}x{}[nodes:{}] candidates.",
                    original.colors().approx_cardinality(),
                    original.spaces().approx_cardinality(),
                    original.symbolic_size(),
                    minimal.colors().approx_cardinality(),
                    minimal.spaces().approx_cardinality(),
                    minimal.symbolic_size(),
                );
            }
        }

        if should_log(log_level) {
            println!(
                "Found {}[nodes:{}] minimal spaces.",
                minimal.approx_cardinality(),
                minimal.symbolic_size(),
            );
        }

        Ok(minimal)
    }

    /// The same as [Self::minimize], but searches for maximal spaces within `spaces`.
    pub fn _maximize<E, F: Fn() -> Result<(), E>>(
        ctx: &SymbolicSpaceContext,
        spaces: &NetworkColoredSpaces,
        log_level: usize,
        interrupt: &F,
    ) -> Result<NetworkColoredSpaces, E> {
        let mut original = spaces.clone();
        let mut maximal = ctx.mk_empty_colored_spaces();

        if should_log(log_level) {
            println!(
                "Start maximal subspace search with {}x{}[nodes:{}] candidates.",
                original.colors().approx_cardinality(),
                original.spaces().approx_cardinality(),
                original.symbolic_size()
            );
        }

        while !original.is_empty() {
            let maximum_candidate = original.pick_space();
            interrupt()?;

            // Compute the set of strict sub spaces.
            let super_spaces = ctx.mk_sub_spaces(maximum_candidate.as_bdd());
            let super_spaces = NetworkColoredSpaces::new(super_spaces, ctx);
            interrupt()?;

            original = original.minus(&super_spaces);
            maximal = maximal.minus(&super_spaces).union(&maximum_candidate);
            interrupt()?;

            if log_essential(
                log_level,
                original.symbolic_size() + maximal.symbolic_size(),
            ) {
                println!(
                    "Maximization in progress: {}x{}[nodes:{}] unprocessed, {}x{}[nodes:{}] candidates.",
                    original.colors().approx_cardinality(),
                    original.spaces().approx_cardinality(),
                    original.symbolic_size(),
                    maximal.colors().approx_cardinality(),
                    maximal.spaces().approx_cardinality(),
                    maximal.symbolic_size(),
                );
            }
        }

        if should_log(log_level) {
            println!(
                "Found {}[nodes:{}] maximal spaces.",
                maximal.approx_cardinality(),
                maximal.symbolic_size(),
            );
        }

        Ok(maximal)
    }
}

#[pymethods]
impl TrapSpaces {
    /// Computes the coloured set of "essential" trap spaces of a Boolean network.
    ///
    /// A trap space is essential if it cannot be further reduced through percolation. In general, every
    /// minimal trap space is always essential.
    #[staticmethod]
    #[pyo3(signature = (ctx, graph, restriction = None))]
    pub fn essential_symbolic_py(
        py: Python,
        ctx: Py<SymbolicSpaceContextPython>,
        graph: &AsynchronousGraph,
        restriction: Option<&ColoredSpaceSet>,
    ) -> PyResult<ColoredSpaceSet> {
        let unit = ctx
            .get()
            .as_native()
            .mk_unit_colored_spaces(graph.as_native());
        let restriction = restriction.map(|it| it.as_native()).unwrap_or(&unit);
        let result = biodivine_lib_param_bn::trap_spaces::TrapSpaces::_essential_symbolic(
            ctx.get().as_native(),
            graph.as_native(),
            restriction,
            global_log_level(py)?,
            &|| py.check_signals(),
        )?;
        Ok(ColoredSpaceSet::wrap_native(ctx.clone(), result))
    }

    /// Computes the minimal coloured trap spaces of the provided `network` within the specified
    /// `restriction` set.
    ///
    /// Currently, this method always slower than [Self::essential_symbolic], because it first has to compute
    /// the essential set.
    #[staticmethod]
    #[pyo3(signature = (ctx, graph, restriction = None))]
    pub fn minimal_symbolic_py(
        py: Python,
        ctx: Py<SymbolicSpaceContextPython>,
        graph: &AsynchronousGraph,
        restriction: Option<&ColoredSpaceSet>,
    ) -> PyResult<ColoredSpaceSet> {
        let unit = ctx
            .get()
            .as_native()
            .mk_unit_colored_spaces(graph.as_native());
        let restriction = restriction.map(|it| it.as_native()).unwrap_or(&unit);
        let result = biodivine_lib_param_bn::trap_spaces::TrapSpaces::_minimal_symbolic(
            ctx.get().as_native(),
            graph.as_native(),
            restriction,
            global_log_level(py)?,
            &|| py.check_signals(),
        )?;
        Ok(ColoredSpaceSet::wrap_native(ctx.clone(), result))
    }

    /// Compute the inclusion-minimal spaces within a particular subset.
    #[staticmethod]
    pub fn minimize_py(
        py: Python,
        ctx: Py<SymbolicSpaceContextPython>,
        set: &ColoredSpaceSet,
    ) -> PyResult<ColoredSpaceSet> {
        let result = biodivine_lib_param_bn::trap_spaces::TrapSpaces::_minimize(
            ctx.get().as_native(),
            set.as_native(),
            global_log_level(py)?,
            &|| py.check_signals(),
        )?;
        Ok(ColoredSpaceSet::wrap_native(ctx.clone(), result))
    }

    /// Compute the inclusion-maximal spaces within a particular subset.
    #[staticmethod]
    pub fn maximize_py(
        py: Python,
        ctx: Py<SymbolicSpaceContextPython>,
        set: &ColoredSpaceSet,
    ) -> PyResult<ColoredSpaceSet> {
        let result = biodivine_lib_param_bn::trap_spaces::TrapSpaces::_maximize(
            ctx.get().as_native(),
            set.as_native(),
            global_log_level(py)?,
            &|| py.check_signals(),
        )?;
        Ok(ColoredSpaceSet::wrap_native(ctx.clone(), result))
    }
}
