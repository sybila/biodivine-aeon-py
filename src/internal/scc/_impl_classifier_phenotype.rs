use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, GraphColors, GraphVertices, SymbolicAsyncGraph};
use std::collections::HashMap;
use crate::internal::scc::ClassifierPhenotype;
use std::iter::zip;
use itertools::Itertools;
use std::string::String;

impl ClassifierPhenotype {
    pub fn new() -> ClassifierPhenotype {
        ClassifierPhenotype {
            classes: HashMap::new(),
            phenotypes: Vec::new()
        }
    }

    pub fn classify_component(component: &GraphColoredVertices, graph: &SymbolicAsyncGraph, eligible_phenotypes: Vec<(String, GraphVertices)>) -> Vec<(String, GraphColors)>{
        let mut single_component_classification: HashMap<String, GraphColors> = HashMap::new();
        for (name, space) in eligible_phenotypes.iter() {
            let in_phenotype = component.intersect_vertices(space);
            single_component_classification.insert(name.clone(), in_phenotype.colors());
        }

        let mut full_component_classification: Vec<(String, GraphColors)> = vec![];
        let mut taken_colors = graph.mk_empty_colors();

        for i in (1..eligible_phenotypes.len()+1).rev() {
            for c in eligible_phenotypes.clone().into_iter().combinations(i) {
                let mut combination_colors = graph.mk_unit_colors();
                let mut phen_group_names: Vec<String> = vec!();
                for (p, space) in c {
                    let bad_colors = graph.mk_unit_colored_vertices().minus_vertices(&space).colors();
                    combination_colors = combination_colors.minus(&bad_colors);
                    phen_group_names.push(p);
                }
                combination_colors = combination_colors.minus(&taken_colors);
                if combination_colors.is_empty() {
                    continue
                }
                let phen_group_name = format!("<{}>", phen_group_names.join(","));
                full_component_classification.push((phen_group_name, combination_colors.clone()));
                taken_colors = taken_colors.minus(&combination_colors);
            }
        }

        full_component_classification
    }

    pub fn classify_all_components(
        components: Vec<GraphColoredVertices>,
        graph: &SymbolicAsyncGraph,
        eligible_phenotypes: Vec<(String, GraphVertices)>
    ) -> HashMap<String, GraphColors> {
        let mut eligible_phenotypes_sorted = eligible_phenotypes.clone();
        eligible_phenotypes_sorted.sort_by(|(p1, _), (p2, _)| {p1.cmp(p2)});

        let classified_components: Vec<Vec<(String, GraphColors)>> = components.iter().map(|x| -> Vec<(String, GraphColors)> {ClassifierPhenotype::classify_component(x, graph, eligible_phenotypes_sorted.clone())}).collect();
        let components_keys: Vec<Vec<String>> = classified_components.clone().into_iter().clone().map(|v| -> Vec<String> {v.iter().map(|x| {x.clone().0}).collect()}).collect();

        let eligible_class_clusters: Vec<Vec<String>>= components_keys.into_iter().multi_cartesian_product().into_iter().collect();

        let mut fully_classified: HashMap<String, GraphColors> = HashMap::new();
        for cluster in eligible_class_clusters {
            let mut belonging_colors = graph.mk_unit_colors();
            for (actual_classes, required_class) in zip(classified_components.clone(), cluster.clone()) {
                if required_class == "#".to_string() {
                    continue;
                }
                let mut violating_colors = graph.mk_unit_colors();
                for (p, c) in actual_classes {
                    if p == required_class {
                        violating_colors = violating_colors.minus(&c);
                    }
                }
                belonging_colors = belonging_colors.minus(&violating_colors);
            }
            if !belonging_colors.is_empty() {
                let mut normalized_keys = cluster.iter().filter(|req_class| {req_class.clone() != &"#".to_string()}).collect::<Vec<&String>>();
                normalized_keys.sort();
                let normalized_key = normalized_keys.iter().join(";");
                if fully_classified.keys().any(|k| {k == &normalized_key}) {
                    fully_classified.insert(normalized_key.clone(), belonging_colors.union(fully_classified.get(&normalized_key.clone()).unwrap()));
                } else {
                    fully_classified.insert(normalized_key.clone(), belonging_colors.clone());
                }
            }
        }

        fully_classified
    }
}
