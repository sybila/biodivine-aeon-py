use crate::bindings::lib_bdd::bdd_variable::BddVariable;
use crate::bindings::lib_bdd::bdd_variable_set::BddVariableSet;
use crate::pyo3_utils::richcmp_eq_by_key;
use crate::AsNative;
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
    fn new(variables: Option<Vec<String>>) -> BddVariableSetBuilder {
        let mut inner = biodivine_lib_bdd::BddVariableSetBuilder::new();
        if let Some(variables) = variables {
            let str_variables = variables.iter().map(|it| it.as_str()).collect::<Vec<_>>();
            inner.make_variables(str_variables.as_slice());
        }
        BddVariableSetBuilder(inner)
    }

    fn __richcmp__(&self, py: Python, other: &Self, op: CompareOp) -> PyResult<Py<PyAny>> {
        richcmp_eq_by_key(py, op, self, other, |x| x.__getstate__())
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

    fn __setstate__(&mut self, state: Vec<String>) {
        self.0 = biodivine_lib_bdd::BddVariableSetBuilder::new();
        let str_variables = state.iter().map(|it| it.as_str()).collect::<Vec<_>>();
        self.0.make_variables(str_variables.as_slice());
    }

    /// Add a single new variable to this `BddVariableSetBuilder`.
    ///
    /// Panics if the variable already exists.
    fn add(&mut self, name: &str) -> BddVariable {
        self.as_native_mut().make_variable(name).into()
    }

    /// Add a collection of new variables to this `BddVariableSetBuilder`.
    ///
    /// Panics if some of the variables already exist.
    fn add_all(&mut self, names: Vec<String>) -> Vec<BddVariable> {
        let str_variables = names.iter().map(|it| it.as_str()).collect::<Vec<_>>();
        self.as_native_mut()
            .make_variables(str_variables.as_slice())
            .into_iter()
            .map(Into::into)
            .collect()
    }

    /// Build the final `BddVariableSet`.
    fn build(&self) -> BddVariableSet {
        self.as_native().clone().build().into()
    }
}
