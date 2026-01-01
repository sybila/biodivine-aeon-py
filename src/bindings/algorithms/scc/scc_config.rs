use crate::bindings::algorithms::graph_representation::PyAsynchronousGraphType;
use crate::throw_type_error;
use biodivine_algo_bdd_scc::scc::SccConfig;
use biodivine_algo_bdd_scc::trimming::TrimSetting;
use pyo3::{Borrowed, FromPyObject, PyAny, PyErr, PyResult, Python};

/// Internal helper struct which corresponds to the `SccConfig` typed dictionary and
/// converts to the native [`SccConfig`].
#[derive(FromPyObject)]
pub struct PySccConfig {
    #[pyo3(item)]
    pub graph: PyAsynchronousGraphType,
    #[pyo3(item, default = TrimSettingType::default())]
    pub should_trim: TrimSettingType,
    #[pyo3(item, default = false)]
    pub filter_long_lived: bool,
    // This option will not be necessary once we can export the algorithm as an iterator.
    #[pyo3(item, default = usize::MAX)]
    pub solution_count: usize,
}

/// Corresponds to `SccConfig | AsynchronousGraph | BooleanNetwork`.
#[derive(FromPyObject)]
pub enum SccConfigOrGraph {
    Graph(PyAsynchronousGraphType),
    Config(PySccConfig),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrimSettingType {
    #[default]
    Both,
    Sinks,
    Sources,
    None,
}

impl PySccConfig {
    pub fn clone_native(&self, py: Python) -> PyResult<SccConfig> {
        let mut config = SccConfig::new(self.graph.clone_native(py)?);
        config.should_trim = self.should_trim.into();
        config.filter_long_lived = self.filter_long_lived;
        Ok(config)
    }
}

impl From<TrimSettingType> for TrimSetting {
    fn from(value: TrimSettingType) -> Self {
        match value {
            TrimSettingType::None => TrimSetting::None,
            TrimSettingType::Both => TrimSetting::Both,
            TrimSettingType::Sinks => TrimSetting::Sinks,
            TrimSettingType::Sources => TrimSetting::Sources,
        }
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for TrimSettingType {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        if let Ok(v) = obj.extract::<String>() {
            match v.as_str() {
                "none" => return Ok(TrimSettingType::None),
                "both" => return Ok(TrimSettingType::Both),
                "sinks" => return Ok(TrimSettingType::Sinks),
                "sources" => return Ok(TrimSettingType::Sources),
                _ => (),
            };
        }

        throw_type_error(format!(
            "Expected one of `none`/`both`/`sinks`/`sources`. Got `{obj:?}`."
        ))
    }
}

impl From<SccConfigOrGraph> for PySccConfig {
    fn from(value: SccConfigOrGraph) -> Self {
        match value {
            SccConfigOrGraph::Config(config) => config,
            SccConfigOrGraph::Graph(graph) => PySccConfig {
                graph,
                should_trim: Default::default(),
                filter_long_lived: false,
                solution_count: usize::MAX,
            },
        }
    }
}
