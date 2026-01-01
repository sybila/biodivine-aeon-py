use std::collections::HashMap;

use pyo3::{PyResult, pymethods};

use crate::{
    bindings::{
        algorithms::graph_representation::PyAsynchronousGraphType,
        lib_param_bn::variable_id::VariableId,
    },
    internal::algorithms::percolation::{Percolation, PercolationConfig},
};

use super::SubspaceRepresentation;

/// These methods are Python facing wrappers of native methods and thus should not be used from
/// within Rust.
#[pymethods]
impl Percolation {
    /// Create a new `PercolationComp` instance from the given `AsynchronousGraph` or `BooleanNetwork`,
    /// with otherwise default configuration.
    #[staticmethod]
    #[pyo3(name = "create_from")]
    pub fn python_create_from(graph_representation: PyAsynchronousGraphType) -> PyResult<Self> {
        Ok(Percolation(PercolationConfig::python_create_from(
            graph_representation,
        )?))
    }

    /// Create a new `PercolationComp` instance with the given `PercolationConfig`.
    #[staticmethod]
    #[pyo3(name = "with_config")]
    pub fn python_with_config(config: PercolationConfig) -> Self {
        Percolation(config)
    }

    /// Performs a percolation of a single subspace.
    ///
    /// Percolation propagates the values of variables that are guaranteed to be constant in the
    /// given subspace. Note that this function will not overwrite values fixed in the original
    /// space if they percolate to a conflicting value. Also note that the result is a subspace
    /// of the original space, i.e. it does not just contain the newly propagated variables.
    ///
    /// This method should technically work on parametrized networks as well, but the constant
    /// check is performed across all interpretations, hence a lot of sub-spaces will not
    /// percolate meaningfully. We recommend using other symbolic methods for such systems.
    #[pyo3(name = "percolate_subspace")]
    pub fn python_percolate_subspace(
        &self,
        subspace: SubspaceRepresentation,
    ) -> PyResult<HashMap<VariableId, bool>> {
        self.percolate_subspace(subspace.into())
            .map_err(|e| e.into())
            .map(|result| SubspaceRepresentation::from(result).into())
    }
}
