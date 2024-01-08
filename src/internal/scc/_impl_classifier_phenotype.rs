use crate::internal::scc::ClassifierPhenotype;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{
    GraphColoredVertices, GraphColors, GraphVertices, SymbolicAsyncGraph,
};
use itertools::Itertools;
use std::collections::HashMap;
use std::iter::zip;
use std::string::String;

impl ClassifierPhenotype {
    pub fn new() -> ClassifierPhenotype {
        ClassifierPhenotype {
            classes: HashMap::new(),
            phenotypes: Vec::new(),
        }
    }

    pub fn classify_component(
        component: &GraphColoredVertices,
        graph: &SymbolicAsyncGraph,
        eligible_phenotypes: &[(String, GraphVertices)],
    ) -> Vec<(String, GraphColors)> {
        let mut single_component_classification: HashMap<String, GraphColors> = HashMap::new();
        for (name, space) in eligible_phenotypes {
            let in_phenotype = component.intersect_vertices(space);
            single_component_classification.insert(name.to_string(), in_phenotype.colors());
        }

        let mut full_component_classification: Vec<(String, GraphColors)> = Vec::new();
        let mut taken_colors = graph.mk_empty_colors();

        for i in (1..eligible_phenotypes.len() + 1).rev() {
            for c in eligible_phenotypes.iter().combinations(i) {
                let mut combination_colors = graph.mk_unit_colors();
                let mut phen_group_names: Vec<String> = Vec::new();
                for (p, _) in c {
                    combination_colors = combination_colors.intersect(&single_component_classification.get(p).unwrap());
                    phen_group_names.push(p.to_string());
                }
                combination_colors = combination_colors.minus(&taken_colors);
                if combination_colors.is_empty() {
                    continue;
                }
                let phen_group_name = format!("<{}>", phen_group_names.join(","));
                taken_colors = taken_colors.union(&combination_colors);
                full_component_classification.push((phen_group_name, combination_colors));
            }
        }

        full_component_classification
    }

    pub fn classify_all_components(
        components: Vec<GraphColoredVertices>,
        graph: &SymbolicAsyncGraph,
        eligible_phenotypes: &Vec<(String, GraphVertices)>,
    ) -> HashMap<String, GraphColors> {
        let mut eligible_phenotypes_sorted = eligible_phenotypes.to_vec();
        eligible_phenotypes_sorted.sort_by(|(p1, _), (p2, _)| p1.cmp(p2));

        let classified_components: Vec<Vec<(String, GraphColors)>> = components
            .iter()
            .map(|x| ClassifierPhenotype::classify_component(x, graph, &eligible_phenotypes_sorted))
            .collect();
        let components_keys: Vec<Vec<String>> = classified_components
            .iter()
            .map(|v| v.iter().map(|(it, _)| it.to_string()).collect())
            .collect();

        let eligible_class_clusters: Vec<Vec<String>> = components_keys
            .into_iter()
            .multi_cartesian_product()
            .collect();

        let mut fully_classified: HashMap<String, GraphColors> = HashMap::new();
        for cluster in eligible_class_clusters {
            let mut belonging_colors = graph.mk_unit_colors();
            for (actual_classes, required_class) in
                zip(classified_components.iter(), cluster.iter())
            {
                if required_class.as_str() == "#" {
                    continue;
                }
                let mut violating_colors = graph.mk_unit_colors();
                for (p, c) in actual_classes {
                    if p == required_class {
                        violating_colors = violating_colors.minus(c);
                    }
                }
                belonging_colors = belonging_colors.minus(&violating_colors);
            }
            if !belonging_colors.is_empty() {
                let mut normalized_keys = cluster
                    .iter()
                    .filter(|req_class| req_class.as_str() != "#")
                    .collect::<Vec<&String>>();
                normalized_keys.sort();
                let normalized_key = normalized_keys.iter().join(";");
                let to_insert = if let Some(value) = fully_classified.get(&normalized_key) {
                    belonging_colors.union(value)
                } else {
                    belonging_colors
                };
                fully_classified.insert(normalized_key, to_insert);
            }
        }

        fully_classified
    }
}
