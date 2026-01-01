use pyo3::prelude::PyModuleMethods;
use pyo3::{Bound, PyResult, types::PyModule};

pub mod attractors;
pub mod fixed_points;
pub mod graph_representation;
pub mod percolation;
pub mod reachability;
pub mod scc;
pub mod token_python;
pub mod trap_spaces;

pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    pyo3_log::init();

    fixed_points::register(module)?;
    trap_spaces::register(module)?;
    percolation::register(module)?;
    module.add_class::<reachability::Reachability>()?;
    module.add_class::<attractors::Attractors>()?;
    module.add_class::<scc::Scc>()?;

    Ok(())
}
