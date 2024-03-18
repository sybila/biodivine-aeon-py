use pyo3::{Py, PyAny, PyResult, Python};

/// A utility struct that represents an explicit function table of a two-argument `Option<bool>`
/// function. We use this to implement a "conversion" from a Python function
/// to a normal rust closure.
pub struct OpFunction2(Vec<Option<bool>>);

/// A utility struct that represents an explicit function table of a three-argument `Option<bool>`
/// function. We use this to implement a "conversion" from a Python function
/// to a normal rust closure.
pub struct OpFunction3(Vec<Option<bool>>);

impl OpFunction2 {
    pub fn new(py: Python, py_function: Py<PyAny>) -> PyResult<OpFunction2> {
        let none: Option<bool> = None;
        let tt = Some(true);
        let ff = Some(false);
        let none_none: Option<bool> = py_function.call1(py, (none, none))?.extract(py)?;
        let none_false: Option<bool> = py_function.call1(py, (none, ff))?.extract(py)?;
        let false_none: Option<bool> = py_function.call1(py, (ff, none))?.extract(py)?;
        let false_false: Option<bool> = py_function.call1(py, (ff, ff))?.extract(py)?;
        let none_true: Option<bool> = py_function.call1(py, (none, tt))?.extract(py)?;
        let true_none: Option<bool> = py_function.call1(py, (tt, none))?.extract(py)?;
        let true_true: Option<bool> = py_function.call1(py, (tt, tt))?.extract(py)?;
        let true_false: Option<bool> = py_function.call1(py, (tt, ff))?.extract(py)?;
        let false_true: Option<bool> = py_function.call1(py, (ff, tt))?.extract(py)?;
        Ok(OpFunction2(vec![
            none_none,
            none_false,
            false_none,
            false_false,
            none_true,
            true_none,
            true_true,
            true_false,
            false_true,
        ]))
    }

    pub fn invoke(&self, left: Option<bool>, right: Option<bool>) -> Option<bool> {
        let i = match (left, right) {
            (None, None) => 0,
            (None, Some(false)) => 1,
            (Some(false), None) => 2,
            (Some(false), Some(false)) => 3,
            (None, Some(true)) => 4,
            (Some(true), None) => 5,
            (Some(true), Some(true)) => 6,
            (Some(true), Some(false)) => 7,
            (Some(false), Some(true)) => 8,
        };
        unsafe { *self.0.get_unchecked(i) }
    }
}

