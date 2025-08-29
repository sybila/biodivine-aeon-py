use pyo3::{Py, PyResult, pyclass, pymethods};

use crate::{
    AsNative as _,
    bindings::{
        algorithms::graph_representation::PyGraphRepresentation,
        lib_param_bn::symbolic::{
            asynchronous_graph::AsynchronousGraph, set_colored_space::ColoredSpaceSet,
            symbolic_space_context::SymbolicSpaceContext,
        },
    },
};

use super::PyTrapSpacesConfig;

/// Implments trap spaces search over an `AsynchronousGraph` and its
/// `SymbolicSpaceContext`.
#[pyclass(name = "TrapSpacesComp", module = "biodivine_aeon", frozen)]
pub struct PyTrapSpaces(PyTrapSpacesConfig);

/// These methods are Python facing wrappers of native methods and thus should not be used from
/// within Rust.
#[pymethods]
impl PyTrapSpaces {
    /// Create a new `TrapSpacesComp` instance with the given `BooleanNetwork`,
    /// `AsynchronousGraph` is currently not supported.
    #[staticmethod]
    pub fn create_from(graph_representation: PyGraphRepresentation) -> PyResult<Self> {
        Ok(PyTrapSpaces(PyTrapSpacesConfig::try_from(
            graph_representation,
        )?))
    }

    /// Create a new `TrapSpacesComp` instance with the given `AsynchronousGraph` and
    /// `SymbolicSpaceContext`.
    #[staticmethod]
    pub fn create_from_graph_with_context(
        graph: &AsynchronousGraph,
        ctx: Py<SymbolicSpaceContext>,
    ) -> Self {
        PyTrapSpaces(PyTrapSpacesConfig::create_from_graph_with_context(
            graph, ctx,
        ))
    }

    /// Create a new `TrapSpacesComp` instance with the given `TrapSpacesConfig`.
    #[staticmethod]
    pub fn with_config(config: PyTrapSpacesConfig) -> Self {
        PyTrapSpaces(config)
    }

    /// Computes the coloured set of "essential" trap spaces of a Boolean network.
    ///
    /// A trap space is essential if it cannot be further reduced through percolation. In general, every
    /// minimal trap space is always essential.
    pub fn essential_symbolic(&self) -> PyResult<ColoredSpaceSet> {
        Ok(ColoredSpaceSet::wrap_native(
            self.0.ctx.clone(),
            self.0.inner.essential_symbolic()?,
        ))
    }

    /// Computes the minimal coloured trap spaces of the underlying `graph` within the configured
    /// `restriction` set.
    ///
    /// Currently, this method always slower than `essential_symbolic()`, because it first has to compute
    /// the essential set.
    pub fn minimal_symbolic(&self) -> PyResult<ColoredSpaceSet> {
        Ok(ColoredSpaceSet::wrap_native(
            self.0.ctx.clone(),
            self.0.inner.minimal_symbolic()?,
        ))
    }

    /// Compute the inclusion-minimal spaces within a particular subset.
    pub fn minimize(&self, set: &ColoredSpaceSet) -> PyResult<ColoredSpaceSet> {
        Ok(ColoredSpaceSet::wrap_native(
            self.0.ctx.clone(),
            self.0.inner.minimize(set.as_native())?,
        ))
    }

    /// Compute the inclusion-maximal spaces within a particular subset.
    pub fn maximize(&self, set: &ColoredSpaceSet) -> PyResult<ColoredSpaceSet> {
        Ok(ColoredSpaceSet::wrap_native(
            self.0.ctx.clone(),
            self.0.inner.maximize(set.as_native())?,
        ))
    }
}
