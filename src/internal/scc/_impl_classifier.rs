use super::{Behaviour, Class, Classifier};
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{
    GraphColoredVertices, GraphColors, SymbolicAsyncGraph,
};
use std::collections::HashMap;
use std::sync::Mutex;

impl Classifier {
    pub fn new(graph: &SymbolicAsyncGraph) -> Classifier {
        let mut map: HashMap<Class, GraphColors> = HashMap::new();
        map.insert(Class::new_empty(), graph.unit_colors().clone());
        Classifier {
            classes: Mutex::new(map),
            attractors: Mutex::new(Vec::new()),
        }
    }

    /*// Try to fetch the current number of discovered classes in a non-blocking manner
    pub fn try_get_num_classes(&self) -> Option<usize> {
        match self.classes.try_lock() {
            Ok(data) => Some((*data).len()),
            _ => None,
        }
    }*/

    /*// Try to obtain a copy of data in a non-blocking manner (useful if we want to check
    // results but the computation is still running).
    pub fn try_export_result(&self) -> Option<HashMap<Class, GraphColors>> {
        match self.classes.try_lock() {
            Ok(data) => Some((*data).clone()),
            _ => None,
        }
    }*/

    /*pub fn try_get_params(&self, class: &Class) -> Option<Option<GraphColors>> {
        match self.classes.try_lock() {
            Ok(data) => Some((*data).get(class).cloned()),
            _ => None,
        }
    }*/

    /*pub fn get_params(&self, class: &Class) -> Option<GraphColors> {
        let data = self.classes.lock().unwrap();
        (*data).get(class).cloned()
    }*/

    pub fn export_result(&self) -> HashMap<Class, GraphColors> {
        let data = self.classes.lock().unwrap();
        (*data).clone()
    }

    /*pub fn export_components(
        &self,
    ) -> Vec<(GraphColoredVertices, HashMap<Behaviour, GraphColors>)> {
        let data = self.attractors.lock().unwrap();
        (*data).clone()
    }*/

    /// Export only components that have the specified behaviour.
    /*pub fn export_components_with_class(&self, class: Behaviour) -> Vec<GraphColoredVertices> {
        let data = self.attractors.lock().unwrap().clone();
        data.into_iter()
            .filter_map(|(attractor, behaviour)| {
                behaviour
                    .get(&class)
                    .map(|colors| attractor.intersect_colors(colors))
            })
            .collect()
    }*/

    /// Static function to classify just one component and immediately obtain results.
    pub fn classify_component(
        component: &GraphColoredVertices,
        graph: &SymbolicAsyncGraph,
    ) -> HashMap<Behaviour, GraphColors> {
        let classifier = Classifier::new(graph);
        classifier.add_component(component.clone(), graph);
        let mut result: HashMap<Behaviour, GraphColors> = HashMap::new();
        for (class, colors) in classifier.export_result() {
            if class.0.is_empty() {
                continue; // This is an empty class - those colors were not in the attractor.
            } else if class.0.len() > 1 {
                unreachable!("Multiple behaviours in one component.");
            } else {
                result.insert(class.0[0], colors);
            }
        }
        result
    }

    /*/// Find attractor of the given witness colour. The argument set must be a singleton.
    pub fn attractors(&self, witness_colour: &GraphColors) -> Vec<(GraphVertices, Behaviour)> {
        if witness_colour.as_bdd() != witness_colour.pick_singleton().as_bdd() {
            eprintln!("WARNING: Computing attractor witnesses for non-singleton color set.");
        }
        let mut result = Vec::new();
        let attractors = self.attractors.lock().unwrap();
        for (attractor, behaviour) in attractors.iter() {
            let attractor_states = attractor.intersect_colors(witness_colour);
            if attractor_states.is_empty() {
                continue;
            }
            let attractor_states = attractor_states.vertices();
            let attractor_behaviour = behaviour
                .iter()
                .find(|(_, c)| witness_colour.is_subset(c))
                .unwrap()
                .0;
            result.push((attractor_states, *attractor_behaviour));
        }
        result
    }*/

    // TODO: Parallelism
    pub fn add_component(&self, component: GraphColoredVertices, graph: &SymbolicAsyncGraph) {
        let mut component_classification = HashMap::new();
        let without_sinks = self.filter_sinks(component.clone(), graph);
        let not_sink_params = without_sinks.colors();
        let sink_params = component.colors().minus(&not_sink_params);
        if !sink_params.is_empty() {
            component_classification.insert(Behaviour::Stability, sink_params);
        }
        if not_sink_params.is_empty() {
            let mut attractors = self.attractors.lock().unwrap();
            (*attractors).push((component, component_classification));
            return;
        }
        if !not_sink_params.is_empty() {
            let mut disorder = graph.mk_empty_colors();
            for variable in graph.as_network().variables() {
                let found_first_successor = &graph.var_can_post(variable, &without_sinks);
                for next_variable in graph.as_network().variables() {
                    if next_variable == variable {
                        continue;
                    }
                    let found_second_successor =
                        &graph.var_can_post(next_variable, found_first_successor);
                    disorder = disorder.union(&found_second_successor.colors());
                }
            }
            let cycle = without_sinks.colors().minus(&disorder);
            if !cycle.is_empty() {
                component_classification.insert(Behaviour::Oscillation, cycle.clone());
                self.push(Behaviour::Oscillation, cycle);
            }
            if !disorder.is_empty() {
                component_classification.insert(Behaviour::Disorder, disorder.clone());
                self.push(Behaviour::Disorder, disorder);
            }
            let mut attractors = self.attractors.lock().unwrap();
            (*attractors).push((component, component_classification));
        }
    }

    fn push(&self, behaviour: Behaviour, params: GraphColors) {
        let mut classes = self.classes.lock().unwrap();
        let mut original_classes: Vec<Class> = (*classes).keys().cloned().collect();
        original_classes.sort();
        original_classes.reverse(); // we need classes from largest to smallest

        for class in original_classes {
            let class_params = &(*classes)[&class];
            let should_move_up = class_params.intersect(&params);
            if !should_move_up.is_empty() {
                let extended_class = class.clone_extended(behaviour);

                // remove moving params from class
                let new_c_p = class_params.minus(&should_move_up);
                if new_c_p.is_empty() {
                    (*classes).remove(&class);
                } else {
                    (*classes).insert(class, new_c_p);
                }

                // add moving params to larger_class
                if let Some(extended_class_params) = (*classes).get(&extended_class) {
                    let new_extended_params = extended_class_params.union(&should_move_up);
                    (*classes).insert(extended_class, new_extended_params);
                } else {
                    (*classes).insert(extended_class, should_move_up);
                }
            }
        }
    }

    /*pub fn print(&self) {
        let classes = self.classes.lock().unwrap();
        for (c, p) in &(*classes) {
            println!("Class {:?}, cardinality: {}", c, p.approx_cardinality());
        }
    }*/

    // TODO: Parallelism
    /// Remove all sink states from the given component (and push them into the classifier).
    fn filter_sinks(
        &self,
        component: GraphColoredVertices,
        graph: &SymbolicAsyncGraph,
    ) -> GraphColoredVertices {
        let mut is_not_sink = graph.empty_vertices().clone();
        for variable in graph.as_network().variables() {
            let has_successor = &graph.var_can_post(variable, &component);
            if !has_successor.is_empty() {
                is_not_sink = is_not_sink.union(has_successor);
            }
        }
        let is_sink = component.colors().minus(&is_not_sink.colors());
        if !is_sink.is_empty() {
            self.push(Behaviour::Stability, is_sink);
        }
        is_not_sink
    }
}
