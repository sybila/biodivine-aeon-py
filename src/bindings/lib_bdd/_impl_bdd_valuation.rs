// PyO3 does not allow ownership transfer, so we are forced to use references.
#![allow(clippy::wrong_self_convention)]

use crate::bindings::lib_bdd::{PyBddPartialValuation, PyBddValuation, PyBddVariable};
use crate::{throw_runtime_error, throw_type_error, AsNative};
use biodivine_lib_bdd::{BddPartialValuation, BddValuation, BddVariable};
use pyo3::basic::CompareOp;
use pyo3::types::{PyDict, PyList, PyTuple};
use pyo3::{pymethods, Py, PyAny, PyResult, Python};
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

impl PyBddValuation {
    /// Try to read a `PyBddValuation` from a dynamic Python object. This can be either:
    ///
    ///  - `PyBddValuation` itself;
    ///  - A list of Boolean values.
    pub(crate) fn from_python_type(any: &PyAny) -> PyResult<Py<PyBddValuation>> {
        let py = any.py();
        if let Ok(val) = any.extract::<Py<PyBddValuation>>() {
            Ok(val)
        } else if let Ok(list) = any.extract::<Vec<bool>>() {
            Py::new(py, PyBddValuation(BddValuation::new(list)))
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
    pub(crate) fn from_python_type(any: &PyAny) -> PyResult<Py<PyBddPartialValuation>> {
        let py = any.py();
        if let Ok(val) = any.extract::<Py<PyBddPartialValuation>>() {
            Ok(val)
        } else {
            let native = if let Ok(dict) = any.downcast::<PyDict>() {
                let mut vars = BddPartialValuation::empty();
                for (k, v) in dict {
                    let k: BddVariable = k.extract::<PyBddVariable>()?.into();
                    let v = v.extract::<bool>()?;
                    vars.set_value(k, v);
                }
                vars
            } else if let Ok(list) = any.downcast::<PyList>() {
                let mut vars = BddPartialValuation::empty();
                for tuple in list {
                    if let Ok(tuple) = tuple.downcast::<PyTuple>() {
                        let (k, v) = Self::extract_valuation_pair(tuple)?;
                        vars.set_value(k, v)
                    } else {
                        return throw_type_error("Expected a list of tuples.");
                    }
                }
                vars
            } else if let Ok(tuple) = any.downcast::<PyTuple>() {
                let mut vars = BddPartialValuation::empty();
                let (k, v) = Self::extract_valuation_pair(tuple)?;
                vars.set_value(k, v);
                vars
            } else {
                return throw_type_error("Expected a partial Bdd valuation.");
            };
            Py::new(py, PyBddPartialValuation(native))
        }
    }

    fn extract_valuation_pair(tuple: &PyTuple) -> PyResult<(BddVariable, bool)> {
        if tuple.len() != 2 {
            throw_type_error("Expected a tuple of length two.")
        } else {
            let var: BddVariable = tuple.get_item(0)?.extract::<PyBddVariable>()?.into();
            let value = tuple.get_item(1)?.extract::<bool>()?;
            Ok((var, value))
        }
    }
}

#[pymethods]
impl PyBddValuation {
    #[new]
    pub fn new(data: &PyAny) -> PyResult<PyBddValuation> {
        if let Ok(num_vars) = data.extract::<u16>() {
            Ok(BddValuation::all_false(num_vars).into())
        } else if let Ok(values) = data.extract::<Vec<bool>>() {
            Ok(BddValuation::new(values).into())
        } else {
            throw_type_error("Expected integer or list of Boolean values.")
        }
    }

    pub fn __getitem__(&self, index: PyBddVariable) -> bool {
        self.as_native().value(index.into())
    }

    pub fn __setitem__(&mut self, index: PyBddVariable, value: bool) {
        self.as_native_mut().set_value(index.into(), value);
    }

    pub fn __len__(&self) -> usize {
        self.as_native().num_vars() as usize
    }

    pub fn __eq__(&self, other: &PyBddValuation) -> bool {
        self.as_native().eq(other.as_native())
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Lt => throw_runtime_error("Unimplemented"),
            CompareOp::Le => throw_runtime_error("Unimplemented"),
            CompareOp::Eq => Ok(self.__eq__(other)),
            CompareOp::Ne => Ok(!self.__eq__(other)),
            CompareOp::Gt => throw_runtime_error("Unimplemented"),
            CompareOp::Ge => throw_runtime_error("Unimplemented"),
        }
    }

    pub fn __hash__(&self) -> isize {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish() as isize
    }

    pub fn __str__(&self) -> String {
        let values = self
            .as_native()
            .clone()
            .vector()
            .into_iter()
            .map(|val| if val { "True" } else { "False" })
            .collect::<Vec<_>>();
        let values = values.join(",");
        format!("[{values}]")
    }

    pub fn __repr__(&self) -> String {
        format!("BddValuation({})", self.__str__())
    }

    pub fn to_list(&self) -> Vec<bool> {
        self.as_native().clone().vector()
    }

    pub fn extends(&self, py: Python, partial_valuation: &PyAny) -> PyResult<bool> {
        let partial_valuation = PyBddPartialValuation::from_python_type(partial_valuation)?;
        let partial_valuation = partial_valuation.borrow(py);
        Ok(self.as_native().extends(partial_valuation.as_native()))
    }
}

#[pymethods]
impl PyBddPartialValuation {
    #[new]
    pub fn new(py: Python, data: &PyAny) -> PyResult<PyBddPartialValuation> {
        if data.is_none() {
            Ok(BddPartialValuation::empty().into())
        } else {
            let data = PyBddPartialValuation::from_python_type(data)?;
            data.extract::<PyBddPartialValuation>(py)
        }
    }

