use pyo3::{create_exception, exceptions::PyException};

// TODO: docs - add fourth argument, documentation
create_exception!(configurable, CreationFailedError, PyException);
