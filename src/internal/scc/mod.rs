use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, GraphColors};
use std::collections::HashMap;
use std::sync::Mutex;

mod _impl_behaviour;
/// **(internal)** Utility methods for the behavior `Class`.
mod _impl_class;
/// **(internal)** Implementation of `Behaviour` classification in `Classifier`.
mod _impl_classifier;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Behaviour {
    Stability,
    Oscillation,
    Disorder,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Class(pub Vec<Behaviour>);

pub struct Classifier {
    classes: Mutex<HashMap<Class, GraphColors>>,
    attractors: Mutex<Vec<(GraphColoredVertices, HashMap<Behaviour, GraphColors>)>>,
}
