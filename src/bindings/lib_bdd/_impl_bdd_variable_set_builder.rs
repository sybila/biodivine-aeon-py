use crate::bindings::lib_bdd::{PyBddVariable, PyBddVariableSet, PyBddVariableSetBuilder};
use biodivine_lib_bdd::BddVariableSetBuilder;
use pyo3::prelude::*;

impl Default for PyBddVariableSetBuilder {
    fn default() -> Self {
        PyBddVariableSetBuilder::new(None)
    }
}

#[pymethods]
impl PyBddVariableSetBuilder {
    #[new]
    #[pyo3(signature = (variables = None))]
    pub fn new(variables: Option<Vec<String>>) -> Self {
        let variables = variables.unwrap_or_default();
        let mut builder = BddVariableSetBuilder::new();
        for var in &variables {
            builder.make_variable(var.as_str());
        }
        PyBddVariableSetBuilder(builder, variables)
    }

    pub fn __str__(&self) -> String {
        format!("BddVariableSetBuilder({:?})", self.1)
    }

    pub fn __repr__(&self) -> String {
        self.__str__()
    }

    pub fn make(&mut self, name: &str) -> PyBddVariable {
        let var = self.0.make_variable(name).into();
        self.1.push(name.to_string());
        var
    }

    pub fn make_all(&mut self, names: Vec<&str>) -> Vec<PyBddVariable> {
        let mut result: Vec<PyBddVariable> = Vec::new();
        for name in names {
            result.push(self.make(name));
        }
        result
    }

    pub fn build(&self) -> PyBddVariableSet {
        let mut builder = BddVariableSetBuilder::new();
        for name in &self.1 {
            builder.make_variable(name);
        }
        builder.build().into()
    }
}
