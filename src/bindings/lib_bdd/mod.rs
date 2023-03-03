use biodivine_lib_bdd::boolean_expression::BooleanExpression;
use biodivine_lib_bdd::{
    Bdd, BddPartialValuation, BddPathIterator, BddSatisfyingValuations, BddValuation, BddVariable,
    BddVariableSet, BddVariableSetBuilder,
};
use macros::Wrapper;
use pyo3::prelude::*;
use pyo3::PyResult;

mod _impl_bdd;
mod _impl_bdd_iterator;
mod _impl_bdd_valuation;
mod _impl_bdd_variable;
mod _impl_bdd_variable_set;
mod _impl_bdd_variable_set_builder;
mod _impl_boolean_expression;

pub(crate) fn register(module: &PyModule) -> PyResult<()> {
    module.add_class::<PyBooleanExpression>()?;
    module.add_class::<PyBdd>()?;
    module.add_class::<PyBddPartialValuation>()?;
    module.add_class::<PyBddValuation>()?;
    module.add_class::<PyBddClauseIterator>()?;
    module.add_class::<PyBddValuationIterator>()?;
    module.add_class::<PyBddVariable>()?;
    module.add_class::<PyBddVariableSet>()?;
    module.add_class::<PyBddVariableSetBuilder>()?;
    Ok(())
}

#[pyclass(name = "BooleanExpression")]
#[derive(Clone, Eq, PartialEq, Wrapper)]
pub struct PyBooleanExpression(BooleanExpression);

#[pyclass(name = "Bdd")]
#[derive(Clone, Eq, PartialEq, Hash, Wrapper)]
pub struct PyBdd(Bdd);

#[pyclass(name = "BddPartialValuation")]
#[derive(Clone, Eq, PartialEq, Hash, Wrapper)]
pub struct PyBddPartialValuation(BddPartialValuation);

#[pyclass(name = "BddValuation")]
#[derive(Clone, Eq, PartialEq, Hash, Wrapper)]
pub struct PyBddValuation(BddValuation);

#[pyclass(name = "BddClauseIterator")]
pub struct PyBddClauseIterator(Py<PyBdd>, BddPathIterator<'static>);

#[pyclass(name = "BddValuationIterator")]
pub struct PyBddValuationIterator(Py<PyBdd>, BddSatisfyingValuations<'static>);

#[pyclass(name = "BddVariable")]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Wrapper)]
pub struct PyBddVariable(BddVariable);

#[pyclass(name = "BddVariableSet")]
#[derive(Clone, Wrapper)]
pub struct PyBddVariableSet(BddVariableSet);

// The string vector in `PyBddVariableSetBuilder` is used to track all created variables.
// The reason for it that `BddVariableSetBuilder` cannot be cloned and since we cannot own it
// in a PyO3 mapped function, we cannot transform it into `BddVariableSet`. Hence we create
// the `BddVariableSet` from the string vector instead.

#[pyclass(name = "BddVariableSetBuilder")]
pub struct PyBddVariableSetBuilder(BddVariableSetBuilder, Vec<String>);
