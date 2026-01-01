use crate::bindings::algorithms::graph_representation::PyAsynchronousGraphType;
use crate::bindings::lib_param_bn::argument_types::variable_id_type::VariableIdType;
use crate::bindings::lib_param_bn::symbolic::asynchronous_graph::AsynchronousGraph;
use crate::bindings::lib_param_bn::variable_id::VariableIdResolvable;
use biodivine_algo_bdd_scc::attractor::AttractorConfig;
use pyo3::{FromPyObject, Py, PyResult, Python};

/// Internal helper struct which corresponds to the `AttractorConfig` typed dictionary and
/// converts to the native [`AttractorConfig`].
#[derive(FromPyObject)]
pub struct PyAttractorConfig {
    #[pyo3(item)]
    pub graph: PyAsynchronousGraphType,
    #[pyo3(item, default = None)]
    pub active_variables: Option<Vec<VariableIdType>>,
    #[pyo3(item, default = None)]
    pub max_symbolic_size: Option<usize>,
}

/// Corresponds to `AttractorConfig | AsynchronousGraph | BooleanNetwork`.
#[derive(FromPyObject)]
pub enum AttractorConfigOrGraph {
    Graph(PyAsynchronousGraphType),
    Config(PyAttractorConfig),
}

impl PyAttractorConfig {
    pub fn clone_native(&self, py: Python) -> PyResult<AttractorConfig> {
        let mut config = AttractorConfig::new(self.graph.clone_native(py)?);
        if let Some(active_variables) = &self.active_variables {
            config.active_variables =
                VariableIdType::resolve_collection(active_variables.clone(), &config.graph)?;
        }
        if let Some(max_symbolic_size) = self.max_symbolic_size {
            config.max_symbolic_size = max_symbolic_size;
        }
        Ok(config)
    }
}

impl From<AttractorConfigOrGraph> for PyAttractorConfig {
    fn from(value: AttractorConfigOrGraph) -> Self {
        match value {
            AttractorConfigOrGraph::Config(config) => config,
            AttractorConfigOrGraph::Graph(graph) => PyAttractorConfig {
                graph,
                active_variables: None,
                max_symbolic_size: None,
            },
        }
    }
}

impl From<Py<AsynchronousGraph>> for AttractorConfigOrGraph {
    fn from(value: Py<AsynchronousGraph>) -> Self {
        AttractorConfigOrGraph::Graph(PyAsynchronousGraphType::from(value))
    }
}
