// PyO3 does not allow ownership transfer, so we are forced to use references.
#![allow(clippy::wrong_self_convention)]

use crate::bindings::lib_bdd::{PyBddPartialValuation, PyBddValuation, PyBddVariable, PyBddVariableSet};
use crate::{throw_runtime_error, throw_type_error, AsNative};
use biodivine_lib_bdd::{BddPartialValuation, BddValuation, BddVariable};
use pyo3::basic::CompareOp;
use pyo3::types::{PyDict, PyList, PyTuple};
use pyo3::{pymethods, PyAny, PyResult};
use std::collections::HashMap;

impl Default for PyBddPartialValuation {
    fn default() -> Self {
        PyBddPartialValuation::new()
    }
}

impl PyBddValuation {
    /// Try to read a BDD valuation from a dynamic Python type. This can be either:
    ///
    ///  - `PyBddValuation` itself;
    ///  - A list of Boolean values.
    pub(crate) fn from_python(any: &PyAny) -> PyResult<PyBddValuation> {
        if let Ok(val) = any.extract::<PyBddValuation>() {
            Ok(val)
        } else if let Ok(list) = any.extract::<Vec<bool>>() {
            Ok(PyBddValuation::from_list(list))
        } else {
            throw_type_error("Expected a Bdd valuation.")
        }
    }
}

impl PyBddPartialValuation {
    /// Try to read a partial BDD valuation from a dynamic Python type. This can be either:
    ///
    ///  - `PyBddPartialValuation` itself;
    ///  - A dictionary that maps BDD variables to bools;
    ///  - A list of variable-bool pairs;
    ///  - A single variable-bool pair.
    pub(crate) fn from_python(any: &PyAny, names: Option<&PyBddVariableSet>) -> PyResult<PyBddPartialValuation> {
        if let Ok(val) = any.extract::<PyBddPartialValuation>() {
            Ok(val)
        } else if let Ok(dict) = any.downcast::<PyDict>() {
            let mut vars = BddPartialValuation::empty();
            for (k, v) in dict {
                let k: BddVariable = if let Some(names) = names {
                    names.find_variable(k)?.unwrap().into()
                } else {
                    k.extract::<PyBddVariable>()?.into()
                };
                let v = v.extract::<bool>()?;
                vars.set_value(k, v);
            }
            Ok(PyBddPartialValuation(vars))
        } else if let Ok(list) = any.downcast::<PyList>() {
            let mut vars = BddPartialValuation::empty();
            for tuple in list {
                if let Ok(tuple) = tuple.downcast::<PyTuple>() {
                    let (k, v) = Self::extract_valuation_pair(tuple, names)?;
                    vars.set_value(k, v)
                } else {
                    return throw_type_error("Expected a list of tuples.");
                }
            }
            Ok(PyBddPartialValuation(vars))
        } else if let Ok(tuple) = any.downcast::<PyTuple>() {
            let mut vars = BddPartialValuation::empty();
            let (k, v) = Self::extract_valuation_pair(tuple, names)?;
            vars.set_value(k, v);
            Ok(PyBddPartialValuation(vars))
        } else {
            throw_type_error("Expected a partial Bdd valuation.")
        }
    }

    fn extract_valuation_pair(tuple: &PyTuple, names: Option<&PyBddVariableSet>) -> PyResult<(BddVariable, bool)> {
        if tuple.len() != 2 {
            throw_type_error("Expected a tuple of length two.")
        } else {
            let k = tuple.get_item(0)?;
            let var: BddVariable = if let Some(names) = names {
                names.find_variable(k)?.unwrap().into()
            } else {
                k.extract::<PyBddVariable>()?.into()
            };
            let value = tuple.get_item(1)?.extract::<bool>()?;
            Ok((var, value))
        }
    }
}

