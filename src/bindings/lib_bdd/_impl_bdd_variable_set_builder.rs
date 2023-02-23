use crate::bindings::lib_bdd::{PyBddVariable, PyBddVariableSet, PyBddVariableSetBuilder};
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
        PyBddVariableSetBuilder(BddVariableSetBuilder::new(), Vec::new())
    }

    pub fn make(&mut self, name: &str) -> PyBddVariable {
        let var = self.0.make_variable(name).into();
        self.1.push(name.to_string());
        var
    }

    pub fn make_all(&mut self, names: Vec<&str>) -> PyResult<Vec<PyBddVariable>> {
        let mut result: Vec<PyBddVariable> = Vec::new();
        for name in names {
            result.push(self.make(name));
        }
        Ok(result)
    }

    pub fn build(&self) -> PyBddVariableSet {
        let mut builder = BddVariableSetBuilder::new();
        for name in &self.1 {
            builder.make_variable(name);
        }
        builder.build().into()
    }
}
