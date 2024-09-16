use biodivine_pbn_control::control::PhenotypeOscillationType;
use pyo3::prelude::*;

mod asynchronous_perturbation_graph;
mod control;
mod model_perturbation;
mod set_colored_perturbation;
mod set_perturbation;

use crate::bindings::pbn_control::control::Control;
use crate::bindings::pbn_control::set_colored_perturbation::_ColorPerturbationModelIterator;
use crate::bindings::pbn_control::set_perturbation::_PerturbationModelIterator;
use crate::throw_type_error;
pub use asynchronous_perturbation_graph::AsynchronousPerturbationGraph;
pub use model_perturbation::PerturbationModel;
pub use set_colored_perturbation::ColoredPerturbationSet;
pub use set_perturbation::PerturbationSet;

pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<AsynchronousPerturbationGraph>()?;
    module.add_class::<PerturbationModel>()?;
    module.add_class::<PerturbationSet>()?;
    module.add_class::<ColoredPerturbationSet>()?;
    module.add_class::<_PerturbationModelIterator>()?;
    module.add_class::<_ColorPerturbationModelIterator>()?;
    module.add_class::<Control>()?;
    Ok(())
}

pub fn extract_phenotype_type(osc: &str) -> PyResult<PhenotypeOscillationType> {
    match osc {
        "forbidden" | "Forbidden" => Ok(PhenotypeOscillationType::Forbidden),
        "allowed" | "Allowed" => Ok(PhenotypeOscillationType::Allowed),
        "required" | "Required" => Ok(PhenotypeOscillationType::Required),
        _ => throw_type_error(
            "Invalid oscillation type. Expected one of ['forbidden', 'allowed', 'required'].",
        ),
    }
}
