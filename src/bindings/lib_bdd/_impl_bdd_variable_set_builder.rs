use crate::bindings::lib_bdd::{PyBddVariable, PyBddVariableSet, PyBddVariableSetBuilder};
use crate::AsNative;
use biodivine_lib_bdd::BddVariableSetBuilder;
use pyo3::prelude::*;

impl Default for PyBddVariableSetBuilder {
    fn default() -> Self {
        PyBddVariableSetBuilder::new()
    }
}

#[pymethods]
impl PyBddVariableSetBuilder {
    #[new]
    pub fn new() -> Self {
        PyBddVariableSetBuilder(BddVariableSetBuilder::new())
    }

    pub fn make(&mut self, name: &str) -> PyBddVariable {
        self.as_native_mut().make_variable(name).into()
    }

    pub fn make_all(&mut self, names: Vec<&str>) -> PyResult<Vec<PyBddVariable>> {
        let mut result: Vec<PyBddVariable> = Vec::new();
        for name in names {
            result.push(self.make(name));
        }
        Ok(result)
    }

    pub fn build(&self) -> PyBddVariableSet {
        self.as_native().clone().build().into()
    }
}
