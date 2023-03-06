use crate::bindings::lib_bdd::{
    PyBdd, PyBddClauseIterator, PyBddPartialValuation, PyBddValuation, PyBddValuationIterator,
};
use crate::AsNative;
use biodivine_lib_bdd::Bdd;
use pyo3::{pymethods, Py, PyRef, PyRefMut, PyResult, Python};

#[pymethods]
impl PyBddValuationIterator {
    #[new]
    pub fn new(py: Python, bdd: Py<PyBdd>) -> PyBddValuationIterator {
        // This hack allows us to "launder" lifetimes between Rust and Python.
        // It is only safe because we copy the `Bdd` and attach it to the "laundered" reference,
        // so there is no (realistic) way the reference can outlive the copy of the `Bdd`.
        // Fortunately, the iterator items are clones and do not reference the `Bdd` directly,
        // so the "laundered" references do not spread beyond the internal code of the iterator.
        let iterator = {
            let bdd_ref = bdd.borrow(py);
            let bdd_ref: &'static Bdd =
                unsafe { (bdd_ref.as_native() as *const Bdd).as_ref().unwrap() };
            bdd_ref.sat_valuations()
        };
        PyBddValuationIterator(bdd, iterator)
    }

    pub fn __str__(&self, py: Python) -> PyResult<String> {
        let bdd_ref = self.0.extract::<PyRef<PyBdd>>(py)?;
        Ok(format!("BddValuationIterator({})", bdd_ref.__str__()))
    }

    pub fn __repr__(&self, py: Python) -> PyResult<String> {
        Ok(format!("<{}>", self.__str__(py)?))
    }

    pub fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    pub fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<PyBddValuation> {
        slf.1.next().map(|it| it.into())
    }
}

#[pymethods]
impl PyBddClauseIterator {
    #[new]
    pub fn new(py: Python, bdd: Py<PyBdd>) -> PyBddClauseIterator {
        // See above for discussion why this is safe.
        let iterator = {
            let bdd_ref = bdd.borrow(py);
            let bdd_ref: &'static Bdd =
                unsafe { (bdd_ref.as_native() as *const Bdd).as_ref().unwrap() };
            bdd_ref.sat_clauses()
        };
        PyBddClauseIterator(bdd, iterator)
    }

    pub fn __str__(&self, py: Python) -> PyResult<String> {
        let bdd_ref = self.0.extract::<PyRef<PyBdd>>(py)?;
        Ok(format!("BddClauseIterator({})", bdd_ref.__str__()))
    }

    pub fn __repr__(&self, py: Python) -> PyResult<String> {
        Ok(format!("<{}>", self.__str__(py)?))
    }

    pub fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    pub fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<PyBddPartialValuation> {
        slf.1.next().map(|it| it.into())
    }
}
