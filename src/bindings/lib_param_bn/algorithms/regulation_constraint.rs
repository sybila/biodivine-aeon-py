use crate::bindings::lib_bdd::bdd::Bdd;
use crate::bindings::lib_param_bn::regulatory_graph::RegulatoryGraph;
use crate::bindings::lib_param_bn::symbolic::symbolic_context::SymbolicContext;
use crate::bindings::lib_param_bn::NetworkVariableContext;
use crate::AsNative;
use pyo3::types::PyDict;
use pyo3::{pyclass, pymethods, Bound, PyAny, PyResult, Python};

/// An "algorithm object" that can be used to compute symbolic constraints that correspond
/// to typical properties of model regulations (i.e. monotonicity and essentiality).
///
/// However, you can use this to create symbolic constraints for arbitrary symbolic functions,
/// not just the update functions of a BN.
#[pyclass(module = "biodivine_aeon", frozen)]
pub struct RegulationConstraint {
    _dummy: (),
}

#[pymethods]
impl RegulationConstraint {
    /// Compute a `Bdd` which is satisfied exactly by those function interpretations for which
    /// the given `function` is positively monotonous in the specified input `variable`.
    ///
    /// Note that the result only depends on the "parameter" variables of the given
    /// `SymbolicContext`. You can thus directly convert it to a `ColorSet` by calling
    /// `ColorSet(context, result)`.
    ///
    /// Also note that (at the moment), the input variable must be one of the network variables.
    /// For example, it cannot be one of the extra variables.
    ///
    /// You can also use this function to check if a non-parametrized function is monotonous:
    /// simply check if the resulting `Bdd` is a `true` function.
    #[staticmethod]
    pub fn mk_activation(
        context: &SymbolicContext,
        function: &Bdd,
        variable: &Bound<'_, PyAny>,
    ) -> PyResult<Bdd> {
        let var = context.resolve_network_variable(variable)?;
        let native =
            biodivine_lib_param_bn::symbolic_async_graph::RegulationConstraint::mk_activation(
                context.as_native(),
                function.as_native(),
                var,
            );

        Ok(Bdd::new_raw_2(context.bdd_variable_set(), native))
    }

    /// Compute a `Bdd` which is satisfied exactly by those function interpretations for which
    /// the given `function` is negatively monotonous in the specified input `variable`.
    ///
    /// Note that the result only depends on the "parameter" variables of the given
    /// `SymbolicContext`. You can thus directly convert it to a `ColorSet` by calling
    /// `ColorSet(context, result)`.
    ///
    /// Also note that (at the moment), the input variable must be one of the network variables.
    /// For example, it cannot be one of the extra variables.
    ///
    /// You can also use this function to check if a non-parametrized function is monotonous:
    /// simply check if the resulting `Bdd` is a `true` function.
    #[staticmethod]
    pub fn mk_inhibition(
        context: &SymbolicContext,
        function: &Bdd,
        variable: &Bound<'_, PyAny>,
    ) -> PyResult<Bdd> {
        let var = context.resolve_network_variable(variable)?;
        let native =
            biodivine_lib_param_bn::symbolic_async_graph::RegulationConstraint::mk_inhibition(
                context.as_native(),
                function.as_native(),
                var,
            );

        Ok(Bdd::new_raw_2(context.bdd_variable_set(), native))
    }

    /// Compute a `Bdd` which is satisfied exactly by those function interpretations for which
    /// the given `function` is has the specified `variable` as an essential input (i.e. it
    /// plays a role in the function's outcome).
    ///
    /// Note that the result only depends on the "parameter" variables of the given
    /// `SymbolicContext`. You can thus directly convert it to a `ColorSet` by calling
    /// `ColorSet(context, result)`.
    ///
    /// Also note that (at the moment), the input variable must be one of the network variables.
    /// For example, it cannot be one of the extra variables.
    ///
    /// You can also use this function to check if a non-parametrized function has an essential
    /// input: simply check if the resulting `Bdd` is a `true` function.
    #[staticmethod]
    pub fn mk_essential(
        context: &SymbolicContext,
        function: &Bdd,
        variable: &Bound<'_, PyAny>,
    ) -> PyResult<Bdd> {
        let var = context.resolve_network_variable(variable)?;
        let native =
            biodivine_lib_param_bn::symbolic_async_graph::RegulationConstraint::mk_observability(
                context.as_native(),
                function.as_native(),
                var,
            );

        Ok(Bdd::new_raw_2(context.bdd_variable_set(), native))
    }

    /// This method takes a symbolic (parametrized) `function` and two network variables
    /// (`source` and `target`).
    ///
    /// It assumes the symbolic `function` represents the update function of the `target`
    /// variable. It then tries to infer the most specific `IdRegulation` that is valid for
    /// every interpretation of the provided function.
    ///
    /// If the function is not parametrized, this simply infers the monotonicity and essentiality
    /// of `source` in the given function.
    ///
    /// Note that the function returns `None` if no interpretation of the function depends on
    /// `source`. It returns `essential=False` only when the function has more than one
    /// interpretation, such that some depend on `source`, but not all.
    #[staticmethod]
    pub fn infer_sufficient_regulation<'a>(
        py: Python<'a>,
        context: &SymbolicContext,
        source: &Bound<'_, PyAny>,
        target: &Bound<'_, PyAny>,
        function: &Bdd,
    ) -> PyResult<Option<Bound<'a, PyDict>>> {
        let source = context.resolve_network_variable(source)?;
        let target = context.resolve_network_variable(target)?;
        let native = biodivine_lib_param_bn::symbolic_async_graph::RegulationConstraint::infer_sufficient_regulation(
            context.as_native(),
            source,
            target,
            function.as_native()
        );

        native
            .map(|it| RegulatoryGraph::encode_regulation(py, &it))
            .transpose()
    }
}