#[pymethods]
impl PyBddValuation {
    fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Lt => throw_runtime_error("Unsupported operation."),
            CompareOp::Le => throw_runtime_error("Unsupported operation."),
            CompareOp::Eq => Ok(self.as_native() == other.as_native()),
            CompareOp::Ne => Ok(self.as_native() != other.as_native()),
            CompareOp::Gt => throw_runtime_error("Unsupported operation."),
            CompareOp::Ge => throw_runtime_error("Unsupported operation."),
        }
    }

    #[new]
    pub fn new(variables: u16) -> PyBddValuation {
        BddValuation::all_false(variables).into()
    }

    pub fn __str__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self.as_native()))
    }

    pub fn __repr__(&self) -> PyResult<String> {
        self.__str__()
    }

    #[staticmethod]
    pub fn from_list(values: Vec<bool>) -> PyBddValuation {
        BddValuation::new(values).into()
    }

    pub fn into_list(&self) -> Vec<bool> {
        self.as_native().clone().vector()
    }

    pub fn __len__(&self) -> usize {
        self.as_native().num_vars() as usize
    }

    pub fn __getitem__(&self, index: PyBddVariable) -> bool {
        self.as_native().value(index.into())
    }

    pub fn __setitem__(&mut self, index: PyBddVariable, value: bool) {
        self.as_native_mut().set_value(index.into(), value);
    }

    pub fn __iter__(&self) -> Vec<bool> {
        self.into_list()
    }

    pub fn extends(&self, partial_valuation: &PyAny) -> PyResult<bool> {
        let partial_valuation = PyBddPartialValuation::from_python(partial_valuation, None)?;
        Ok(self.as_native().extends(partial_valuation.as_native()))
    }
}

#[pymethods]
impl PyBddPartialValuation {
    fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Lt => throw_runtime_error("Unsupported operation."),
            CompareOp::Le => throw_runtime_error("Unsupported operation."),
            CompareOp::Eq => Ok(self.as_native() == other.as_native()),
            CompareOp::Ne => Ok(self.as_native() != other.as_native()),
            CompareOp::Gt => throw_runtime_error("Unsupported operation."),
            CompareOp::Ge => throw_runtime_error("Unsupported operation."),
        }
    }

    #[new]
    pub fn new() -> PyBddPartialValuation {
        BddPartialValuation::empty().into()
    }

    pub fn __str__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self.as_native()))
    }

    pub fn __repr__(&self) -> PyResult<String> {
        self.__str__()
    }

    pub fn __len__(&self) -> usize {
        self.as_native().cardinality() as usize
    }

    pub fn __getitem__(&self, index: PyBddVariable) -> Option<bool> {
        self.as_native().get_value(index.into())
    }

    pub fn __setitem__(&mut self, index: PyBddVariable, value: Option<bool>) {
        if let Some(value) = value {
            self.as_native_mut().set_value(index.into(), value);
        } else {
            self.as_native_mut().unset_value(index.into());
        }
    }

    pub fn __delitem__(&mut self, index: PyBddVariable) {
        self.as_native_mut().unset_value(index.into());
    }

    pub fn __contains__(&self, index: PyBddVariable) -> bool {
        self.as_native().has_value(index.into())
    }

    pub fn __iter__(&self) -> Vec<(PyBddVariable, bool)> {
        self.into_list()
    }

    pub fn extends(&self, partial_valuation: &PyAny) -> PyResult<bool> {
        let partial_valuation = PyBddPartialValuation::from_python(partial_valuation, None)?;
        Ok(self.as_native().extends(partial_valuation.as_native()))
    }

    pub fn into_list(&self) -> Vec<(PyBddVariable, bool)> {
        self.as_native()
            .to_values()
            .into_iter()
            .map(|(k, v)| (k.into(), v))
            .collect()
    }

    pub fn into_dict(&self) -> HashMap<PyBddVariable, bool> {
        self.as_native()
            .to_values()
            .into_iter()
            .map(|(k, v)| (k.into(), v))
            .collect()
    }

    #[staticmethod]
    pub fn from_data(data: &PyAny) -> PyResult<PyBddPartialValuation> {
        PyBddPartialValuation::from_python(data, None)
    }
}
