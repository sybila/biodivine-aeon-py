//! Main high-level functionality regarding the BN classification based on HCTL properties.

use crate::internal::classification::load_inputs::*;
use crate::internal::classification::write_output::{write_classifier_output, write_empty_report};

use biodivine_hctl_model_checker::mc_utils::{
    collect_unique_hctl_vars, get_extended_symbolic_graph,
};
use biodivine_hctl_model_checker::model_checking::{
    model_check_multiple_trees_dirty, model_check_tree_dirty,
};
use biodivine_hctl_model_checker::postprocessing::sanitizing::sanitize_colors;
use biodivine_hctl_model_checker::preprocessing::parser::parse_and_minimize_hctl_formula;

use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{
    GraphColoredVertices, GraphColors, SymbolicAsyncGraph, SymbolicContext,
};
use biodivine_lib_param_bn::{BooleanNetwork, ModelAnnotation};

use biodivine_hctl_model_checker::preprocessing::hctl_tree::HctlTreeNode;
use std::cmp::max;

/// Return the set of colors for which ALL system states are contained in the given color-vertex
/// set (i.e., if the given relation is a result of model checking a property, get colors for which
/// the property holds universally in every state).
///
/// Formally, this is a universal projection on the colors of the given `colored_vertices`.
pub fn get_universal_colors(
    stg: &SymbolicAsyncGraph,
    colored_vertices: &GraphColoredVertices,
) -> GraphColors {
    let complement = stg.unit_colored_vertices().minus(colored_vertices);
    stg.unit_colors().minus(&complement.colors())
}

