use crate::bindings::lib_bdd_2::bdd_variable::BddVariable;
use crate::bindings::lib_bdd_2::bdd_variable_set::BddVariableSet;
use crate::{throw_runtime_error, AsNative};
use macros::Wrapper;
use pyo3::basic::CompareOp;
use pyo3::prelude::*;

/// A utility class that can be used to build `BddVariableSet` iteratively instead of
/// providing all the variable names at once.
///
/// ```python
/// builder = BddVariableSetBuilder()
/// x = builder.add("var_x")
/// a, b, c = builder.add_all(["a", "b", "c"])
/// ctx = builder.build()
/// assert ctx.var_count() == 4
/// assert ctx.get_variable_name(b) == "b"
/// assert ctx.find_variable("x") is None
/// assert ctx.find_variable("var_x") == x
/// ```
#[pyclass(module = "biodivine_aeon")]
#[derive(Clone, Wrapper)]
pub struct BddVariableSetBuilder(biodivine_lib_bdd::BddVariableSetBuilder);

#[pymethods]
impl BddVariableSetBuilder {
    #[new]
    #[pyo3(signature = (variables = None))]
    fn new(variables: Option<Vec<&str>>) -> BddVariableSetBuilder {
        let mut inner = biodivine_lib_bdd::BddVariableSetBuilder::new();
        if let Some(variables) = variables {
            inner.make_variables(&variables);
        }
        BddVariableSetBuilder(inner)
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
        let self_names = self.__getstate__();
        let other_names = other.__getstate__();
        match op {
            CompareOp::Eq => Ok(self_names.eq(&other_names)),
            CompareOp::Ne => Ok(self_names.ne(&other_names)),
            _ => throw_runtime_error("`BddVariableSetBuilder` cannot be ordered."),
        }
    }

    fn __len__(&self) -> usize {
        usize::from(self.0.clone().build().num_vars())
    }

    fn __str__(&self) -> String {
        let size = self.as_native().clone().build().num_vars();
        format!("BddVariableSetBuilder(len = {})", size)
    }

    fn __repr__(&self) -> String {
        let names = self.__getstate__();
        format!("BddVariableSetBuilder({:?})", names)
    }

    fn __getstate__(&self) -> Vec<String> {
        let vars = self.as_native().clone().build();
        vars.variables()
            .into_iter()
            .map(|it| vars.name_of(it))
            .collect()
    }

    fn __setstate__(&mut self, state: Vec<&str>) {
        self.0 = biodivine_lib_bdd::BddVariableSetBuilder::new();
        self.0.make_variables(&state);
    }

    /// Add a single new variable to this `BddVariableSetBuilder`.
    ///
    /// Panics if the variable already exists.
    fn add(&mut self, name: &str) -> BddVariable {
        self.as_native_mut().make_variable(name).into()
    }

    /// Add a collection of new variables to this `BddVariableSetBuilder`.
    ///
    /// Panics if some of variables already exist.
    fn add_all(&mut self, names: Vec<&str>) -> Vec<BddVariable> {
        self.as_native_mut()
            .make_variables(&names)
            .into_iter()
            .map(Into::into)
            .collect()
    }

    /// Build the final `BddVariableSet`.
    fn build(&self) -> BddVariableSet {
        self.as_native().clone().build().into()
    }
}
