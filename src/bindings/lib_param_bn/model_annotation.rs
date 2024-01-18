use crate::{throw_runtime_error, AsNative};
use macros::Wrapper;
use pyo3::basic::CompareOp;
use pyo3::prelude::*;

/*
   I am sorry for this mess, but this seems to be the best solution at the moment
*/

/// Annotations are "meta" objects that can be declared as part of `.aeon` models to add additional
/// properties that are not directly recognized by the main AEON toolbox.
///
/// An annotation object behaves like a tree, where each node can have an optional string value,
/// and an unordered collection of children which behave mostly like a dictionary:
///
/// ```python
/// ann = ModelAnnotation()
/// # Empty ModelAnnotation is created automatically when key is first accessed.
/// desc = ann['description']
/// assert desc.value is None
/// desc.value = "Multiline...\n"
/// desc.value += "...test description"
/// desc['x'].value = "Variable X"
/// desc['y'].value = "Variable Y"
///
/// assert len(desc) == 2
/// assert ann['description']['x'].value == "Variable X"
/// ```
///
/// This generates the following set of annotation comments:
/// ```text
/// #!description:Multiline...
/// #!description:...test description
/// #!description:x:Variable X
/// #!description:y:Variable Y
/// ```
///
/// #### Annotation syntax
///
/// Annotations are comments which start with `#!`. After the `#!` "preamble", each annotation
/// can contains a "path prefix" with path segments separated using `:` (path segments can be
/// surrounded by white space that is automatically trimmed). Based on these path
/// segments, the parser will create an annotation tree. If there are multiple annotations with
/// the same path, their values are concatenated using newlines.
///
/// For example, annotations can be used to describe the model layout:
///
/// ```text
/// #! layout : var_1 : 10,20
/// #! layout : var_2 : 14,-3
/// ```
///
/// Another usage for annotations are additional properties expected from the model, for
/// example written in CTL:
/// ```text
/// #! property : AG (problem => AF apoptosis)
/// ```
///
/// Obviously, you can also use annotations to specify model metadata:
/// ```text
/// #! name: My Awesome Model
/// #! description: This model describes ...
/// #! description:var_1: This variable describes ...
/// ```
///
/// You can use "empty" path (e.g. `#! is_multivalued`), and you can use an empty annotation
/// value with a non-empty path (e.g. `#! is_multivalued:var_1:`). Though this is not particularly
/// encouraged: it is better to just have `var_1` as the annotation value if you can do that.
/// An exception to this may be a case where `is_multivalued:var_1:` has an "optional" value and
/// you want to express that while the "key" is provided, the "value" is missing. Similarly, for
/// the sake of completeness, it is technically allowed to use empty path names (e.g. `a::b:value`
/// translates to `["a", "", "b"] = "value"`), but it is discouraged.
///
/// Note that the path segments should only contain alphanumeric characters and underscores,
/// but can be escaped using backticks (`` ` ``; other backticks in path segments are not allowed).
/// Similarly, annotation values cannot contain colons (path segment separators) or backticks,
/// unless escaped with `` #`ACTUAL_STRING`# ``. You can also use escaping if you wish to
/// retain whitespace surrounding the annotation value. As mentioned, multi-line values can be
/// split into multiple annotation comments.
#[pyclass(module = "biodivine_aeon", frozen)]
#[derive(Clone)]
pub struct ModelAnnotation {
    root: Py<ModelAnnotationRoot>,
    path: Vec<String>,
}

#[pyclass(module = "biodivine_aeon", name = "_ModelAnnotation")]
#[derive(Clone, Wrapper)]
pub struct ModelAnnotationRoot(biodivine_lib_param_bn::ModelAnnotation);

fn mk_path(path: &[String]) -> Vec<&str> {
    // TODO: delete once latest lib-param-bn is released.
    path.iter().map(|it| it.as_str()).collect()
}

