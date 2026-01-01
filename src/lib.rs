use pyo3::exceptions::{PyIndexError, PyInterruptedError, PyRuntimeError, PyTypeError};
use pyo3::prelude::*;
use pyo3::{PyErrArguments, PyResult, Python};

/// A module with all the glue and wrapper code that makes the Python bindings work.
///
/// Should be split into submodules based on individual crates such that each submodule
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
pub mod bindings;

/// In this module, we have copied some of the internal AEON algorithms that we cannot include
/// directly since they are not part of a public crate. Try to keep this module as small as
/// possible -- ideally, the stuff in here should be eventually published to crates.io and turned
/// into a dependency.
///
mod internal;
mod pyo3_utils;

fn set_log_level(_py: Python, module: &Bound<'_, PyModule>) -> PyResult<()> {
    /* Disable default logging. Turns out this is not so useful after all.

    // This should be an error when running as a script or a normal shell, but returns a name in notebooks.
    // https://stackoverflow.com/questions/15411967/how-can-i-check-if-code-is-executed-in-the-ipython-notebook
    let is_notebook = py.eval_bound("get_ipython().__class__.__name__", None, None);
    if let Ok(is_notebook) = is_notebook {
        if let Ok(is_notebook) = is_notebook.extract::<String>() {
            println!("Detected IPython (`{is_notebook}`). Log level set to `LOG_ESSENTIAL`.");
            return module.setattr("LOG_LEVEL", 1);
        }
    }
    // This should detect if we are running in a shell, as opposed to a script.
    // Context:
    // https://stackoverflow.com/questions/2356399/tell-if-python-is-in-interactive-mode
    // https://stackoverflow.com/questions/6108330/checking-for-interactive-shell-in-a-python-script
    let sys = PyModule::import_bound(py, "sys")?;
    let locals = PyDict::new_bound(py);
    locals.set_item("sys", sys)?;
    let has_ps = py.eval_bound("hasattr(sys, 'ps1')", None, Some(&locals))?;
    if let Ok(true) = has_ps.extract::<bool>() {
        println!("Detected interactive mode. Log level set to `LOG_ESSENTIAL`.");
        return module.setattr("LOG_LEVEL", 1);
    }
    */
    module.setattr("LOG_LEVEL", 0)
}

fn global_log_level(py: Python) -> PyResult<usize> {
    let module = PyModule::import(py, "biodivine_aeon")?;
    module.getattr("LOG_LEVEL")?.extract()
}

const LOG_NOTHING: usize = 0;

fn should_log(log_level: usize) -> bool {
    log_level > LOG_NOTHING
}

/// AEON.py is a Python library for working with Boolean networks. It supports a wide
/// range of features, including symbolic representation of network dynamics. More general
/// information is available on [GitHub](https://github.com/sybila/biodivine-aeon-py/).
///
/// To explore the library documentation, you can start with one of the core data classes:
/// `RegulatoryGraph`, `BooleanNetwork` and `AsynchronousGraph`. Alternatively, to learn more about the available
/// algorithms, you may want to look at `Attractors`, `TrapSpaces`, `FixedPoints`, `Classification`,
/// or `Control`.
#[pymodule]
fn biodivine_aeon(py: Python, module: &Bound<'_, PyModule>) -> PyResult<()> {
    set_log_level(py, module)?;
    bindings::lib_bdd::register(module)?;
    bindings::lib_param_bn::register(module)?;
    bindings::lib_hctl_model_checker::register(module)?;
    bindings::bn_classifier::register(module)?;
    bindings::pbn_control::register(module)?;
    #[cfg(feature = "algorithms-pyo3-bindings")]
    bindings::algorithms::register(module)?;
    Ok(())
}

/// This trait works similar to the `From` conversion, but it explicitly takes and returns
/// a reference, which makes it a bit easier for type inference to figure out what is going
/// on. As such, this can often be used to simplify some conversions.
///
/// Note that you don't need this for "value structs" (i.e., implementing copy), where everything
/// is copied anyway and references aren't relevant.
trait AsNative<T> {
    fn as_native(&self) -> &T;
    fn as_native_mut(&mut self) -> &mut T;
}

/// Helper function to quickly throw a type error.
fn throw_type_error<T, A>(message: A) -> PyResult<T>
where
    A: Send + Sync + PyErrArguments + 'static,
{
    Err(PyTypeError::new_err(message))
}

/// Helper function to quickly throw a runtime error.
fn throw_runtime_error<T, A>(message: A) -> PyResult<T>
where
    A: Send + Sync + PyErrArguments + 'static,
{
    Err(runtime_error::<A>(message))
}

/// Helper function to quickly create a runtime error.
fn runtime_error<A>(message: A) -> PyErr
where
    A: Send + Sync + PyErrArguments + 'static,
{
    PyRuntimeError::new_err(message)
}

fn throw_index_error<T, A>(message: A) -> PyResult<T>
where
    A: Send + Sync + PyErrArguments + 'static,
{
    Err(PyIndexError::new_err(message))
}

fn index_error<A>(message: A) -> PyErr
where
    A: Send + Sync + PyErrArguments + 'static,
{
    PyIndexError::new_err(message)
}

fn throw_interrupted_error<T, A>(message: A) -> PyResult<T>
where
    A: Send + Sync + PyErrArguments + 'static,
{
    Err(PyInterruptedError::new_err(message))
}
