use pyo3::types::{PyModule, PyModuleMethods};
use pyo3::{Bound, PyResult};

pub mod bdd;
pub mod bdd_pointer;
pub mod bdd_valuation;
pub mod bdd_variable;
pub mod bdd_variable_set;
pub mod bdd_variable_set_builder;
pub mod boolean_expression;
pub mod op_function;

pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<bdd::Bdd>()?;
    module.add_class::<bdd::NaiveSampler>()?;
    module.add_class::<bdd::UniformValuationSampler>()?;
    module.add_class::<bdd::_BddValuationIterator>()?;
    module.add_class::<bdd::_BddClauseIterator>()?;
    module.add_class::<boolean_expression::BooleanExpression>()?;
    module.add_class::<bdd_variable::BddVariable>()?;
    module.add_class::<bdd_pointer::BddPointer>()?;
    module.add_class::<bdd_valuation::BddValuation>()?;
    module.add_class::<bdd_valuation::BddPartialValuation>()?;
    module.add_class::<bdd_variable_set::BddVariableSet>()?;
    module.add_class::<bdd_variable_set_builder::BddVariableSetBuilder>()?;
    Ok(())
}
