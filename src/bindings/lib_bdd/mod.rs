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
    module.add_class::<PyBdd>()?;
    module.add_class::<PyBddVariable>()?;
    module.add_class::<PyBddVariableSet>()?;
    module.add_class::<PyBooleanExpression>()?;
    module.add_class::<PyBddVariableSetBuilder>()?;
    Ok(())
}

#[pyclass(name = "Bdd")]
#[derive(Clone, Eq, PartialEq, Hash, Wrapper)]
pub struct PyBdd(Bdd);

#[pyclass(name = "BddValuationIterator")]
pub struct PyBddValuationIterator(BddSatisfyingValuations<'static>, Bdd);
#[pyclass(name = "BddClauseIterator")]
pub struct PyBddClauseIterator(BddPathIterator<'static>, Bdd);

#[pyclass(name = "BddValuation")]
#[derive(Clone, Eq, PartialEq, Hash, Wrapper)]
pub struct PyBddValuation(BddValuation);
#[pyclass(name = "BddPartialValuation")]
#[derive(Clone, Eq, PartialEq, Hash, Wrapper)]
pub struct PyBddPartialValuation(BddPartialValuation);

/// An identifier of a Boolean decision variable used within a `Bdd`.
#[pyclass(name = "BddVariable")]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Wrapper)]
pub struct PyBddVariable(BddVariable);

/// An object which manages a set of `Bdd` decision variables and makes it possible to
/// create new `Bdd` objects using these variables.
///
/// In particular, it maps `BddVariable` identifiers to actual variable names.
#[pyclass(name = "BddVariableSet")]
#[derive(Clone, Wrapper)]
pub struct PyBddVariableSet(BddVariableSet);

/// Abstract syntax tree of an expression which describes a particular Boolean formula.
///
/// TODO: This class currently has no API except for conversion from and into `String`.
#[pyclass(name = "BooleanExpression")]
#[derive(Clone, Eq, PartialEq, Wrapper)]
pub struct PyBooleanExpression(BooleanExpression);

/// A builder object that lets you gradually construct a `BddVariableSet` instead of supplying
/// all variable names at once.
#[pyclass(name = "BddVariableSetBuilder")]
pub struct PyBddVariableSetBuilder(BddVariableSetBuilder, Vec<String>);

// Note that above, the string vector is necessary because `BddVariableSetBuilder` does not
// implement clone and there is no way to actually destroy it during final conversion
// to `BddVariableSet`. Hence we have to make a copy of the builder using the saved variable names.