// TODO: Figure out how this should be pickled?
#[pymethods]
impl ModelAnnotation {
    /// Create a new `ModelAnnotation` with an optional string `value`.
    ///
    /// The child annotations can be then set similar to a normal dictionary.
    #[new]
    #[pyo3(signature = (value = None))]
    pub fn new(py: Python, value: Option<String>) -> PyResult<ModelAnnotation> {
        let root: ModelAnnotationRoot = if let Some(value) = value {
            biodivine_lib_param_bn::ModelAnnotation::with_value(value).into()
        } else {
            biodivine_lib_param_bn::ModelAnnotation::new().into()
        };
        Ok(ModelAnnotation {
            root: Py::new(py, root)?,
            path: Vec::new(),
        })
    }

    fn __richcmp__(&self, py: Python, other: &ModelAnnotation, op: CompareOp) -> Py<PyAny> {
        // First, check the paths.
        match op {
            CompareOp::Eq => {
                if self.path != other.path {
                    return false.into_py(py);
                }
            }
            CompareOp::Ne => {
                if self.path != other.path {
                    return true.into_py(py);
                }
            }
            _ => return py.NotImplemented(),
        }

        // If paths match the operator, do the same thing with the root references.
        // Here, we are not doing semantic checking, just pointer equivalence, which makes
        // sure both objects reference the same underlying dictionary.
        match op {
            CompareOp::Eq => (self.root.as_ptr() == other.root.as_ptr()).into_py(py),
            CompareOp::Ne => (self.root.as_ptr() != other.root.as_ptr()).into_py(py),
            _ => unreachable!(),
        }
    }

    fn __copy__(&self) -> ModelAnnotation {
        self.clone()
    }

    pub fn __deepcopy__(&self, py: Python, _memo: &PyAny) -> PyResult<ModelAnnotation> {
        let root_copy = self.root.borrow(py).as_native().clone();
        Ok(ModelAnnotation {
            root: Py::new(py, ModelAnnotationRoot::from(root_copy))?,
            path: self.path.clone(),
        })
    }

    fn __str__(&self, py: Python) -> String {
        let self_path = mk_path(&self.path);
        self.root
            .borrow(py)
            .as_native()
            .get_child(&self_path)
            .map(|it| format!("{}", it))
            .unwrap_or_else(String::new)
    }

    fn __repr__(&self, py: Python) -> String {
        let self_str = self.__str__(py);
        format!("ModelAnnotation.from_aeon({:?})", self_str)
    }

    fn __len__(&self, py: Python) -> usize {
        let self_path = mk_path(&self.path);
        self.root
            .borrow(py)
            .as_native()
            .get_child(&self_path)
            .map(|it| it.children().len())
            .unwrap_or(0)
    }

    fn __getitem__(&self, key: &str) -> ModelAnnotation {
        let mut path = self.path.clone();
        path.push(key.to_string());
        ModelAnnotation {
            root: self.root.clone(),
            path,
        }
    }

    fn __setitem__(&self, key: &str, value: ModelAnnotation, py: Python) {
        // First, find the actual annotation of the given `value`.
        let value_path = mk_path(&value.path);
        let value_root_ref = value.root.borrow(py);
        let value_child = value_root_ref.as_native().get_child(&value_path);

        let mut self_path = mk_path(&self.path);
        if let Some(value_child) = value_child {
            // If the annotation exists, copy it to the new location.
            let mut self_root_ref = self.root.borrow_mut(py);
            self_path.push(key);
            let child = self_root_ref.as_native_mut().ensure_child(&self_path);
            *child = value_child.clone();
        } else {
            // The annotation is `None`, we want to remove the current annotation here.
            let mut self_root_ref = self.root.borrow_mut(py);
            let parent = self_root_ref.as_native_mut().get_mut_child(&self_path);
            if let Some(parent) = parent {
                parent.children_mut().remove(key);
            }
            // If the parent does not exist, the child is already `None`.
        }
    }

