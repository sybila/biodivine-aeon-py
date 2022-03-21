use crate::bindings::lib_bdd::{PyBddVariable, PyBddVariableSet, PyBddVariableSetBuilder};
use biodivine_lib_bdd::BddVariableSetBuilder;
use pyo3::prelude::*;
use pyo3::types::PyList;

#[pymethods]
impl PyBddVariableSetBuilder {
    /// Create a new, empty `BddVariableSetBuilder`.
    #[new]
    pub fn new() -> Self {
        PyBddVariableSetBuilder(BddVariableSetBuilder::new(), Vec::new())
    }

    /// Create a new variable with the given name. Returns a `BddVariable` identifier that can be
    /// later used to create and query actual `Bdd` objects.
    pub fn make_variable(&mut self, name: &str) -> PyBddVariable {
        let var = self.0.make_variable(name).into();
        self.1.push(name.to_string());
        var
    }

    /// Create multiple variables with the names supplied as a list. Returns a list of
    /// the corresponding `BddVariable` identifiers.
    pub fn make(&mut self, py: Python, names: &PyList) -> PyResult<PyObject> {
        let mut result: Vec<PyBddVariable> = Vec::new();
        for i in 0..names.len() {
            let name = names.get_item(i)?.extract::<String>()?;
            let var = self.0.make_variable(name.as_str());
            self.1.push(name);
            result.push(var.into());
        }
        Ok(result.into_py(py))
    }

    /// Convert this `BddVariableSetBuilder` to an actual `BddVariableSet`.
    pub fn build(&self) -> PyBddVariableSet {
        let mut builder = BddVariableSetBuilder::new();
        for name in &self.1 {
            builder.make_variable(name);
        }
        builder.build().into()
    }
}
