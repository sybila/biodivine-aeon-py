use crate::AsNative;
use crate::bindings::algorithms::scc::scc_config::{PySccConfig, SccConfigOrGraph};
use crate::bindings::lib_param_bn::symbolic::set_colored_vertex::ColoredVertexSet;
use biodivine_algo_bdd_scc::scc::{ChainScc, FwdBwdScc};
use computation_process::Stateful;
use pyo3::{PyResult, Python, pyclass, pymethods};

pub mod scc_config;

#[pyclass(module = "biodivine_aeon", frozen)]
pub struct Scc {
    _dummy: (),
}

#[pymethods]
impl Scc {
    /// Compute all non-trivial strongly connected components of an asynchronous state-transition
    /// graph using the quadratic *forward-backward* algorithm (with reachability saturation).
    ///
    /// See also `SccConfig` for more information about algorithm configuration.
    ///
    /// Note that we consider all single-state components to be trivial, meaning this method will
    /// also skip all sink states.
    #[staticmethod]
    #[pyo3(signature = (config, initial_set = None))]
    pub fn fwd_bwd(
        config: SccConfigOrGraph,
        initial_set: Option<&ColoredVertexSet>,
        py: Python,
    ) -> PyResult<Vec<ColoredVertexSet>> {
        let py_config = PySccConfig::from(config);
        let py_ctx = py_config.graph.clone_py_context(py)?;
        let config = py_config.clone_native(py)?;

        let initial_set = if let Some(r) = initial_set {
            r.as_native().clone()
        } else {
            config.graph.mk_unit_colored_vertices()
        };

        cancel_this::on_python(|| {
            let mut result = Vec::new();
            for scc in FwdBwdScc::configure(config, initial_set).take(py_config.solution_count) {
                let scc = ColoredVertexSet::mk_native(py_ctx.clone(), scc?);
                result.push(scc);
            }
            Ok(result)
        })
    }

    /// Compute all non-trivial strongly connected components of an asynchronous state-transition
    /// graph using the linear *chain* algorithm (with reachability saturation; note that this
    /// means the algorithm is not *always* linear, but it helps significantly in practice).
    ///
    /// See also `SccConfig` for more information about algorithm configuration.
    ///
    /// Note that we consider all single-state components to be trivial, meaning this method will
    /// also skip all sink states.
    #[staticmethod]
    #[pyo3(signature = (config, initial_set = None))]
    pub fn chain(
        config: SccConfigOrGraph,
        initial_set: Option<&ColoredVertexSet>,
        py: Python,
    ) -> PyResult<Vec<ColoredVertexSet>> {
        let py_config = PySccConfig::from(config);
        let py_ctx = py_config.graph.clone_py_context(py)?;
        let config = py_config.clone_native(py)?;

        let initial_set = if let Some(r) = initial_set {
            r.as_native().clone()
        } else {
            config.graph.mk_unit_colored_vertices()
        };

        cancel_this::on_python(|| {
            let mut result = Vec::new();
            for scc in ChainScc::configure(config, initial_set).take(py_config.solution_count) {
                let scc = ColoredVertexSet::mk_native(py_ctx.clone(), scc?);
                result.push(scc);
            }
            Ok(result)
        })
    }
}
