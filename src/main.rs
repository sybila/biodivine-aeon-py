use std::collections::HashMap;
use std::hash::Hash;
use std::path::Path;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::{BooleanNetwork, VariableId};
use biodivine_lib_param_bn::symbolic_async_graph::{GraphVertices, SymbolicAsyncGraph};
use biodivine_aeon::internal::scc::algo_interleaved_transition_guided_reduction::interleaved_transition_guided_reduction;
use biodivine_aeon::internal::scc::algo_xie_beerel::xie_beerel_attractors;
use biodivine_aeon::internal::scc::ClassifierPhenotype;

fn main() {
    let model_path = Path::new("model.aeon");
    let model = BooleanNetwork::try_from_file(model_path).unwrap();
    let stg = SymbolicAsyncGraph::new(&model).unwrap();
    let phenotypes = get_mapk_phenotypes(&stg);

    let (states, transitions) = interleaved_transition_guided_reduction(&stg, stg.mk_unit_colored_vertices());
    let result = xie_beerel_attractors(&stg, &states, &transitions);
    let classes = ClassifierPhenotype::classify_all_components(result, &stg, &phenotypes);
    println!("{:?}", classes.keys());
}

pub fn get_mapk_phenotypes(graph: &SymbolicAsyncGraph) -> Vec<(String, GraphVertices)> {
    let mut phenotypes = Vec::new();
    let mut phenotype_vals = HashMap::new();
    phenotype_vals.insert("v_Apoptosis", true);
    phenotype_vals.insert("v_Growth_Arrest", true);
    phenotype_vals.insert("v_Proliferation", false);
    let apoptosis_space = build_phenotype(graph, phenotype_vals.clone());
    phenotypes.push(("apoptosis".to_string(), apoptosis_space.clone()));

    let mut phenotype_vals = HashMap::new();
    phenotype_vals.insert("v_Apoptosis", false);
    phenotype_vals.insert("v_Growth_Arrest", false);
    phenotype_vals.insert("v_Proliferation", true);
    let proliferation_space = build_phenotype(graph, phenotype_vals.clone());
    phenotypes.push(("proliferation".to_string(), proliferation_space.clone()));


    let mut phenotype_vals = HashMap::new();
    phenotype_vals.insert("v_Apoptosis", false);
    phenotype_vals.insert("v_Growth_Arrest", true);
    phenotype_vals.insert("v_Proliferation", false);
    let growth_arrest_space  = build_phenotype(graph, phenotype_vals.clone());
    phenotypes.push(("growth_arrest".to_string(), growth_arrest_space.clone()));


    let mut phenotype_vals = HashMap::new();
    phenotype_vals.insert("v_Apoptosis", false);
    phenotype_vals.insert("v_Growth_Arrest", false);
    phenotype_vals.insert("v_Proliferation", false);
    let no_decision_space = build_phenotype(graph, phenotype_vals);
    phenotypes.push(("no_decision".to_string(), no_decision_space.clone()));

    let other_space = graph.mk_unit_vertices().minus(&apoptosis_space).minus(&proliferation_space).minus(&growth_arrest_space).minus(&no_decision_space);
    phenotypes.push(("other".to_string(), other_space));

    return phenotypes
}


pub fn build_phenotype(graph: &SymbolicAsyncGraph, phenotype: HashMap<&str, bool>) -> GraphVertices {
    let mut result = graph.unit_colored_vertices().clone();
    for (var, value) in phenotype {
        let var_id = resolve_var_id(graph, var).unwrap();
        let subspace = graph.fix_network_variable(var_id, value.clone());
        result = result.intersect(&subspace);
    }

    result.vertices().clone()
}

pub fn resolve_var_id(graph: &SymbolicAsyncGraph, var: &str) -> Option<VariableId> {
    let mut v_name: &str = "";
    for v in graph.variables() {
        // Resolve variable name
        v_name = graph.as_network().unwrap().get_variable_name(v);
        if var == v_name {
            return Some(v)
        }
    }
    assert_eq!(var, v_name, "Unknown variable");
    None
}