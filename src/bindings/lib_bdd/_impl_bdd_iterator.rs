use crate::bindings::lib_bdd::{
    PyBddClauseIterator, PyBddPartialValuation, PyBddValuation, PyBddValuationIterator,
};
use pyo3::{pymethods, PyRef, PyRefMut};

#[pymethods]
impl PyBddValuationIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<PyBddValuation> {
        slf.0.next().map(|it| it.into())
    }
}

#[pymethods]
impl PyBddClauseIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<PyBddPartialValuation> {
        slf.0.next().map(|it| it.into())
    }
}
