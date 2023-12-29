use crate::bindings::lib_bdd::bdd_variable::BddVariable;
use crate::bindings::lib_bdd::bdd_variable_set::BddVariableSet;
use crate::pyo3_utils::{resolve_boolean, richcmp_eq_inner};
use crate::{throw_runtime_error, throw_type_error, AsNative};
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

/// Represents a *complete* valuation of all variables in a `Bdd`.
///
/// This can be seen as a safer alternative to `list[bool]` which can be also indexed using `BddVariableType`
/// and ensures that the length always matches the total number of the symbolic variables.
///
/// ```python
/// ctx = BddVariableSet(["a", "b", "c"])
///
/// assert BddValuation(ctx).values() == [False, False, False]
///
/// val_1 = BddValuation(ctx, [0,1,1])
/// val_1_copy = BddValuation(val_1)
/// assert val_1 == eval(repr(val_1))
/// assert str(val_1) == "[0,1,1]"
/// assert len(val_1) == 3
/// assert "a" in val_1 and "z" not in val_1
/// assert val_1["a"] == val_1_copy["a"]
/// assert val_1[BddVariable(2)] == val_1_copy[BddVariable(2)]
/// val_1_copy["a"] = 1
/// assert val_1["a"] != val_1_copy["a"]
///
/// valuations_as_keys = { val_1: "foo", val_1_copy: "bar" }
/// assert valuations_as_keys[val_1] == "foo"
/// ```
#[pyclass(module = "biodivine_aeon")]
#[derive(Clone)]
pub struct BddValuation {
    ctx: Py<BddVariableSet>,
    value: biodivine_lib_bdd::BddValuation,
}

#[pymethods]
impl BddValuation {
    #[new]
    #[pyo3(signature = (ctx, values = None))]
    pub fn new(ctx: &PyAny, values: Option<&PyAny>) -> PyResult<BddValuation> {
        if let Ok(valuation) = ctx.extract::<BddValuation>() {
            if values.is_some() {
                return throw_type_error("Unexpected second argument.");
            }
            return Ok(valuation.clone());
        }
        if let Ok(valuation) = ctx.extract::<BddPartialValuation>() {
            if values.is_some() {
                return throw_type_error("Unexpected second argument.");
            }
            return match biodivine_lib_bdd::BddValuation::try_from(valuation.value) {
                Err(_) => throw_runtime_error("Not all variables are fixed."),
                Ok(value) => Ok(BddValuation {
                    ctx: valuation.ctx,
                    value,
                }),
            };
        }
        match ctx.extract::<Py<BddVariableSet>>() {
            Err(_) => throw_type_error(
                "Expected one of `BddValuation`, `BddPartialValuation`, or `BddVariableSet`.",
            ),
            Ok(ctx) => {
                let var_count = ctx.get().variable_count();
                match values {
                    None => {
                        let var_count = u16::try_from(var_count).unwrap();
                        let value = biodivine_lib_bdd::BddValuation::all_false(var_count);
                        Ok(BddValuation { ctx, value })
                    }
                    Some(values) => {
                        if let Ok(list) = values.downcast::<PyList>() {
                            if list.len() != var_count {
                                return throw_runtime_error(format!(
                                    "Expected {} variables, got {}.",
                                    var_count,
                                    list.len()
                                ));
                            }
                            let value = list
                                .iter()
                                .map(resolve_boolean)
                                .collect::<PyResult<Vec<bool>>>()?;
                            let value = biodivine_lib_bdd::BddValuation::new(value);
                            Ok(BddValuation { ctx, value })
                        } else {
                            throw_type_error("Expected `list[BoolType]` or `None`.")
                        }
                    }
                }
            }
        }
    }

    fn __richcmp__(&self, py: Python, other: &BddValuation, op: CompareOp) -> Py<PyAny> {
        richcmp_eq_inner(py, op, &self, &other, |x| &x.value)
    }

    fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.value.hash(&mut hasher);
        hasher.finish()
    }

    fn __str__(&self) -> String {
        self.value.to_string()
    }

    fn __repr__(&self) -> String {
        let ints = self.values().into_iter().map(i32::from).collect::<Vec<_>>();
        format!("BddValuation({}, {:?})", self.ctx.get().__repr__(), ints)
    }

    fn __getnewargs__(&self) -> (Py<BddVariableSet>, Vec<bool>) {
        (self.ctx.clone(), self.values())
    }

    /// Access the underlying `BddVariableSet` connected to this `BddValuation`.
    pub fn __ctx__(&self) -> Py<BddVariableSet> {
        self.ctx.clone()
    }

    fn __len__(&self) -> usize {
        usize::from(self.value.num_vars())
    }

    fn __getitem__(&self, key: &PyAny) -> PyResult<bool> {
        let ctx = self.ctx.get();
        let var = ctx.resolve_variable(key)?;
        Ok(self.value[var])
    }

    fn __setitem__(&mut self, key: &PyAny, value: &PyAny) -> PyResult<()> {
        let ctx = self.ctx.get();
        let value = resolve_boolean(value)?;
        let var = ctx.resolve_variable(key)?;
        self.value[var] = value;
        Ok(())
    }

    fn __contains__(&self, key: &PyAny) -> bool {
        let ctx = self.ctx.get();
        ctx.resolve_variable(key).is_ok()
    }

    /// The list of `BddVariable` keys used by this `BddValuation`.
    ///
    /// ```python
    /// ctx = BddVariableSet(["a", "b", "c"])
    /// val_1 = BddValuation(ctx, [0,1,1])
    /// assert val_1.keys() == ctx.variable_ids()
    /// ```
    fn keys(&self) -> Vec<BddVariable> {
        self.ctx.get().variable_ids()
    }

    /// The list of `bool` values stored in this `BddValuation`.
    ///
    /// ```python
    /// ctx = BddVariableSet(["a", "b", "c"])
    /// val_1 = BddValuation(ctx, [0,1,1])
    /// assert val_1.values() == [False, True, True]
    /// ```
    fn values(&self) -> Vec<bool> {
        self.value.clone().vector()
    }

    /// The list of `(BddVariable, bool)` tuples, similar to `dict.items()` (can be also used to build a dictionary).
    ///
    /// ```python
    /// ctx = BddVariableSet(["a", "b", "c"])
    /// a, b, c = ctx.variable_ids()
    /// val_1 = BddValuation(ctx, [0,1,1])
    /// val_dict = dict(val_1.items())
    /// assert val_dict == { a: False, b: True, c: True }
    /// ```
    fn items(&self) -> Vec<(BddVariable, bool)> {
        self.keys().into_iter().zip(self.values()).collect()
    }

    /// Returns true of this `BddValuation` also matches the constraints of a given `BddPartialValuation`.
    ///
    /// ```python
    /// ctx = BddVariableSet(["a", "b", "c"])
    /// val_1 = BddValuation(ctx, [0,1,1])
    /// val_2 = BddValuation(ctx, [0,0,0])
    /// p_val_1 = BddPartialValuation(ctx, {'a': 0, 'c': 1 })
    /// assert val_1.extends(p_val_1)
    /// assert not val_2.extends(p_val_1)
    /// ```
    fn extends(&self, valuation: &BddPartialValuation) -> bool {
        self.value.extends(&valuation.value)
    }
}

impl AsNative<biodivine_lib_bdd::BddValuation> for BddValuation {
    fn as_native(&self) -> &biodivine_lib_bdd::BddValuation {
        &self.value
    }

    fn as_native_mut(&mut self) -> &mut biodivine_lib_bdd::BddValuation {
        &mut self.value
    }
}

impl BddValuation {
    pub fn new_raw(
        ctx: Py<BddVariableSet>,
        value: biodivine_lib_bdd::BddValuation,
    ) -> BddValuation {
        BddValuation { ctx, value }
    }
}

