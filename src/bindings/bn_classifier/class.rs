use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::hash::{DefaultHasher, Hash, Hasher};

use pyo3::basic::CompareOp;
use pyo3::types::{PyAnyMethods, PyTuple};
use pyo3::{Bound, Py, PyAny, PyResult, Python, pyclass, pymethods};

use crate::bindings::lib_param_bn::symbolic::set_color::ColorSet;
use crate::pyo3_utils::richcmp_eq_by_key;
use crate::{throw_runtime_error, throw_type_error};

/// A `Class` is an immutable collection of sorted "features", such that each feature is
/// a described by its string name. A `Class` is used by the various classification workflows
/// in AEON to label a specific mode of behavior that a system can exhibit.
///
/// Depending on which operations are used, a class can behave either as a `set` (each feature
/// can only appear once in a `Class`), or a `list` (multiple features of the same name appear
/// multiple times). This entirely depends on the classification workflow used and is not
/// a property of a `Class` itself (you can even mix the `set` and `list` behavior depending on
/// the exact feature you are adding). Note that an "empty" class is allowed.
///
/// The main reason why we even need `Class` is that lists and sets are not hash-able in Python,
/// hence we can't use them as keys in dictionaries. Because `Class` is immutable, it has a stable
/// hash and is safe to use as a dictionary key.
///
#[pyclass(module = "biodivine_aeon", frozen)]
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Class {
    items: Vec<String>,
}

impl Class {
    pub fn new_native(mut items: Vec<String>) -> Class {
        items.sort();
        Class { items }
    }

    pub fn as_serial_string(&self) -> String {
        // We use +++ to separate individual items in the class list. It is not perfect,
        // but should be Ok for the vast majority of usages.
        for i in &self.items {
            assert!(!i.contains("+++"))
        }
        self.items.join("+++")
    }

    pub fn from_serial_string(value: String) -> Class {
        let items = value
            .split("+++")
            .map(|it| it.to_string())
            .collect::<Vec<_>>();
        Class::new_native(items)
    }
}

#[pymethods]
impl Class {
    /// Create a `Class` from a `list` of string features.
    ///
    /// The `items` are sorted, but duplicates are not removed.
    #[new]
    pub fn new(items: Bound<'_, PyAny>) -> PyResult<Class> {
        let mut items = if let Ok(item) = items.extract::<String>() {
            vec![item]
        } else if let Ok(items) = items.extract::<Vec<String>>() {
            items
        } else if let Ok(items) = items.extract::<HashSet<String>>() {
            Vec::from_iter(items)
        } else {
            return throw_type_error("Expected `str`, `list[str]`, or `set[str]`.");
        };
        for i in &items {
            if i.contains("+++") {
                return throw_runtime_error("Feature names cannot contain `+++`.");
            }
        }
        items.sort();
        Ok(Class { items })
    }

    fn __richcmp__(&self, py: Python, other: &Class, op: CompareOp) -> PyResult<Py<PyAny>> {
        richcmp_eq_by_key(py, op, &self, &other, |x| &x.items)
    }

    fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.items.hash(&mut hasher);
        hasher.finish()
    }

    pub fn __str__(&self) -> String {
        format!("{:?}", self.items)
    }

    pub fn __repr__(&self) -> String {
        format!("Class({})", self.__str__())
    }

    fn __getnewargs__<'a>(&self, py: Python<'a>) -> PyResult<Bound<'a, PyTuple>> {
        PyTuple::new(py, vec![self.feature_list()])
    }

    fn __len__(&self) -> usize {
        self.items.len()
    }

    pub fn __contains__(&self, key: String) -> bool {
        self.items.contains(&key)
    }

    /// Return the `set` of unique features that appear in this `Class`.
    fn feature_set(&self) -> HashSet<String> {
        self.items.iter().cloned().collect()
    }

    /// Return the `list` of features, including duplicates, that appear in this `Class`.
    fn feature_list(&self) -> Vec<String> {
        self.items.clone()
    }

    /// Count the number of times a given `feature` appears in this `Class`.
    fn feature_count(&self, feature: String) -> usize {
        let mut result = 0;
        for x in &self.items {
            match x.cmp(&feature) {
                Ordering::Equal => result += 1,
                Ordering::Greater => return result,
                _ => (),
            }
        }
        result
    }

    /// Create a `Class` instance that extends this class with the given feature (or features).
    /// If an added feature already exists (at least once), it is not added again.
    pub fn ensure(&self, feature: &Bound<'_, PyAny>) -> PyResult<Class> {
        let to_add = if let Ok(feature) = feature.extract::<String>() {
            vec![feature]
        } else if let Ok(list) = feature.extract::<Vec<String>>() {
            list
        } else if let Ok(class) = feature.extract::<Class>() {
            class.items.clone()
        } else {
            return throw_type_error("Expected `str`, `list[str]`, or `Class`.");
        };

        let to_add = to_add
            .into_iter()
            .filter(|x| !self.items.contains(x))
            .collect::<Vec<_>>();

        if to_add.is_empty() {
            Ok(self.clone())
        } else {
            let mut items = self.items.clone();
            items.extend(to_add);
            items.sort();
            Ok(Class { items })
        }
    }

    /// Create a `Class` instance where the given feature (or features) is added. If an added
    /// feature already exists, it is added again.
    pub fn append(&self, feature: &Bound<'_, PyAny>) -> PyResult<Class> {
        let to_add = if let Ok(feature) = feature.extract::<String>() {
            vec![feature]
        } else if let Ok(list) = feature.extract::<Vec<String>>() {
            list
        } else if let Ok(class) = feature.extract::<Class>() {
            class.items.clone()
        } else {
            return throw_type_error("Expected `str`, `list[str]`, or `Class`.");
        };

        let mut items = self.items.clone();
        items.extend(to_add);
        items.sort();
        Ok(Class { items })
    }

    /// Create a `Class` with all occurrences of a particular feature (or features) removed.
    pub fn erase(&self, feature: &Bound<'_, PyAny>) -> PyResult<Class> {
        let to_remove = if let Ok(feature) = feature.extract::<String>() {
            vec![feature]
        } else if let Ok(list) = feature.extract::<Vec<String>>() {
            list
        } else if let Ok(class) = feature.extract::<Class>() {
            class.items.clone()
        } else {
            return throw_type_error("Expected `str`, `list[str]`, or `Class`.");
        };

        let items = self
            .items
            .iter()
            .filter(|it| !to_remove.contains(it))
            .cloned()
            .collect();
        Ok(Class { items })
    }

    /// Create a `Class` with the given feature (or features) removed. Only the specific provided
    /// number of occurrences is removed.
    pub fn minus(&self, feature: &Bound<'_, PyAny>) -> PyResult<Class> {
        let mut to_remove = if let Ok(feature) = feature.extract::<String>() {
            vec![feature]
        } else if let Ok(list) = feature.extract::<Vec<String>>() {
            list
        } else if let Ok(class) = feature.extract::<Class>() {
            class.items.clone()
        } else {
            return throw_type_error("Expected `str`, `list[str]`, or `Class`.");
        };

        let mut retained = Vec::new();
        for test in &self.items {
            if let Some(i) = to_remove.iter().position(|it| it == test) {
                to_remove.remove(i);
            } else {
                retained.push(test.clone())
            }
        }

        Ok(Class { items: retained })
    }
}

pub fn extend_map(m: &mut HashMap<Class, ColorSet>, k: &Class, v: ColorSet) {
    if let Some(slot) = m.get_mut(k) {
        *slot = slot.union(&v);
    } else {
        m.insert(k.clone(), v);
    }
}