    pub fn __str__(&self) -> String {
        // TODO:
        //  If we ever get this into a state where it should output valid Python,
        //  this needs to change true/false to True/False.
        format!("BddPartialValuation({:?})", self.as_native().to_values())
    }

    pub fn __repr__(&self) -> String {
        format!("<{}>", self.__str__())
    }

    pub fn __len__(&self) -> usize {
        self.as_native().cardinality() as usize
    }

    pub fn __getitem__(&self, index: PyBddVariable) -> Option<bool> {
        self.as_native().get_value(index.into())
    }

    pub fn __setitem__(&mut self, index: PyBddVariable, value: bool) {
        self.as_native_mut().set_value(index.into(), value);
    }

    pub fn __delitem__(&mut self, index: PyBddVariable) {
        self.as_native_mut().unset_value(index.into());
    }

    pub fn __contains__(&self, index: PyBddVariable) -> bool {
        self.as_native().has_value(index.into())
    }

    pub fn __iter__(&self) -> Vec<PyBddVariable> {
        self.as_native()
            .to_values()
            .into_iter()
            .map(|(k, _)| k.into())
            .collect::<Vec<_>>()
    }

    pub fn __hash__(&self) -> isize {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish() as isize
    }

    pub fn __eq__(&self, other: &PyBddPartialValuation) -> bool {
        self.as_native().eq(other.as_native())
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Lt => throw_runtime_error("Unimplemented"),
            CompareOp::Le => throw_runtime_error("Unimplemented"),
            CompareOp::Eq => Ok(self.__eq__(other)),
            CompareOp::Ne => Ok(!self.__eq__(other)),
            CompareOp::Gt => throw_runtime_error("Unimplemented"),
            CompareOp::Ge => throw_runtime_error("Unimplemented"),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.as_native().is_empty()
    }

    pub fn extends(&self, py: Python, partial_valuation: &PyAny) -> PyResult<bool> {
        let partial_valuation = PyBddPartialValuation::from_python_type(partial_valuation)?;
        let partial_valuation = partial_valuation.borrow(py);
        Ok(self.as_native().extends(partial_valuation.as_native()))
    }

    pub fn support_set(&self) -> HashSet<PyBddVariable> {
        self.as_native()
            .to_values()
            .into_iter()
            .map(|(k, _)| k.into())
            .collect()
    }

    pub fn to_dict(&self) -> HashMap<PyBddVariable, bool> {
        self.as_native()
            .to_values()
            .into_iter()
            .map(|(k, v)| (k.into(), v))
            .collect()
    }

    pub fn to_list(&self) -> Vec<(PyBddVariable, bool)> {
        self.as_native()
            .to_values()
            .into_iter()
            .map(|(k, v)| (k.into(), v))
            .collect()
    }
}