/// Represents a *partial* valuation of symbolic variables of a `Bdd`.
///
/// This can be seen as a safer alternative to `dict[BddVariable, bool]` that can be also indexed using any
/// `BddVariableType`.
///
/// ```python
/// ctx = BddVariableSet(["a", "b", "c"])
///
/// assert len(BddPartialValuation(ctx)) == 0
///
/// val_1 = BddPartialValuation(ctx, {'a': 0, 'b': 1})
/// val_2 = BddPartialValuation(BddValuation(ctx, [0, 1, 0]))
/// val_3 = BddPartialValuation(val_1)
///
/// assert val_1 == eval(repr(val_1))
/// assert str(val_1) == "{'a': 0, 'b': 1}"
/// assert len(val_1) == 2
/// assert "a" in val_1 and "z" not in val_1
/// assert (val_1['a'] is not None) and (not val_1['a'])
/// assert (val_1['b'] is not None) and (val_1['b'])
/// # For "valid" variables, we return `None`, but fail for invalid variables.
/// assert val_1['c'] is None
/// with pytest.raises(IndexError):
///     assert val_1['z']
/// assert val_1["a"] == val_3["a"]
/// assert val_1[BddVariable(2)] == val_3[BddVariable(2)]
/// val_3["a"] = 1
/// assert val_1["a"] != val_3["a"]
///
/// assert val_1.keys() == [BddVariable(0), BddVariable(1)]
/// assert val_1.values() == [False, True]
/// assert dict(val_1.items()) == val_1.to_dict()
/// ```
#[pyclass(module = "biodivine_aeon")]
#[derive(Clone)]
pub struct BddPartialValuation {
    ctx: Py<BddVariableSet>,
    value: biodivine_lib_bdd::BddPartialValuation,
}

#[pymethods]
impl BddPartialValuation {
    #[new]
    #[pyo3(signature = (ctx, values = None))]
    fn new(ctx: &PyAny, values: Option<&PyAny>) -> PyResult<BddPartialValuation> {
        if let Ok(valuation) = ctx.extract::<BddPartialValuation>() {
            if values.is_some() {
                return throw_type_error("Unexpected second argument.");
            }
            return Ok(valuation.clone());
        }
        if let Ok(valuation) = ctx.extract::<BddValuation>() {
            if values.is_some() {
                return throw_type_error("Unexpected second argument.");
            }
            return Ok(BddPartialValuation {
                ctx: valuation.ctx.clone(),
                value: biodivine_lib_bdd::BddPartialValuation::from(valuation.value.clone()),
            });
        }
        match ctx.extract::<Py<BddVariableSet>>() {
            Err(_) => throw_type_error(
                "Expected one of `BddValuation`, `BddPartialValuation`, or `BddVariableSet`.",
            ),
            Ok(ctx) => match values {
                None => {
                    let value = biodivine_lib_bdd::BddPartialValuation::empty();
                    Ok(BddPartialValuation { ctx, value })
                }
                Some(values) => {
                    if let Ok(dict) = values.downcast::<PyDict>() {
                        let value = dict
                            .iter()
                            .map(|(a, b)| {
                                let a = ctx.get().resolve_variable(a);
                                let b = resolve_boolean(b);
                                match (a, b) {
                                    (Ok(a), Ok(b)) => Ok((a, b)),
                                    (Err(e), _) | (_, Err(e)) => Err(e),
                                }
                            })
                            .collect::<PyResult<Vec<(biodivine_lib_bdd::BddVariable, bool)>>>()?;
                        let value = biodivine_lib_bdd::BddPartialValuation::from_values(&value);
                        Ok(BddPartialValuation { ctx, value })
                    } else {
                        throw_type_error("Expected `dict[BddVariableType, BoolType]` or `None`.")
                    }
                }
            },
        }
    }

    fn __richcmp__(&self, py: Python, other: &BddPartialValuation, op: CompareOp) -> Py<PyAny> {
        richcmp_eq_inner(py, op, &self, &other, |x| &x.value)
    }

    fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.value.hash(&mut hasher);
        hasher.finish()
    }

    fn __str__(&self) -> String {
        let items = self
            .value
            .to_values()
            .into_iter()
            .map(|(var, value)| {
                let name = self.ctx.get().as_native().name_of(var);
                let value = i32::from(value);
                format!("'{}': {}", name, value)
            })
            .collect::<Vec<_>>();
        format!("{{{}}}", items.join(", "))
    }

    fn __repr__(&self) -> String {
        format!(
            "BddPartialValuation({}, {})",
            self.ctx.get().__repr__(),
            self.__str__()
        )
    }

    fn __getnewargs__(&self) -> (Py<BddVariableSet>, HashMap<BddVariable, bool>) {
        (self.ctx.clone(), self.to_dict())
    }

    /// Access the underlying `BddVariableSet` connected to this `BddValuation`.
    pub fn __ctx__(&self) -> Py<BddVariableSet> {
        self.ctx.clone()
    }

    fn __len__(&self) -> usize {
        usize::from(self.value.cardinality())
    }

    fn __getitem__(&self, key: &PyAny) -> PyResult<Option<bool>> {
        let ctx = self.ctx.get();
        let var = ctx.resolve_variable(key)?;
        Ok(self.value[var])
    }

    fn __setitem__(&mut self, key: &PyAny, value: Option<&PyAny>) -> PyResult<()> {
        let ctx = self.ctx.get();
        let value = value.map(resolve_boolean).transpose()?;
        let var = ctx.resolve_variable(key)?;
        self.value[var] = value;
        Ok(())
    }

    fn __delitem__(&mut self, key: &PyAny) -> PyResult<()> {
        let ctx = self.ctx.get();
        let var = ctx.resolve_variable(key)?;
        self.value.unset_value(var);
        Ok(())
    }

    fn __contains__(&self, key: &PyAny) -> bool {
        let ctx = self.ctx.get();
        ctx.resolve_variable(key).is_ok()
    }

    /// The list of `BddVariable` identifiers for which a fixed value is stored in this `BddPartialValuation`.
    pub fn keys(&self) -> Vec<BddVariable> {
        self.value
            .to_values()
            .into_iter()
            .map(|(a, _)| a.into())
            .collect()
    }

    /// The list of `bool` values stored for individual `BddVariable` keys.
    pub fn values(&self) -> Vec<bool> {
        self.value.to_values().into_iter().map(|(_, b)| b).collect()
    }

    /// The list of `(BddVariable, bool)` tuples, similar to `dict.items()`.
    pub fn items(&self) -> Vec<(BddVariable, bool)> {
        self.value
            .to_values()
            .into_iter()
            .map(|(a, b)| (a.into(), b))
            .collect()
    }

    /// A utility method for directly converting this `BddPartialValuation` to `dict[BddVariable, bool]`.
    pub fn to_dict(&self) -> HashMap<BddVariable, bool> {
        self.value
            .to_values()
            .into_iter()
            .map(|(a, b)| (a.into(), b))
            .collect()
    }

    /// True if this valuation is an extension (i.e. a more specified version) of the `other` valuation.
    pub fn extends(&self, other: &BddPartialValuation) -> bool {
        self.value.extends(other.as_native())
    }

    /// Return the set of variables that are actively used by this `BddPartialValuation`.
    ///
    /// (This is equivalent to `BddPartialValuation.keys`, but returns a `set` instead of a `list`)
    pub fn support_set(&self) -> HashSet<BddVariable> {
        self.value
            .to_values()
            .into_iter()
            .map(|(a, _)| a.into())
            .collect()
    }
}

impl AsNative<biodivine_lib_bdd::BddPartialValuation> for BddPartialValuation {
    fn as_native(&self) -> &biodivine_lib_bdd::BddPartialValuation {
        &self.value
    }

    fn as_native_mut(&mut self) -> &mut biodivine_lib_bdd::BddPartialValuation {
        &mut self.value
    }
}

impl BddPartialValuation {
    pub fn new_raw(
        ctx: Py<BddVariableSet>,
        value: biodivine_lib_bdd::BddPartialValuation,
    ) -> BddPartialValuation {
        BddPartialValuation { ctx, value }
    }
}
