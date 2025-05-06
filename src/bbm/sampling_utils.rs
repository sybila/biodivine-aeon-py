use biodivine_lib_bdd::BddPartialValuation;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColors, SymbolicAsyncGraph};
use biodivine_lib_param_bn::BooleanNetwork;

use rand::prelude::StdRng;
use rand::SeedableRng;

/// Randomly select a color from the given set of colors.
pub(super) fn pick_random_color(
    rng: &mut StdRng,
    graph: &SymbolicAsyncGraph,
    color_set: &GraphColors,
) -> GraphColors {
    let ctx = graph.symbolic_context();
    let random_witness = color_set.as_bdd().random_valuation(rng).unwrap();
    let mut partial_valuation = BddPartialValuation::empty();
    for var in ctx.parameter_variables() {
        partial_valuation.set_value(*var, random_witness[*var]);
    }
    let singleton_bdd = ctx
        .bdd_variable_set()
        .mk_conjunctive_clause(&partial_valuation);
    // We can use the "raw copy" function because into the new BDD, we only carried over
    // the BDD variables that encode network parameters.
    color_set.copy(singleton_bdd)
}

/// Generate a given number of instances of a provided partially specified BN.
/// Instances are selected at random, but user provides a random `seed` for
/// reproducibility.
pub(super) fn pick_random_instances(
    bn: &BooleanNetwork,
    instance_count: usize,
    seed: u64,
) -> Result<Vec<BooleanNetwork>, String> {
    let mut random_state = StdRng::seed_from_u64(seed);
    let graph = SymbolicAsyncGraph::new(bn)?;

    // start with a unit set, and we'll be slowly picking out individual colors
    let mut color_set = graph.mk_unit_colors();

    // collect `instance_count` networks
    let mut instances = Vec::new();
    while instances.len() < instance_count && !color_set.is_empty() {
        // get singleton color for the instance
        let instance_color = pick_random_color(&mut random_state, &graph, &color_set);
        assert!(instance_color.is_singleton());

        let instantiated_bn = graph.pick_witness(&instance_color);
        instances.push(instantiated_bn);

        // remove the color from the set so that we have unique instances
        color_set = color_set.minus(&instance_color);
    }

    Ok(instances)
}
