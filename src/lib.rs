use pyo3::exceptions::{PyRuntimeError, PyTypeError};
use pyo3::prelude::*;
use pyo3::{PyResult, Python};

/// A module with all the glue and wrapper code that makes the Python bindings work.
///
/// Should be split into sub-modules based on individual crates such that each sub-module
/// provides a `register` function that allows us to export it into Python here
/// within the root module.
///
/// Also, as a convention, wrapper structs for native types should use prefix "Py*", the same
/// way other structs are named within PyO3. In Python, the name can be different
/// (especially without "Py*") --- use the `name` attribute of `#[pyclass]` for that.
///
/// For each wrapper struct, please provide a `From`/`Into` conversion to the original type and
/// a somewhat reasonable `__str__` and `__repr__` implementations (`__str__` is normal
/// `to_string`, `__repr__` is `to_string` when printing in interpreter).
///
/// If reasonable, please also implement `__hash__` and `__richcmp__` to ensure correct type
/// comparison (alternatively, `__richcmp__` can be substituted for `__lt__`, `__eq__`, etc.).
///
mod bindings;

/// In this module, we have copied some of the internal AEON algorithms that we cannot include
/// directly since they are not part of a public crate. Try to keep this module as small as
/// possible -- ideally, the stuff in here should be eventually published to crates.io and turned
/// into a dependency.
///
mod internal;

/// A Python module implemented in Rust.
#[pymodule]
fn biodivine_aeon(_py: Python, module: &PyModule) -> PyResult<()> {
    bindings::lib_bdd::register(module)?;
    bindings::lib_param_bn::register(module)?;
    bindings::aeon::register(module)?;
    bindings::pbn_control::register(module)?;
    Ok(())
}

/// This trait works similar to the `From` conversion, but it explicitly takes and returns
/// a reference, which makes it a bit easier for type inference to figure out what is going
/// on. As such, this can be often used to simplify some conversions.
///
/// Note that you don't need this for "value structs" (i.e. implementing copy), where everything
/// is copied anyway and references do not do much.
trait AsNative<T> {
    fn as_native(&self) -> &T;
    fn as_native_mut(&mut self) -> &mut T;
}

/// Helper function to quickly throw a type error.
fn throw_type_error<T, A: 'static>(message: A) -> PyResult<T>
where
    A: Send + Sync + IntoPy<Py<PyAny>>,
{
    Err(PyTypeError::new_err(message))
}

/// Helper function to quickly throw a runtime error.
fn throw_runtime_error<T, A: 'static>(message: A) -> PyResult<T>
where
    A: Send + Sync + IntoPy<Py<PyAny>>,
{
    Err(runtime_error::<T, A>(message))
}

/// Helper function to quickly create a runtime error.
fn runtime_error<T, A: 'static>(message: A) -> PyErr
where
    A: Send + Sync + IntoPy<Py<PyAny>>,
{
    PyRuntimeError::new_err(message)
}
