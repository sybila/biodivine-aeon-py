use std::collections::HashMap;

use biodivine_hctl_model_checker::model_checking::{
    model_check_multiple_extended_formulae_dirty, model_check_multiple_formulae_dirty,
};
use pyo3::prelude::PyAnyMethods;
use pyo3::types::PyList;
use pyo3::{Bound, Py, PyAny, PyResult, Python, pyclass, pymethods};

use crate::bindings::lib_hctl_model_checker::hctl_formula::HctlFormula;
use crate::bindings::lib_param_bn::symbolic::asynchronous_graph::AsynchronousGraph;
use crate::bindings::lib_param_bn::symbolic::set_colored_vertex::ColoredVertexSet;
use crate::{AsNative, throw_runtime_error, throw_type_error};

#[pyclass(module = "biodivine_aeon", frozen)]
pub struct ModelChecking {
    _dummy: (),
}

#[pymethods]
impl ModelChecking {
    /// Verify the provided HCTL formula or formulas, returning the vertex-color pairs for which
    /// the property holds.
    ///
    /// The property argument can be either a single formula (`str` or `HctlFormula`), in which
    /// case the result is a single `ColoredVertexSet`. Alternatively, you can provide a list
    /// of formulas, in which case the result is a `list[ColoredVertexSet]`.
    ///
    /// If the formulas contain extended propositions or quantifier domains, you should provide
    /// a `substitution` map which assigns each proposition a set of valid vertex-color pairs
    /// (the algorithm fails if the used extended propositions cannot be resolved).
    ///
    /// *The following only applies to HCTL formulas that use quantified state variables (i.e.
    /// "plain" CTL formulas do not need this):*
    ///
    /// Note that the provided `AsynchronousGraph` must contain enough symbolic variables
    /// to successfully represent all quantified variables in the provided formulae. You can
    /// create such graph using `AsynchronousGraph.mk_for_model_checking`, or by manually creating
    /// enough extra symbolic variables for each variable in the `SymbolicContext`.
    ///
    /// The resulting `ColoredVertexSet` uses the symbolic encoding of the provided
    /// `AsynchronousGraph` that supports the extra symbolic variables necessary for model
    /// checking. If you want to output the result in a way that is compatible with the "default"
    /// `AsynchronousGraph` representation (i.e. `AsynchronousGraph(network)`), you can use
    /// `AsynchronousGraph.transfer_from` to translate the sets into the "default"
    /// symbolic encoding.
    #[staticmethod]
    #[pyo3(signature = (graph, property, substitution = None))]
    pub fn verify<'a>(
        py: Python<'a>,
        graph: &AsynchronousGraph,
        property: &Bound<'a, PyAny>,
        substitution: Option<HashMap<String, ColoredVertexSet>>,
    ) -> PyResult<Bound<'a, PyAny>> {
        // Extract properties. This could be either one property, or a list of properties.
        let mut properties = Vec::new();
        let mut is_singular = true;
        if let Ok(prop) = property.extract::<HctlFormula>() {
            properties.push(prop.__str__());
        } else if let Ok(prop_str) = property.extract::<String>() {
            properties.push(prop_str);
        } else if let Ok(prop_list) = property.downcast::<PyList>() {
            is_singular = false;
            for x in prop_list {
                if let Ok(prop) = x.extract::<HctlFormula>() {
                    properties.push(prop.__str__());
                } else if let Ok(prop_str) = x.extract::<String>() {
                    properties.push(prop_str);
                } else {
                    return throw_type_error(format!(
                        "Expected `str` or `HctlFormula`. Got {x:?}."
                    ));
                }
            }
        } else {
            return throw_type_error(format!(
                "Expected `str`, `HctlFormula`, or `list`. Got {property:?}."
            ));
        }

        let properties = properties.iter().map(|it| it.as_str()).collect::<Vec<_>>();

        let result = if let Some(substitution) = substitution {
            let native = substitution
                .into_iter()
                .map(|(a, b)| (a, b.as_native().clone()))
                .collect::<HashMap<_, _>>();
            model_check_multiple_extended_formulae_dirty(properties, graph.as_native(), &native)
        } else {
            // Model-check as normal properties.
            model_check_multiple_formulae_dirty(properties, graph.as_native())
        };

        // Perform the necessary type conversions to return either a single element, or
        // a list of elements, depending on context. There's probably a nicer way to do this,
        // but should be good enough for now.
        match result {
            Err(e) => throw_runtime_error(e),
            Ok(result) => {
                if is_singular {
                    let item = result.into_iter().next().unwrap();
                    let result = ColoredVertexSet::mk_native(graph.symbolic_context(), item);
                    Ok(Py::new(py, result)?.into_bound(py).into_any())
                } else {
                    let result_iter = result
                        .into_iter()
                        .map(|it| {
                            Py::new(
                                py,
                                ColoredVertexSet::mk_native(graph.symbolic_context(), it),
                            )
                        })
                        .collect::<PyResult<Vec<Py<ColoredVertexSet>>>>()?;
                    let result_list = PyList::new(py, result_iter)?;

                    Ok(result_list.into_any())
                }
            }
        }
    }
}
