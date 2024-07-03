//pub mod lib_bdd;
//pub mod lib_param_bn;
//pub mod aeon;
//pub mod pbn_control;
//pub mod lib_hctl_model_checker;
//pub mod bn_classifier;

use pyo3::{PyResult, Python};

pub mod lib_bdd;
pub mod lib_hctl_model_checker;
pub mod lib_param_bn;

pub fn global_interrupt() -> PyResult<()> {
    Python::with_gil(|py| py.check_signals())
}
