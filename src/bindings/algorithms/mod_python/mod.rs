use pyo3::{types::PyModule, Bound, PyResult};

use super::{fixed_points, percolation, reachability, trap_spaces};

pub mod graph_representation;

pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    pyo3_log::init();

    fixed_points::register(module)?;
    trap_spaces::register(module)?;
    percolation::register(module)?;
    reachability::register(module)?;

    Ok(())
}