    fn __delitem__(&self, key: &str, py: Python) {
        let self_path = mk_path(&self.path);
        let mut self_root_ref = self.root.borrow_mut(py);
        let parent = self_root_ref.as_native_mut().get_mut_child(&self_path);
        if let Some(parent) = parent {
            parent.children_mut().remove(key);
        }
    }

    fn __contains__(&self, key: &str, py: Python) -> bool {
        let mut child_path = mk_path(&self.path);
        child_path.push(key);
        self.root
            .borrow(py)
            .as_native()
            .get_child(&child_path)
            .is_some()
    }

    #[getter]
    pub fn get_value(&self, py: Python) -> Option<String> {
        let self_path = mk_path(&self.path);
        let root_ref = self.root.borrow_mut(py);
        root_ref.as_native().get_value(&self_path).cloned()
    }

    #[setter]
    pub fn set_value(&self, py: Python, value: Option<String>) {
        let self_path = mk_path(&self.path);
        let mut root_ref = self.root.borrow_mut(py);
        let value_ref = root_ref
            .as_native_mut()
            .ensure_child(&self_path)
            .value_mut();
        *value_ref = value;
    }

    /// Parse an annotation object from the string representing the contents of an `.aeon` file.
    #[staticmethod]
    pub fn from_aeon(file_contents: &str, py: Python) -> PyResult<ModelAnnotation> {
        let native = biodivine_lib_param_bn::ModelAnnotation::from_model_string(file_contents);
        let root = Py::new(py, ModelAnnotationRoot::from(native))?;
        Ok(ModelAnnotation {
            root,
            path: Vec::new(),
        })
    }

    /// Parse an annotation object from an `.aeon` file at the given `path`.
    #[staticmethod]
    pub fn from_file(path: &str, py: Python) -> PyResult<ModelAnnotation> {
        if !path.ends_with(".aeon") {
            return throw_runtime_error("Expected path to an `.aeon` file.");
        }

        match std::fs::read_to_string(path) {
            Ok(file_contents) => Self::from_aeon(file_contents.as_str(), py),
            Err(e) => throw_runtime_error(format!("Cannot read file: {}.", e)),
        }
    }

    /// Return the list of annotations that are the direct descendants of this annotation.
    pub fn values(&self, py: Python) -> Vec<ModelAnnotation> {
        let self_path = mk_path(&self.path);
        let root_ref = self.root.borrow(py);
        let child_ref = root_ref.as_native().get_child(&self_path);
        if let Some(child_ref) = child_ref {
            let mut keys = child_ref.children().keys().collect::<Vec<_>>();
            keys.sort();
            keys.into_iter()
                .map(|k| {
                    let mut path = self.path.clone();
                    path.push(k.clone());
                    ModelAnnotation {
                        root: self.root.clone(),
                        path,
                    }
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Return the sorted list of keys that are stored in this annotation.
    pub fn keys(&self, py: Python) -> Vec<String> {
        let self_path = mk_path(&self.path);
        let root_ref = self.root.borrow(py);
        let child_ref = root_ref.as_native().get_child(&self_path);
        if let Some(child_ref) = child_ref {
            let mut keys = child_ref.children().keys().cloned().collect::<Vec<_>>();
            keys.sort();
            keys
        } else {
            Vec::new()
        }
    }

    /// Return the list key-value pairs that correspond to the direct descendants
    /// of this annotation.
    pub fn items(&self, py: Python) -> Vec<(String, ModelAnnotation)> {
        let self_path = mk_path(&self.path);
        let root_ref = self.root.borrow(py);
        let child_ref = root_ref.as_native().get_child(&self_path);
        if let Some(child_ref) = child_ref {
            let mut keys = child_ref.children().keys().collect::<Vec<_>>();
            keys.sort();
            keys.into_iter()
                .map(|k| {
                    let mut path = self.path.clone();
                    path.push(k.clone());
                    (
                        k.clone(),
                        ModelAnnotation {
                            root: self.root.clone(),
                            path,
                        },
                    )
                })
                .collect()
        } else {
            Vec::new()
        }
    }
}
