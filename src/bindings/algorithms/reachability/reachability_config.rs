use crate::bindings::algorithms::graph_representation::PyAsynchronousGraphType;
use crate::bindings::lib_param_bn::argument_types::variable_id_type::VariableIdType;
use crate::bindings::lib_param_bn::variable_id::VariableIdResolvable;
use biodivine_algo_bdd_scc::reachability::ReachabilityConfig;
use pyo3::{FromPyObject, PyResult, Python};

/// Internal helper struct which corresponds to the `ReachabilityConfig` typed dictionary and
/// converts to the native [`ReachabilityConfig`].
#[derive(FromPyObject)]
pub struct PyReachabilityConfig {
    #[pyo3(item)]
    pub graph: PyAsynchronousGraphType,
    #[pyo3(item, default = None)]
    pub active_variables: Option<Vec<VariableIdType>>,
    #[pyo3(item, default = None)]
    pub max_iterations: Option<usize>,
    #[pyo3(item, default = None)]
    pub max_symbolic_size: Option<usize>,
}

/// Corresponds to `ReachabilityConfig | AsynchronousGraph | BooleanNetwork`.
#[derive(FromPyObject)]
pub enum ReachabilityConfigOrGraph {
    Graph(PyAsynchronousGraphType),
    Config(PyReachabilityConfig),
}

impl PyReachabilityConfig {
    pub fn clone_native(&self, py: Python) -> PyResult<ReachabilityConfig> {
        let mut config = ReachabilityConfig::new(self.graph.clone_native(py));
        if let Some(active_variables) = &self.active_variables {
            config.active_variables =
                VariableIdType::resolve_collection(active_variables.clone(), &config.graph)?;
        }
        if let Some(max_iterations) = self.max_iterations {
            config.max_iterations = max_iterations;
        }
        if let Some(max_symbolic_size) = self.max_symbolic_size {
            config.max_symbolic_size = max_symbolic_size;
        }
        Ok(config)
    }
}

impl From<ReachabilityConfigOrGraph> for PyReachabilityConfig {
    fn from(value: ReachabilityConfigOrGraph) -> Self {
        match value {
            ReachabilityConfigOrGraph::Config(config) => config,
            ReachabilityConfigOrGraph::Graph(graph) => PyReachabilityConfig {
                graph,
                active_variables: None,
                max_iterations: None,
                max_symbolic_size: None,
            },
        }
    }
}
