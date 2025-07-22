use crate::bindings::lib_param_bn::symbolic::asynchronous_graph::AsynchronousGraph;
use crate::bindings::lib_param_bn::symbolic::set_colored_space::ColoredSpaceSet;
use crate::bindings::lib_param_bn::symbolic::symbolic_space_context::SymbolicSpaceContext;
use crate::{AsNative, global_log_level};
use pyo3::prelude::*;

#[pyclass(module = "biodivine_aeon", frozen)]
pub struct TrapSpaces {
    _dummy: (),
}

#[pymethods]
impl TrapSpaces {
    /// Computes the coloured set of "essential" trap spaces of a Boolean network.
    ///
    /// A trap space is essential if it cannot be further reduced through percolation. In general, every
    /// minimal trap space is always essential.
    #[staticmethod]
    #[pyo3(signature = (ctx, graph, restriction = None))]
    pub fn essential_symbolic(
        py: Python,
        ctx: Py<SymbolicSpaceContext>,
        graph: &AsynchronousGraph,
        restriction: Option<&ColoredSpaceSet>,
    ) -> PyResult<ColoredSpaceSet> {
        let unit = ctx
            .get()
            .as_native()
            .mk_unit_colored_spaces(graph.as_native());
        let restriction = restriction.map(|it| it.as_native()).unwrap_or(&unit);
        let result = biodivine_lib_param_bn::trap_spaces::TrapSpaces::_essential_symbolic(
            ctx.get().as_native(),
            graph.as_native(),
            restriction,
            global_log_level(py)?,
            &|| py.check_signals(),
        )?;
        Ok(ColoredSpaceSet::wrap_native(ctx.clone(), result))
    }

    /// Computes the minimal coloured trap spaces of the provided `network` within the specified
    /// `restriction` set.
    ///
    /// Currently, this method always slower than [Self::essential_symbolic], because it first has to compute
    /// the essential set.
    #[staticmethod]
    #[pyo3(signature = (ctx, graph, restriction = None))]
    pub fn minimal_symbolic(
        py: Python,
        ctx: Py<SymbolicSpaceContext>,
        graph: &AsynchronousGraph,
        restriction: Option<&ColoredSpaceSet>,
    ) -> PyResult<ColoredSpaceSet> {
        let unit = ctx
            .get()
            .as_native()
            .mk_unit_colored_spaces(graph.as_native());
        let restriction = restriction.map(|it| it.as_native()).unwrap_or(&unit);
        let result = biodivine_lib_param_bn::trap_spaces::TrapSpaces::_minimal_symbolic(
            ctx.get().as_native(),
            graph.as_native(),
            restriction,
            global_log_level(py)?,
            &|| py.check_signals(),
        )?;
        Ok(ColoredSpaceSet::wrap_native(ctx.clone(), result))
    }

    /// Compute the inclusion-minimal spaces within a particular subset.
    #[staticmethod]
    pub fn minimize(
        py: Python,
        ctx: Py<SymbolicSpaceContext>,
        set: &ColoredSpaceSet,
    ) -> PyResult<ColoredSpaceSet> {
        let result = biodivine_lib_param_bn::trap_spaces::TrapSpaces::_minimize(
            ctx.get().as_native(),
            set.as_native(),
            global_log_level(py)?,
            &|| py.check_signals(),
        )?;
        Ok(ColoredSpaceSet::wrap_native(ctx.clone(), result))
    }

    /// Compute the inclusion-maximal spaces within a particular subset.
    #[staticmethod]
    pub fn maximize(
        py: Python,
        ctx: Py<SymbolicSpaceContext>,
        set: &ColoredSpaceSet,
    ) -> PyResult<ColoredSpaceSet> {
        let result = biodivine_lib_param_bn::trap_spaces::TrapSpaces::_maximize(
            ctx.get().as_native(),
            set.as_native(),
            global_log_level(py)?,
            &|| py.check_signals(),
        )?;
        Ok(ColoredSpaceSet::wrap_native(ctx.clone(), result))
    }
}
