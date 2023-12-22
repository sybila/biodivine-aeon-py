use crate::bindings;
use pyo3::types::PyModule;
use pyo3::PyResult;

pub mod bdd;
pub mod bdd_pointer;
pub mod bdd_valuation;
pub mod bdd_variable;
pub mod bdd_variable_set;
pub mod bdd_variable_set_builder;
pub mod boolean_expression;

pub fn register(module: &PyModule) -> PyResult<()> {
    module.add_class::<bdd::Bdd>()?;
    module.add_class::<boolean_expression::BooleanExpression>()?;
    module.add_class::<bdd_variable::BddVariable>()?;
    module.add_class::<bdd_pointer::BddPointer>()?;
    module.add_class::<bdd_valuation::BddValuation>()?;
    module.add_class::<bdd_valuation::BddPartialValuation>()?;
    module.add_class::<bdd_variable_set::BddVariableSet>()?;
    module.add_class::<bdd_variable_set_builder::BddVariableSetBuilder>()?;
    Ok(())
}