/// Perform the classification of Boolean networks based on given properties.
/// Takes a path to a file in annotated `AEON` format containing a partially defined BN model
/// and 2 sets of HCTL formulae. Assertions are formulae that must be satisfied, and properties
/// are formulae used for classification.
///
/// First, colors satisfying all assertions are computed, and then the set of remaining colors is
/// decomposed into categories based on satisfied properties. One class = colors where the same set
/// of properties is satisfied (universally).
///
/// Report and BDDs representing resulting classes are generated into `output_zip` archive.
pub fn classify(model_path: &str, output_zip: &str) -> Result<(), String> {
    // TODO: allow caching between model-checking assertions and properties somehow

    // load the model and two sets of formulae (from model annotations)
    let Ok(aeon_str) = std::fs::read_to_string(model_path) else {
        return Err(format!("Input file `{model_path}` is not accessible."));
    };
    let bn = BooleanNetwork::try_from(aeon_str.as_str())?;
    let ctx = SymbolicContext::new(&bn).unwrap();
    let annotations = ModelAnnotation::from_model_string(aeon_str.as_str());
    let assertions = read_model_assertions(&annotations);
    let named_properties = read_model_properties(&annotations)?;
    println!("Loaded model and properties out of `{model_path}`.");

    println!("Parsing formulae and generating symbolic representation...");
    // Combine all assertions into one formula and add it to the list of properties.
    let assertion = build_combined_assertion(&assertions);
    // Adjust message depending on the number of properties (singular/multiple)
    let assertion_message = if assertions.len() == 1 {
        "property (assertion)"
    } else {
        "properties (assertions)"
    };
    println!(
        "Successfully parsed all {} required {assertion_message} and all {} classification properties.",
        assertions.len(),
        named_properties.len(),
    );

    // Parse all formulae and count the max. number of HCTL variables across formulae.
    let assertion_tree = parse_and_minimize_hctl_formula(&ctx, &assertion)?;
    let mut num_hctl_vars = collect_unique_hctl_vars(assertion_tree.clone()).len();
    let mut property_trees: Vec<HctlTreeNode> = Vec::new();
    for (_name, formula) in &named_properties {
        let tree = parse_and_minimize_hctl_formula(&ctx, formula.as_str())?;
        let tree_vars = collect_unique_hctl_vars(tree.clone()).len();
        num_hctl_vars = max(num_hctl_vars, tree_vars);
        property_trees.push(tree);
    }

    // Instantiate extended STG with enough variables to evaluate all formulae.
    let Ok(graph) = get_extended_symbolic_graph(&bn, num_hctl_vars as u16) else {
        return Err("Unable to generate STG for provided PSBN model.".to_string());
    };
    println!(
        "Successfully encoded model with {} variables and {} parameters.",
        graph.symbolic_context().num_state_variables(),
        graph.symbolic_context().num_parameter_variables(),
    );
    println!(
        "Model admits {:.0} instances.",
        graph.mk_unit_colors().approx_cardinality(),
    );

    println!("Evaluating required properties (this may take some time)...");
    // Compute the colors (universally) satisfying the combined assertion formula.
    let assertion_result = model_check_tree_dirty(assertion_tree, &graph)?;
    let valid_colors = get_universal_colors(&graph, &assertion_result);
    println!("Required properties successfully evaluated.");
    println!(
        "{:.0} instances satisfy all required properties.",
        valid_colors.approx_cardinality(),
    );

    if valid_colors.is_empty() {
        println!("No instance satisfies given required properties. Aborting.");
        return write_empty_report(&assertions, output_zip).map_err(|e| format!("{e:?}"));
    }

    // restrict the colors on the symbolic graph
    let graph = SymbolicAsyncGraph::with_custom_context(
        &bn,
        graph.symbolic_context().clone(),
        valid_colors.as_bdd().clone(),
    )?;

    println!("Evaluating classification properties (this may take some time)...");
    // Model check all properties on the restricted graph.
    let property_result = model_check_multiple_trees_dirty(property_trees, &graph)?;
    let property_colors: Vec<GraphColors> = property_result
        .iter()
        .map(|result| get_universal_colors(&graph, result))
        .collect();
    println!("Classification properties successfully evaluated.");

    // This is an important step where we ensure that the "model checking context"
    // does not "leak" outside of the BN classifier. In essence, this ensures that the
    // BDD that we output is compatible with any `SymbolicAsyncGraph` based on the
    // originally supplied model (i.e. if we want to read the BDD, we don't have to
    // add any additional state variables to the symbolic context).
    let valid_colors = sanitize_colors(&graph, &valid_colors);
    let property_colors: Vec<GraphColors> = property_colors
        .iter()
        .map(|c| sanitize_colors(&graph, c))
        .collect();

    // do the classification while printing the report and dumping resulting BDDs
    println!("Generating classification mapping based on model-checking results...");
    write_classifier_output(
        &assertions,
        &valid_colors,
        &named_properties,
        &property_colors,
        output_zip,
        aeon_str.as_str(),
    )
    .map_err(|e| format!("{e:?}"))?;
    println!("Results saved to `{output_zip}`.");

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::internal::classification::classify::{
        build_combined_assertion, read_model_assertions, read_model_properties,
    };
    use biodivine_hctl_model_checker::mc_utils::collect_unique_hctl_vars;
    use biodivine_hctl_model_checker::preprocessing::parser::parse_and_minimize_hctl_formula;
    use biodivine_lib_param_bn::symbolic_async_graph::SymbolicContext;
    use biodivine_lib_param_bn::{BooleanNetwork, ModelAnnotation};
    use std::cmp::max;

    #[test]
    /// Test the formulae parsing and variable counting
    fn test_formulae_variable_count() {
        let aeon_str = r"
            $v_2:!v_3
            v_3 -| v_2
            $v_3:v_3
            v_3 -> v_3
        ";
        let bn = BooleanNetwork::try_from(aeon_str).unwrap();
        let ctx = SymbolicContext::new(&bn).unwrap();
        let formulae = vec![
            "!{x}: AX {x}".to_string(),
            "!{y}: (AG EF {y} & (!{z}: AX {z}))".to_string(),
        ];

        let mut var_count = 0;
        for f in formulae {
            let tree = parse_and_minimize_hctl_formula(&ctx, f.as_str()).unwrap();
            let c = collect_unique_hctl_vars(tree).len();
            var_count = max(c, var_count);
        }
        assert_eq!(var_count, 2);
    }

    #[test]
    /// Test combining of assertion formulae into one conjunction formula.
    fn test_assertion_formulae_merge() {
        let formula1 = "3{x}: @{x}: AX {x}".to_string();
        let formula2 = "false".to_string();
        let formula3 = "a & b".to_string();

        // empty vector should result in true constant
        assert_eq!(build_combined_assertion(&[]), "true".to_string());

        // otherwise, result should be a conjunction ending with `& true`
        assert_eq!(
            build_combined_assertion(&[formula1.clone(), formula2.clone()]),
            "(3{x}: @{x}: AX {x}) & (false)".to_string(),
        );
        assert_eq!(
            build_combined_assertion(&[formula1, formula2, formula3]),
            "(3{x}: @{x}: AX {x}) & (false) & (a & b)".to_string(),
        )
    }

    #[test]
    /// Test extracting the formulae from the AEON format annotations.
    fn test_extracting_formulae() {
        let aeon_str = r"
            #! dynamic_assertion: #`true`#
            #! dynamic_assertion: #`3{x}: @{x}: AX {x}`#
            #! dynamic_property: p1: #`3{x}: @{x}: AG EF {x}`#
            #! dynamic_property: p2: #`3{x}: @{x}: AX AF {x}`#
            $v_2:!v_3
            v_3 -| v_2
            $v_3:v_3
            v_3 -> v_3
        ";

        let annotations = ModelAnnotation::from_model_string(aeon_str);
        let assertions = read_model_assertions(&annotations);
        let named_properties = read_model_properties(&annotations).unwrap();

        assert_eq!(
            assertions,
            vec!["true".to_string(), "3{x}: @{x}: AX {x}".to_string()]
        );

        assert_eq!(named_properties.len(), 2);
        assert!(named_properties.contains(&("p1".to_string(), "3{x}: @{x}: AG EF {x}".to_string())));
        assert!(named_properties.contains(&("p2".to_string(), "3{x}: @{x}: AX AF {x}".to_string())));
    }

    #[test]
    /// Test that extracting entities from the corrupted AEON format annotations.
    fn test_extracting_formulae_corrupted() {
        let aeon_str = r"
            #! dynamic_assertion: #`true`#
            #! dynamic_property: p1: #`3{x}: @{x}: AG EF {x}`#
            #! dynamic_property: p1: #`3{x}: @{x}: AX {x}`#
            $v_2:!v_3
            v_3 -| v_2
            $v_3:v_3
            v_3 -> v_3
        ";
        let annotations = ModelAnnotation::from_model_string(aeon_str);
        let props = read_model_properties(&annotations);
        assert!(props.is_err());
        assert_eq!(
            props.err().unwrap().as_str(),
            "Found multiple properties named `p1`."
        );
    }
}