impl OpFunction3 {
    pub fn new(py: Python, py_function: Py<PyAny>) -> PyResult<OpFunction3> {
        let none: Option<bool> = None;
        let tt = Some(true);
        let ff = Some(false);
        let none_none_none: Option<bool> =
            py_function.call1(py, (none, none, none))?.extract(py)?;
        let none_none_false: Option<bool> = py_function.call1(py, (none, none, ff))?.extract(py)?;
        let none_false_none: Option<bool> = py_function.call1(py, (none, ff, none))?.extract(py)?;
        let false_none_none: Option<bool> = py_function.call1(py, (ff, none, none))?.extract(py)?;
        let none_false_false: Option<bool> = py_function.call1(py, (none, ff, ff))?.extract(py)?;
        let false_none_false: Option<bool> = py_function.call1(py, (ff, none, ff))?.extract(py)?;
        let false_false_none: Option<bool> = py_function.call1(py, (ff, ff, none))?.extract(py)?;
        let false_false_false: Option<bool> = py_function.call1(py, (ff, ff, ff))?.extract(py)?;
        let none_none_true: Option<bool> = py_function.call1(py, (none, none, tt))?.extract(py)?;
        let none_true_none: Option<bool> = py_function.call1(py, (none, tt, none))?.extract(py)?;
        let true_none_none: Option<bool> = py_function.call1(py, (tt, none, none))?.extract(py)?;
        let none_true_true: Option<bool> = py_function.call1(py, (none, tt, tt))?.extract(py)?;
        let true_none_true: Option<bool> = py_function.call1(py, (tt, none, tt))?.extract(py)?;
        let true_true_none: Option<bool> = py_function.call1(py, (tt, tt, none))?.extract(py)?;
        let true_true_true: Option<bool> = py_function.call1(py, (tt, tt, tt))?.extract(py)?;
        let none_false_true: Option<bool> = py_function.call1(py, (none, ff, tt))?.extract(py)?;
        let none_true_false: Option<bool> = py_function.call1(py, (none, tt, ff))?.extract(py)?;
        let false_none_true: Option<bool> = py_function.call1(py, (ff, none, tt))?.extract(py)?;
        let true_none_false: Option<bool> = py_function.call1(py, (tt, none, ff))?.extract(py)?;
        let false_true_none: Option<bool> = py_function.call1(py, (ff, tt, none))?.extract(py)?;
        let true_false_none: Option<bool> = py_function.call1(py, (tt, ff, none))?.extract(py)?;
        let false_false_true: Option<bool> = py_function.call1(py, (ff, ff, tt))?.extract(py)?;
        let false_true_false: Option<bool> = py_function.call1(py, (ff, tt, ff))?.extract(py)?;
        let true_false_false: Option<bool> = py_function.call1(py, (tt, ff, ff))?.extract(py)?;
        let false_true_true: Option<bool> = py_function.call1(py, (ff, tt, tt))?.extract(py)?;
        let true_false_true: Option<bool> = py_function.call1(py, (tt, ff, tt))?.extract(py)?;
        let true_true_false: Option<bool> = py_function.call1(py, (tt, tt, ff))?.extract(py)?;
        Ok(OpFunction3(vec![
            none_none_none,
            none_none_false,
            none_false_none,
            false_none_none,
            none_false_false,
            false_none_false,
            false_false_none,
            false_false_false,
            none_none_true,
            none_true_none,
            true_none_none,
            none_true_true,
            true_none_true,
            true_true_none,
            true_true_true,
            none_false_true,
            none_true_false,
            false_none_true,
            true_none_false,
            false_true_none,
            true_false_none,
            false_false_true,
            false_true_false,
            true_false_false,
            false_true_true,
            true_false_true,
            true_true_false,
        ]))
    }

    pub fn invoke(&self, a: Option<bool>, b: Option<bool>, c: Option<bool>) -> Option<bool> {
        let i = match (a, b, c) {
            (None, None, None) => 0,
            (None, None, Some(false)) => 1,
            (None, Some(false), None) => 2,
            (Some(false), None, None) => 3,
            (None, Some(false), Some(false)) => 4,
            (Some(false), None, Some(false)) => 5,
            (Some(false), Some(false), None) => 6,
            (Some(false), Some(false), Some(false)) => 7,
            (None, None, Some(true)) => 8,
            (None, Some(true), None) => 9,
            (Some(true), None, None) => 10,
            (None, Some(true), Some(true)) => 11,
            (Some(true), None, Some(true)) => 12,
            (Some(true), Some(true), None) => 13,
            (Some(true), Some(true), Some(true)) => 14,
            (None, Some(false), Some(true)) => 15,
            (None, Some(true), Some(false)) => 16,
            (Some(false), None, Some(true)) => 17,
            (Some(true), None, Some(false)) => 18,
            (Some(false), Some(true), None) => 19,
            (Some(true), Some(false), None) => 20,
            (Some(false), Some(false), Some(true)) => 21,
            (Some(false), Some(true), Some(false)) => 22,
            (Some(true), Some(false), Some(false)) => 23,
            (Some(false), Some(true), Some(true)) => 24,
            (Some(true), Some(false), Some(true)) => 25,
            (Some(true), Some(true), Some(false)) => 26,
        };
        unsafe { *self.0.get_unchecked(i) }
    }
}
