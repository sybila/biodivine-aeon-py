use crate::bindings::annotations::network_regulation::NetworkRegulationAnnotation;
use crate::bindings::annotations::network_variable::NetworkVariableAnnotation;
use crate::bindings::annotations::regulatory_graph::{REGULATION, VARIABLE};
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
/// can contain a "path prefix" with path segments separated using `:` (path segments can be
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
/// An exception to this may be a case where `is_multivalued:var_1:` has an "optional" value, and
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
#[pyclass(module = "biodivine_aeon", frozen, subclass)]
#[derive(Clone)]
pub struct ModelAnnotation {
    root: Py<ModelAnnotationRoot>,
    path: Vec<String>,
}

#[pyclass(module = "biodivine_aeon", name = "_ModelAnnotation")]
#[derive(Clone, Wrapper)]
pub struct ModelAnnotationRoot(biodivine_lib_param_bn::ModelAnnotation);

/// Useful for creating mutable copies of a path.
fn mk_path(path: &[String]) -> Vec<&str> {
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

    pub fn __richcmp__(&self, py: Python, other: &ModelAnnotation, op: CompareOp) -> Py<PyAny> {
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

    pub fn __copy__(&self) -> ModelAnnotation {
        self.clone()
    }

    pub fn __deepcopy__(&self, py: Python, _memo: &Bound<'_, PyAny>) -> PyResult<ModelAnnotation> {
        let root_copy = self.root.borrow(py).as_native().clone();
        Ok(ModelAnnotation {
            root: Py::new(py, ModelAnnotationRoot::from(root_copy))?,
            path: self.path.clone(),
        })
    }

    pub fn __str__(&self, py: Python) -> String {
        self.root
            .borrow(py)
            .as_native()
            .get_child(&self.path)
            .map(|it| format!("{}", it))
            .unwrap_or_else(String::new)
    }

    pub fn __repr__(&self, py: Python) -> String {
        let self_str = self.__str__(py);
        format!("ModelAnnotation.from_aeon({:?})", self_str)
    }

    pub fn __len__(&self, py: Python) -> usize {
        self.root
            .borrow(py)
            .as_native()
            .get_child(&self.path)
            .map(|it| it.children().len())
            .unwrap_or(0)
    }

    pub fn __getitem__(&self, key: &str) -> ModelAnnotation {
        let mut path = self.path.clone();
        path.push(key.to_string());
        ModelAnnotation {
            root: self.root.clone(),
            path,
        }
    }

    pub fn __setitem__(&self, key: &str, value: ModelAnnotation, py: Python) {
        // First, find the actual annotation of the given `value`.
        let value_root_ref = value.root.borrow(py);
        let value_child = value_root_ref.as_native().get_child(&value.path);

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

    pub fn __delitem__(&self, key: &str, py: Python) {
        let mut self_root_ref = self.root.borrow_mut(py);
        let parent = self_root_ref.as_native_mut().get_mut_child(&self.path);
        if let Some(parent) = parent {
            parent.children_mut().remove(key);
        }
    }

    pub fn __contains__(&self, key: &str, py: Python) -> bool {
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
        let root_ref = self.root.borrow_mut(py);
        root_ref.as_native().get_value(&self.path).cloned()
    }

    #[setter]
    /// Sets the string value of the annotation node at the current path.
    ///
    /// If `value` is `Some`, updates the annotation node's value; if `None`, removes the value, leaving the node present.
    ///
    /// # Examples
    ///
    /// ```
    /// annotation.set_value(py, Some("New value".to_string()));
    /// assert_eq!(annotation.get_value(py), Some("New value".to_string()));
    /// annotation.set_value(py, None);
    /// assert_eq!(annotation.get_value(py), None);
    /// ```
    pub fn set_value(&self, py: Python, value: Option<String>) {
        let mut root_ref = self.root.borrow_mut(py);
        let value_ref = root_ref
            .as_native_mut()
            .ensure_child(&self.path)
            .value_mut();
        *value_ref = value;
    }

    #[getter]
    /// Returns the annotation value as a vector of lines, splitting the string value at newlines.
    ///
    /// If the annotation node has no value, returns `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// let ann = ModelAnnotation::new(Some("line1\nline2".to_string()));
    /// let gil = Python::acquire_gil();
    /// let py = gil.python();
    /// assert_eq!(ann.get_lines(py), Some(vec!["line1".to_string(), "line2".to_string()]));
    /// ```
    pub fn get_lines(&self, py: Python) -> Option<Vec<String>> {
        self.get_value(py)
            .map(|data| data.lines().map(|it| it.to_string()).collect())
    }

    #[setter]
    /// Sets the annotation value as a multiline string constructed from a vector of lines.
    ///
    /// Joins the provided vector of strings with newline characters and sets the result as the annotation value at the current path. If `None` is provided, the annotation value is cleared.
    ///
    /// # Examples
    ///
    /// ```
    /// annotation.set_lines(py, Some(vec!["line1".to_string(), "line2".to_string()]));
    /// assert_eq!(annotation.get_value(py), Some("line1\nline2".to_string()));
    /// annotation.set_lines(py, None);
    /// assert_eq!(annotation.get_value(py), None);
    /// ```
    pub fn set_lines(&self, py: Python, value: Option<Vec<String>>) {
        let value = value.map(|it| it.join("\n"));
        self.set_value(py, value)
    }

    /// Parse an annotation object from the string representing the contents of an `.aeon` file.
    #[staticmethod]
    /// Parses annotation data from a string containing `.aeon` model contents and returns the root annotation node.
    ///
    /// # Examples
    ///
    /// ```
    /// let py = Python::acquire_gil().python();
    /// let aeon_str = "#! model:author = John Doe";
    /// let annotation = ModelAnnotation::from_aeon(py, aeon_str).unwrap();
    /// assert_eq!(annotation.get_value(py), Some("John Doe".to_string()));
    /// ```
    pub fn from_aeon(py: Python, file_contents: &str) -> PyResult<ModelAnnotation> {
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
            Ok(file_contents) => Self::from_aeon(py, file_contents.as_str()),
            Err(e) => throw_runtime_error(format!("Cannot read file: {}.", e)),
        }
    }

    /// Return the list of annotations that are the direct descendants of this annotation.
    pub fn values(&self, py: Python) -> Vec<ModelAnnotation> {
        let root_ref = self.root.borrow(py);
        let child_ref = root_ref.as_native().get_child(&self.path);
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
        let root_ref = self.root.borrow(py);
        let child_ref = root_ref.as_native().get_child(&self.path);
        if let Some(child_ref) = child_ref {
            let mut keys = child_ref.children().keys().cloned().collect::<Vec<_>>();
            keys.sort();
            keys
        } else {
            Vec::new()
        }
    }

    /// Return the list key-value pairs that correspond to the direct descendants
    /// Returns a sorted list of key-annotation pairs for all direct children of this annotation node.
    ///
    /// Each pair consists of the child key and a `ModelAnnotation` representing the child node.
    ///
    /// # Examples
    ///
    /// ```
    /// let annotation = ModelAnnotation::new(Some("root".to_string()), py).unwrap();
    /// let child = ModelAnnotation::new(Some("child".to_string()), py).unwrap();
    /// annotation["child"] = child.clone();
    /// let items = annotation.items(py);
    /// assert_eq!(items.len(), 1);
    /// assert_eq!(items[0].0, "child");
    /// ```    pub fn items(&self, py: Python) -> Vec<(String, ModelAnnotation)> {
        let root_ref = self.root.borrow(py);
        let child_ref = root_ref.as_native().get_child(&self.path);
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

impl ModelAnnotation {
    /// Creates a `ModelAnnotation` referencing the root of the given annotation tree.
    ///
    /// Returns a new `ModelAnnotation` instance at the root path, sharing the provided `ModelAnnotationRoot`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use biodivine_aeon::ModelAnnotationRoot;
    /// # use pyo3::Python;
    /// # Python::with_gil(|py| {
    /// let root = ModelAnnotationRoot::from_aeon("".to_string()).unwrap();
    /// let annotation = ModelAnnotation::from_root(py, root).unwrap();
    /// assert_eq!(annotation.path().len(), 0);
    /// # });
    /// ```
    pub fn from_root(py: Python, root: ModelAnnotationRoot) -> PyResult<ModelAnnotation> {
        let root = Py::new(py, root)?;
        Ok(ModelAnnotation {
            root,
            path: Vec::new(),
        })
    }

    /// Returns the root annotation object associated with this node.
    ///
    /// # Examples
    ///
    /// ```
    /// let annotation = ModelAnnotation::new(Some("value".to_string()));
    /// let root = annotation.to_root();
    /// ```
    pub fn to_root(&self) -> Py<ModelAnnotationRoot> {
        self.root.clone()
    }
}

impl From<Py<ModelAnnotationRoot>> for ModelAnnotation {
    /// Creates a `ModelAnnotation` referencing the root annotation node from a given `ModelAnnotationRoot`.
    ///
    /// # Examples
    ///
    /// ```
    /// let root: Py<ModelAnnotationRoot> = ...; // Assume root is initialized
    /// let annotation = ModelAnnotation::from(root);
    /// assert!(annotation.path.is_empty());
    /// ```
    fn from(value: Py<ModelAnnotationRoot>) -> Self {
        ModelAnnotation {
            root: value,
            path: Vec::new(),
        }
    }
}

/// Helper methods that are used to implement annotations that are officially supported by AEON.
impl ModelAnnotationRoot {
    /// Make a copy with all variable and regulation annotations that are associated
    /// Returns a new annotation root with all variable and regulation annotations for the specified variable names removed.
    ///
    /// Removes both variable-level and regulation-level annotations associated with each provided variable name, including regulations where the variable is a source or target.
    ///
    /// # Parameters
    /// - `names`: Variable names whose annotations and regulations should be removed.
    ///
    /// # Returns
    /// A new `ModelAnnotationRoot` with the specified variables and their regulations removed.
    ///
    /// # Examples
    ///
    /// ```
    /// let root = ModelAnnotationRoot::from_aeon("...aeon content...", py).unwrap();
    /// let updated = root.drop_variables(&vec!["A".to_string(), "B".to_string()], py).unwrap();
    /// // Variables "A" and "B" and their regulations are no longer present in `updated`.
    /// ```    pub fn drop_variables(
        &self,
        names: &[String],
        py: Python,
    ) -> PyResult<Py<ModelAnnotationRoot>> {
        let mut copy_native = self.0.clone();
        let variables = copy_native.get_mut_child(&[VARIABLE]);
        if let Some(variables) = variables {
            let variable_annotations = variables.children_mut();
            for name in names {
                variable_annotations.remove(name);
            }
        }
        let regulations = copy_native.get_mut_child(&[REGULATION]);
        if let Some(regulations) = regulations {
            let source_map = regulations.children_mut();
            for name in names {
                source_map.remove(name);
            }
            for target_data in source_map.values_mut() {
                let target_map = target_data.children_mut();
                for name in names {
                    target_map.remove(name);
                }
            }
        }
        Py::new(py, ModelAnnotationRoot(copy_native))
    }

    /// Make a copy with the specified variable and its associated regulations "inlined".
    ///
    /// This means we try to "merge" the variable data into the variables into which we
    /// are inlining. This usually just means appending whatever is available into the existing
    /// Returns a new annotation root with the specified variable inlined into its targets.
    ///
    /// This operation merges the annotation data of the given variable into all its target variables and updates regulation annotations accordingly. The original variable and its associated regulations are removed, and any overlapping regulations are merged, preferring target-specific data when conflicts occur.
    ///
    /// # Parameters
    /// - `name`: The name of the variable to inline.
    /// - `old_rg`: The regulatory graph describing variable relationships.
    ///
    /// # Returns
    /// A new `ModelAnnotationRoot` with the variable inlined.
    ///
    /// # Examples
    ///
    /// ```
    /// let root = ModelAnnotationRoot::from_aeon("...aeon content...").unwrap();
    /// let rg = RegulatoryGraph::from_aeon("...aeon content...").unwrap();
    /// let py = Python::acquire_gil().python();
    /// let new_root = root.inline_variable("X", &rg, py).unwrap();
    /// ```    pub fn inline_variable(
        &self,
        name: &str,
        old_rg: &biodivine_lib_param_bn::RegulatoryGraph,
        py: Python,
    ) -> PyResult<Py<ModelAnnotationRoot>> {
        // (1) Make a native copy of current annotations.
        let mut copy_native = self.0.clone();

        // At this point, the variable must be known.
        let old_id = old_rg.find_variable(name).unwrap();

        // (2) Append inlined variable to all of its targets (assuming we have some
        // annotation data for it).
        if let Some(source_ann) = self.0.get_child(&[VARIABLE, name]) {
            for target in old_rg.targets(old_id) {
                let target_name = old_rg.get_variable_name(target);
                let target_ann = copy_native.ensure_child(&[VARIABLE, target_name]);
                NetworkVariableAnnotation::extend_with(target_ann, source_ann);
            }
        }

        // (3) Remove the variable from the annotation copy. Also remove any regulations that are
        // associated with the variable (we'll copy these separately in the next step).
        if let Some(variables) = copy_native.get_mut_child(&[VARIABLE]) {
            variables.children_mut().remove(name);
        }
        if let Some(regulations) = copy_native.get_mut_child(&[REGULATION]) {
            regulations.children_mut().remove(name);
            for targets_ann in regulations.children_mut().values_mut() {
                targets_ann.children_mut().remove(name);
            }
        }

        // (4) Append affected regulations into the inlined copies. That is, for every A -> B -> C
        // that got merged into A -> C, we merge the two regulations, preferring
        // the B -> C regulation. However, there is a slight problem if B -> C already exists.
        // In that case, we need to merge everything.
        for regulator in old_rg.regulators(old_id) {
            let regulator_name = old_rg.get_variable_name(regulator);
            for target in old_rg.targets(old_id) {
                let target_name = old_rg.get_variable_name(target);
                // Retrieve old annotations.
                let a_b_ann = self
                    .0
                    .get_child(&[REGULATION, regulator_name, name])
                    .cloned();
                let b_c_ann = self.0.get_child(&[REGULATION, name, target_name]).cloned();
                // Merge the annotations (assuming they are present).
                let ann = match (a_b_ann, b_c_ann) {
                    (Some(a_b), Some(mut b_c)) => {
                        NetworkRegulationAnnotation::extend_with(&mut b_c, &a_b);
                        b_c
                    }
                    (Some(a_b), None) => a_b,
                    (None, Some(b_c)) => b_c,
                    (None, None) => continue, // No annotations, skip.
                };
                if let Some(current_a_c) =
                    copy_native.get_mut_child(&[REGULATION, regulator_name, target_name])
                {
                    // If the annotation already exists, we extend it with new data.
                    NetworkRegulationAnnotation::extend_with(current_a_c, &ann);
                } else {
                    // Otherwise, we create it.
                    copy_native
                        .ensure_child(&[REGULATION, regulator_name])
                        .children_mut()
                        .insert(target_name.clone(), ann);
                }
            }
        }

        Py::new(py, ModelAnnotationRoot(copy_native))
    }

    /// Removes the annotation for a regulation from the specified source variable to the target variable.
    ///
    /// If the regulation annotation exists under the `REGULATION` subtree for the given source and target, it is deleted.  
    /// Does nothing if the specified regulation does not exist.
    ///
    /// # Arguments
    ///
    /// * `source` - The name of the source variable.
    /// * `target` - The name of the target variable.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut root = ModelAnnotationRoot::default();
    /// root.remove_regulation("A", "B").unwrap();
    /// ```    pub fn remove_regulation(&mut self, source: &str, target: &str) -> PyResult<()> {
        if let Some(regulations) = self.0.get_mut_child(&[REGULATION, source]) {
            regulations.children_mut().remove(target);
        }
        Ok(())
    }

    /// Renames a variable in both the "variable" and "regulation" annotation subtrees.
    ///
    /// This updates all annotation keys corresponding to `old_name` to use `new_name` in both the variable and regulation sections of the annotation tree.
    ///
    /// # Arguments
    ///
    /// * `old_name` - The current name of the variable to be renamed.
    /// * `new_name` - The new name to assign to the variable.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut root = ModelAnnotationRoot::from_aeon("...aeon content...").unwrap();
    /// root.rename_variable("old_var", "new_var").unwrap();
    /// ```    pub fn rename_variable(&mut self, old_name: &str, new_name: &str) -> PyResult<()> {
        if let Some(variables) = self.0.get_mut_child(&[VARIABLE]) {
            let variables_map = variables.children_mut();
            let current_data = variables_map.remove(old_name);
            if let Some(current_data) = current_data {
                variables_map.insert(new_name.to_string(), current_data);
            }
        }
        if let Some(regulations) = self.0.get_mut_child(&[REGULATION]) {
            let regulations_map = regulations.children_mut();
            let current_data = regulations_map.remove(old_name);
            if let Some(current_data) = current_data {
                regulations_map.insert(new_name.to_string(), current_data);
            }
            for target_data in regulations_map.values_mut() {
                let target_map = target_data.children_mut();
                let current_data = target_map.remove(old_name);
                if let Some(current_data) = current_data {
                    target_map.insert(new_name.to_string(), current_data);
                }
            }
        }
        Ok(())
    }

    /// Returns a new Python object containing a deep copy of the annotation root.
    ///
    /// The returned object is independent of the original and contains a cloned copy of the native annotation data.
    ///
    /// # Examples
    ///
    /// ```
    /// let root = ModelAnnotationRoot::from_aeon("".to_string()).unwrap();
    /// let py_copy = root.py_copy(py).unwrap();
    /// assert_ne!(root.as_ptr(), py_copy.as_ptr());
    /// ```    pub fn py_copy(&self, py: Python) -> PyResult<Py<ModelAnnotationRoot>> {
        Py::new(py, ModelAnnotationRoot(self.0.clone()))
    }
}
