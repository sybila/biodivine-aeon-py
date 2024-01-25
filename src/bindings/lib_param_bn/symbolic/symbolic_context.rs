use crate::bindings::lib_bdd::bdd::Bdd;
use crate::bindings::lib_bdd::bdd_variable::BddVariable;
use crate::bindings::lib_bdd::bdd_variable_set::BddVariableSet;
use crate::bindings::lib_param_bn::boolean_network::BooleanNetwork;
use crate::bindings::lib_param_bn::parameter_id::ParameterId;
use crate::bindings::lib_param_bn::update_function::UpdateFunction;
use crate::bindings::lib_param_bn::variable_id::VariableId;
use crate::pyo3_utils::{resolve_boolean, richcmp_eq_by_key};
use crate::{index_error, throw_index_error, throw_runtime_error, throw_type_error, AsNative};
use biodivine_lib_param_bn::FnUpdate;
use either::{Either, Left, Right};
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::collections::HashMap;

/// Intuitively, a `SymbolicContext` encodes the entities of a `BooleanNetwork` into a set
/// of symbolic variables managed by a `BddVariableSet`. Using this representation, we can
/// ecode sets of states, sets of function interpretations, or relations over both.
///
/// Internally, each `VariableId` corresponds to one `BddVariable`, and each uninterpreted
/// function corresponds to a table of `2^k` `BddVariable` identifiers, which together represent
/// the logical table of the uninterpreted function.
///
/// An uninterpreted function is created for each explicit and implicit parameter of the
/// `BooleanNetwork` for which a `SymbolicContext` is constructed. The `SymbolicContext` is
/// static and does not update even if the supplied network changes later. Also, keep in mind
/// that implicit parameters are only created for variables with missing update functions,
/// not all network variables.
///
/// Additionally, you can specify your own "extra" symbolic variables. These can be used to build
/// more complex symbolic algorithms on top of the basic encoding, like model checking, control,
/// or trap space detection. These extra variables are grouped with the network variables for
/// convenience. This also determines their ordering within the `BddVariableSet`: the extra
/// variables associated with variable `x` are created right after `x` for best locality.
///
/// Finally, `SymbolicContext` allows to build and interpret `Bdd` objects that are valid in
/// the encoding it describes. For example, you can use `SymbolicContext.mk_update_function`
/// to create a symbolic `Bdd` representation of an `UpdateFunction`.
///
///  > Whenever a `SymbolicContext` returns a list of sortable objects (like `BddVariable`,
/// `VariableId`, or `ParameterId`), it is expected that these objects are sorted.
///
#[pyclass(module = "biodivine_aeon", frozen, subclass)]
#[derive(Clone)]
pub struct SymbolicContext {
    native: biodivine_lib_param_bn::symbolic_async_graph::SymbolicContext,
    // A copy of the underlying variable set for the purpose of BDD linking.
    bdd_vars: Py<BddVariableSet>,
}

impl AsNative<biodivine_lib_param_bn::symbolic_async_graph::SymbolicContext> for SymbolicContext {
    fn as_native(&self) -> &biodivine_lib_param_bn::symbolic_async_graph::SymbolicContext {
        &self.native
    }

    fn as_native_mut(
        &mut self,
    ) -> &mut biodivine_lib_param_bn::symbolic_async_graph::SymbolicContext {
        &mut self.native
    }
}

#[pymethods]
impl SymbolicContext {
    /// A `SymbolicContext` is created by providing a `BooleanNetwork` and optional
    /// `extra_variables` dictionary.
    ///
    /// At the moment, it is required that all explicit parameters (uninterpreted functions) that
    /// are declared in the given network are actually used by some update function in said
    /// network.
    ///
    /// *In the future, this restriction will be lifted, but it is not quite clear how soon will
    /// this happen.*
    ///
    /// Furthermore, due to this dependence on a `BooleanNetwork` structure, a
    /// `SymbolicContext` cannot be currently pickled. It is recommended that you instead save
    /// the `.aeon` representation of the `BooleanNetwork` in question.
    ///
    #[new]
    #[pyo3(signature = (network, extra_variables = None))]
    pub fn new(
        py: Python,
        network: Py<BooleanNetwork>,
        extra_variables: Option<&PyDict>,
    ) -> PyResult<SymbolicContext> {
        let bn = network.borrow(py);
        let mut extra = HashMap::new();
        if let Some(extra_variables) = extra_variables {
            for (k, v) in extra_variables {
                let k = bn.as_ref().resolve_variable(k)?;
                let v = v.extract::<u16>()?;
                extra.insert(k, v);
            }
        }
        let ctx = biodivine_lib_param_bn::symbolic_async_graph::SymbolicContext::with_extra_state_variables(bn.as_native(), &extra);
        let ctx = match ctx {
            Ok(ctx) => ctx,
            Err(e) => return throw_runtime_error(e),
        };
        Ok(SymbolicContext {
            bdd_vars: Py::new(py, BddVariableSet::from(ctx.bdd_variable_set().clone()))?,
            native: ctx,
        })
    }

    fn __richcmp__(&self, py: Python, other: &Self, op: CompareOp) -> Py<PyAny> {
        richcmp_eq_by_key(py, op, &self, &other, |x| x.as_native())
    }

    pub fn __str__(&self) -> String {
        format!(
            "SymbolicContext(network_variables={}, extra_variables={}, explicit_functions={}, implicit_functions={})",
            self.network_variable_count(),
            self.extra_bdd_variable_count(),
            self.explicit_function_count(),
            self.implicit_function_count(),
        )
    }

    /*

        Currently unavailable because we cannot create a context without a BN, but we don't know
        the original BN anymore.

    fn __repr__(&self) -> String {
        unimplemented!()
    }

    fn __getnewargs__(&self) -> String {
        unimplemented!()
    }

     */

    fn __copy__(self_: Py<SymbolicContext>) -> Py<SymbolicContext> {
        self_.clone()
    }

    fn __deepcopy__(&self, _memo: &PyAny) -> SymbolicContext {
        self.clone()
    }

    /// The number of the network variables (or state variables) that are encoded by this `SymbolicContext`.
    pub fn network_variable_count(&self) -> usize {
        self.as_native().num_state_variables()
    }

    /// The names of the network variables that are encoded by this `SymbolicContext`.
    pub fn network_variable_names(&self) -> Vec<String> {
        self.as_native()
            .network_variables()
            .map(|it| self.as_native().get_network_variable_name(it))
            .collect()
    }

    /// The `VariableId` identifiers of the network variables that are encoded by this
    /// `SymbolicContext`.
    pub fn network_variables(&self) -> Vec<VariableId> {
        self.as_native()
            .network_variables()
            .map(VariableId::from)
            .collect()
    }

    /// The `BddVariable` IDs of symbolic variables that encode the network variables
    /// in this `SymbolicContext`.
    pub fn network_bdd_variables(&self) -> Vec<BddVariable> {
        self.as_native()
            .state_variables()
            .iter()
            .cloned()
            .map(BddVariable::from)
            .collect()
    }

    /// Return a `VariableId` of the specified network variable, assuming such variable exists.
    ///
    /// Compare to methods like `BooleanNetwork.find_variable`, this method can also resolve
    /// a `BddVariable` to the corresponding `VariableId` (assuming said `BddVariable` encodes
    /// a network variable).
    pub fn find_network_variable(&self, variable: &PyAny) -> PyResult<Option<VariableId>> {
        if let Ok(id) = variable.extract::<VariableId>() {
            return if id.__index__() < self.network_variable_count() {
                Ok(Some(id))
            } else {
                Ok(None)
            };
        }
        if let Ok(id) = variable.extract::<BddVariable>() {
            return Ok(self
                .as_native()
                .find_state_variable(id.into())
                .map(VariableId::from));
        }
        if let Ok(name) = variable.extract::<String>() {
            return Ok(self
                .as_native()
                .find_network_variable(name.as_str())
                .map(VariableId::from));
        }
        throw_type_error("Expected `VariableId`, `BddVariable` or `str`.")
    }

    /// The same as `SymbolicContext.find_network_variable`, but returns the `BddVariable`
    /// which encodes the specified network variable.
    pub fn find_network_bdd_variable(&self, variable: &PyAny) -> PyResult<Option<BddVariable>> {
        if let Ok(id) = variable.extract::<VariableId>() {
            return if id.__index__() < self.network_variable_count() {
                Ok(Some(self.as_native().get_state_variable(id.into()).into()))
            } else {
                Ok(None)
            };
        }
        if let Ok(id) = variable.extract::<BddVariable>() {
            return if self.as_native().state_variables().contains(id.as_native()) {
                Ok(Some(id))
            } else {
                Ok(None)
            };
        }
        if let Ok(name) = variable.extract::<String>() {
            return Ok(self
                .as_native()
                .find_network_variable(name.as_str())
                .map(|it| self.as_native().get_state_variable(it).into()));
        }
        throw_type_error("Expected `VariableId`, `BddVariable` or `str`.")
    }

    /// The name of a particular network variable.
    pub fn get_network_variable_name(&self, variable: &PyAny) -> PyResult<String> {
        let variable = self.resolve_network_variable(variable)?;
        Ok(self.as_native().get_network_variable_name(variable))
    }

    /// The *total* number of extra symbolic variables that are present in this `SymbolicContext`.
    pub fn extra_bdd_variable_count(&self) -> usize {
        self.as_native().num_extra_state_variables()
    }

    /// The list of all extra symbolic variable in this `SymbolicContext`.
    pub fn extra_bdd_variables_list(&self) -> Vec<BddVariable> {
        self.as_native()
            .all_extra_state_variables()
            .iter()
            .cloned()
            .map(BddVariable::from)
            .collect()
    }

    /// A dictionary which returns the list of extra symbolic variables for each network variable.
    pub fn extra_bdd_variables(&self) -> HashMap<VariableId, Vec<BddVariable>> {
        let mut result = HashMap::new();
        for var in self.as_native().network_variables() {
            let extra = self
                .as_native()
                .extra_state_variables(var)
                .iter()
                .cloned()
                .map(BddVariable::from)
                .collect::<Vec<_>>();
            if !extra.is_empty() {
                result.insert(VariableId::from(var), extra);
            }
        }
        result
    }

    /// The number of explicit functions (parameters) managed by this `SymbolicContext`.
    pub fn explicit_function_count(&self) -> usize {
        self.as_native().network_parameters().count()
    }

    /// The list of `ParameterId` objects identifying the individual explicit functions
    /// in this `SymbolicContext`.
    pub fn explicit_functions(&self) -> Vec<ParameterId> {
        self.as_native()
            .network_parameters()
            .map(ParameterId::from)
            .collect()
    }

    /// The list of all symbolic variables which this `SymbolicContext` uses for the encoding of
    /// the explicit uninterpreted functions.
    pub fn explicit_functions_bdd_variables_list(&self) -> Vec<BddVariable> {
        let mut result = Vec::new();
        for par in self.as_native().network_parameters() {
            let table = self.as_native().get_explicit_function_table(par);
            let vars = table
                .symbolic_variables()
                .iter()
                .cloned()
                .map(BddVariable::from);
            result.extend(vars);
        }
        result
    }

    /// A dictionary which maps the `ParameterId` of an explicit function to the list of symbolic
    /// variables that are used to encode said function.
    pub fn explicit_functions_bdd_variables(&self) -> HashMap<ParameterId, Vec<BddVariable>> {
        let mut result = HashMap::new();
        for par in self.as_native().network_parameters() {
            let table = self.as_native().get_explicit_function_table(par);
            let vars = table
                .symbolic_variables()
                .iter()
                .cloned()
                .map(BddVariable::from)
                .collect::<Vec<_>>();
            result.insert(ParameterId::from(par), vars);
        }
        result
    }

    /// The number of implicit functions (parameters) managed by this `SymbolicContext`.
    pub fn implicit_function_count(&self) -> usize {
        self.as_native().network_implicit_parameters().len()
    }

    /// The list of variables that have an implicit function declared for them.
    pub fn implicit_functions(&self) -> Vec<VariableId> {
        self.as_native()
            .network_implicit_parameters()
            .into_iter()
            .map(VariableId::from)
            .collect()
    }

    /// The list of all symbolic variables which this `SymbolicContext` uses for the encoding of
    /// the implicit uninterpreted functions.
    pub fn implicit_functions_bdd_variables_list(&self) -> Vec<BddVariable> {
        let mut result = Vec::new();
        for var in self.as_native().network_implicit_parameters() {
            let table = self.as_native().get_implicit_function_table(var).unwrap();
            let vars = table
                .symbolic_variables()
                .iter()
                .cloned()
                .map(BddVariable::from);
            result.extend(vars);
        }
        result
    }

    /// A dictionary which maps the `VariableId` of an implicit function to the list of symbolic
    /// variables that are used to encode said function.
    pub fn implicit_functions_bdd_variables(&self) -> HashMap<VariableId, Vec<BddVariable>> {
        let mut result = HashMap::new();
        for var in self.as_native().network_implicit_parameters() {
            let table = self.as_native().get_implicit_function_table(var).unwrap();
            let vars = table
                .symbolic_variables()
                .iter()
                .cloned()
                .map(BddVariable::from)
                .collect::<Vec<_>>();
            result.insert(VariableId::from(var), vars);
        }
        result
    }

    /// The total number of uninterpreted functions in this encoding.
    pub fn function_count(&self) -> usize {
        self.explicit_function_count() + self.implicit_function_count()
    }

    /// The list of all explicit and implicit uninterpreted functions in this encoding.
    pub fn functions<'a>(&self, py: Python<'a>) -> PyResult<&'a PyList> {
        let result = PyList::empty(py);
        for x in self.explicit_functions() {
            result.append(x.into_py(py))?;
        }
        for x in self.implicit_functions() {
            result.append(x.into_py(py))?;
        }
        Ok(result)
    }

    /// The list of all symbolic variables which this `SymbolicContext` uses for the encoding of
    /// the uninterpreted functions (both implicit and explicit).
    pub fn functions_bdd_variables_list(&self) -> Vec<BddVariable> {
        // This list may not sorted in the older library versions.
        let mut result = self
            .as_native()
            .parameter_variables()
            .iter()
            .cloned()
            .map(BddVariable::from)
            .collect::<Vec<_>>();
        result.sort();
        result
    }

    /// A dictionary which maps the `VariableId` and `ParameterId` objects identifying individual
    /// uninterpreted functions to the symbolic variables that are used to encode said function.
    pub fn functions_bdd_variables<'a>(&self, py: Python<'a>) -> PyResult<&'a PyDict> {
        let result = PyDict::new(py);
        for (k, v) in self.explicit_functions_bdd_variables() {
            result.set_item(k.into_py(py), v.into_py(py))?;
        }
        for (k, v) in self.implicit_functions_bdd_variables() {
            result.set_item(k.into_py(py), v.into_py(py))?;
        }
        Ok(result)
    }

    /// Find a `ParameterId` (explicit function) or `VariableId` (implicit function) which
    /// identifies the specified function, or `None` if such function does not exist.
    ///
    /// The function can accept a `BddVariable`. In such case, it will try to identify the
    /// function table in which the variable resides. Note that this is a linear search.
    pub fn find_function(&self, function: &PyAny, py: Python) -> PyResult<Option<PyObject>> {
        if let Ok(id) = function.extract::<ParameterId>() {
            return if id.__index__() < self.explicit_function_count() {
                Ok(Some(id.into_py(py)))
            } else {
                Ok(None)
            };
        }

        if let Ok(id) = function.extract::<VariableId>() {
            let id_native = *id.as_native();
            return if id.__index__() < self.network_variable_count() {
                if self
                    .as_native()
                    .get_implicit_function_table(id_native)
                    .is_some()
                {
                    Ok(Some(id.into_py(py)))
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            };
        }

        if let Ok(name) = function.extract::<&str>() {
            let var_id = self.as_native().find_network_variable(name);
            let par_id = self.as_native().find_network_parameter(name);
            return match (var_id, par_id) {
                (None, None) => Ok(None),
                (Some(_), Some(_)) => unreachable!(),
                (_, Some(par_id)) => Ok(Some(ParameterId::from(par_id).into_py(py))),
                (Some(var_id), _) => {
                    if self
                        .as_native()
                        .get_implicit_function_table(var_id)
                        .is_some()
                    {
                        Ok(Some(VariableId::from(var_id).into_py(py)))
                    } else {
                        Ok(None)
                    }
                }
            };
        }

        if let Ok(bdd_var) = function.extract::<BddVariable>() {
            // Find the variable/parameter which identifies the function in which the variable
            // is used, if any.
            let bdd_var = *bdd_var.as_native();
            for var in self.as_native().network_variables() {
                if let Some(table) = self.as_native().get_implicit_function_table(var) {
                    if table.contains(bdd_var) {
                        return Ok(Some(VariableId::from(var).into_py(py)));
                    }
                }
            }
            for par in self.as_native().network_parameters() {
                let table = self.as_native().get_explicit_function_table(par);
                if table.contains(bdd_var) {
                    return Ok(Some(ParameterId::from(par).into_py(py)));
                }
            }
            return Ok(None);
        }

        throw_type_error("Expected `ParameterId`, `VariableId`, `BddVariable`, or a string name.")
    }

    /// Return the name of an uninterpreted function. For explicit functions, the name of the
    /// network parameter is returned. For implicit functions, the name of the corresponding
    /// network variable is returned.
    pub fn get_function_name(&self, function: &PyAny) -> PyResult<String> {
        match self.resolve_function(function)? {
            Left(var) => Ok(self.as_native().get_network_variable_name(var)),
            Right(par) => Ok(self.as_native().get_network_parameter_name(par)),
        }
    }

    /// Return the arity (the number of arguments) of the specified uninterpreted function.
    pub fn get_function_arity(&self, function: &PyAny) -> PyResult<u16> {
        match self.resolve_function(function)? {
            Left(var) => Ok(self.as_native().get_network_implicit_parameter_arity(var)),
            Right(par) => Ok(self.as_native().get_network_parameter_arity(par)),
        }
    }

    /// Return the "function table" a specified uninterpreted function. A function table consists
    /// of tuples, where each tuple represents one input-output pair of the logical table of the
    /// function in question. The input is a list of Boolean values, the output is a symbolic
    /// variable that represents the output of the function for this input vector.
    pub fn get_function_table(&self, function: &PyAny) -> PyResult<Vec<(Vec<bool>, BddVariable)>> {
        let table = match self.resolve_function(function)? {
            Left(var) => self.as_native().get_implicit_function_table(var).unwrap(),
            Right(par) => self.as_native().get_explicit_function_table(par),
        };
        Ok(table
            .into_iter()
            .map(|(k, v)| (k, BddVariable::from(v)))
            .collect())
    }

    /// Create a new constant (`True`/`False`) `Bdd`.
    pub fn mk_constant(&self, value: &PyAny) -> PyResult<Bdd> {
        let value = resolve_boolean(value)?;
        let rs_bdd = self.as_native().mk_constant(value);
        Ok(Bdd::new_raw_2(self.bdd_vars.clone(), rs_bdd))
    }

    /// Create a new `Bdd` which is true if and only if the specified network variable is true.
    ///
    /// This is equivalent to calling `SymbolicContext.mk_update_function` with
    /// `UpdateFunction.mk_var(variable)` as the argument.
    pub fn mk_network_variable(&self, variable: &PyAny) -> PyResult<Bdd> {
        let variable = self.resolve_network_variable(variable)?;
        let rs_bdd = self.as_native().mk_state_variable_is_true(variable);
        Ok(Bdd::new_raw_2(self.bdd_vars.clone(), rs_bdd))
    }

    /// Create a new `Bdd` which is true if and only if the specified extra symbolic variable
    /// is true. The variable is specified by its associated network variable and index (or offset).
    ///
    /// If the arguments are not provided, the method uses the first network variable and index `0`.
    #[pyo3(signature = (variable = None, index = None))]
    pub fn mk_extra_bdd_variable(
        &self,
        variable: Option<&PyAny>,
        index: Option<usize>,
    ) -> PyResult<Bdd> {
        let variable = if let Some(variable) = variable {
            self.resolve_network_variable(variable)?
        } else {
            biodivine_lib_param_bn::VariableId::from_index(0)
        };
        let index = index.unwrap_or(0);
        let rs_bdd = self
            .as_native()
            .mk_extra_state_variable_is_true(variable, index);
        Ok(Bdd::new_raw_2(self.bdd_vars.clone(), rs_bdd))
    }

    /// Create a `Bdd` which is valid if and only if the specified uninterpreted function is valid.
    ///
    /// The function takes a vector of arguments which must match the arity of the uninterpreted
    /// function. An argument can be either an arbitrary `Bdd` object, or an `UpdateFunction`
    /// from which a `Bdd` is then constructed.
    pub fn mk_function(&self, function: &PyAny, arguments: Vec<&PyAny>) -> PyResult<Bdd> {
        let arguments = arguments
            .into_iter()
            .map(|it| self.resolve_function_bdd(it))
            .collect::<PyResult<Vec<_>>>()?;
        let table = match self.resolve_function(function)? {
            Left(var) => self.as_native().get_implicit_function_table(var).unwrap(),
            Right(par) => self.as_native().get_explicit_function_table(par),
        };
        let rs_bdd = self.as_native().mk_function_table_true(table, &arguments);
        Ok(Bdd::new_raw_2(self.bdd_vars.clone(), rs_bdd))
    }

    /// Create a `Bdd` which is valid if and only if the specified `UpdateFunction` is valid.
    ///
    /// This can be used to build any Boolean function that depends on the network variables and
    /// the explicit uninterpreted functions. It cannot be used to operate on the extra symbolic
    /// variables or implicit uninterpreted functions.
    pub fn mk_update_function(&self, function: &UpdateFunction) -> Bdd {
        let rs_bdd = self.as_native().mk_fn_update_true(function.as_native());
        Bdd::new_raw_2(self.bdd_vars.clone(), rs_bdd)
    }

    /// The underlying `BddVariableSet` used for the encoding.
    pub fn bdd_variable_set(&self) -> Py<BddVariableSet> {
        self.bdd_vars.clone()
    }

    /// This is similar to `BddVariableSet.transfer_from`, but is applied at the level of
    /// symbolic contexts.
    ///
    /// In other words, you can use this method to translate `Bdd` objects between contexts
    /// that use similar variables and parameters, as long as the `Bdd` only uses objects that
    /// are present in both context, and are ordered the same in both contexts.
    ///
    pub fn transfer_from(&self, bdd: &Bdd, old_ctx: &SymbolicContext) -> PyResult<Bdd> {
        let Some(rs_bdd) = self
            .as_native()
            .transfer_from(bdd.as_native(), old_ctx.as_native())
        else {
            return throw_runtime_error("The contexts are not compatible.");
        };
        Ok(Bdd::new_raw_2(self.bdd_vars.clone(), rs_bdd))
    }

    /// Compute a "canonical context" for this `SymbolicContext`. A canonical context has no
    /// extra symbolic variables. In other words, it is a context that contains only
    /// the elements necessary to encode the original `BooleanNetwork` and nothing else.
    ///
    /// You can use this method in combination with `SymbolicContext.transfer_from` in algorithms
    /// that require more complicated symbolic contexts. After such algorithm computes a result,
    /// it can be safely transferred to the "canonical context" which ensures it can be then
    /// interpreted using only the input model, without any internal information about the
    /// algorithm that was used to obtain the result.
    ///
    pub fn to_canonical_context(&self, py: Python) -> PyResult<SymbolicContext> {
        let canonical = self.as_native().as_canonical_context();
        Ok(SymbolicContext {
            bdd_vars: Py::new(
                py,
                BddVariableSet::from(canonical.bdd_variable_set().clone()),
            )?,
            native: canonical,
        })
    }

    /// Create a new `SymbolicContext` which is compatible with the current context (it uses the
    /// same `BddVariableSet`), but is missing the given network variable.
    ///
    /// The new context uses the same `ParameterId` identifiers as the old context, but has
    /// different `VariableId` identifiers, since one of the variables is no longer used, and
    /// `VariableId` identifiers must be always a contiguous sequence. You should use variable
    /// names to "translate" `VariableId` identifiers between the two symbolic context. Of course,
    /// `SymbolicContext.transfer_from` should also still work.
    ///
    /// Note that the extra symbolic variables and implicit function tables do not disappear,
    /// even if they are only used by the eliminated variable. They will still appear in
    /// `SymbolicContext.extra_bdd_variables_list` and
    /// `SymbolicContext.functions_bdd_variables_list`. However, you will no longer
    /// find them in `SymbolicContext.extra_bdd_variables` and
    /// `SymbolicContext.functions_bdd_variables`, since their variable is eliminated.
    pub fn eliminate_network_variable(&self, variable: &PyAny) -> PyResult<SymbolicContext> {
        let variable = self.resolve_network_variable(variable)?;
        let eliminated = self.as_native().eliminate_network_variable(variable);
        Ok(SymbolicContext {
            bdd_vars: self.bdd_vars.clone(),
            native: eliminated,
        })
    }
}

impl SymbolicContext {
    pub fn wrap_native(
        py: Python,
        ctx: biodivine_lib_param_bn::symbolic_async_graph::SymbolicContext,
    ) -> PyResult<SymbolicContext> {
        Ok(SymbolicContext {
            bdd_vars: Py::new(py, BddVariableSet::from(ctx.bdd_variable_set().clone()))?,
            native: ctx,
        })
    }
    pub fn resolve_function_bdd(&self, function: &PyAny) -> PyResult<biodivine_lib_bdd::Bdd> {
        if let Ok(function) = function.extract::<Bdd>() {
            return Ok(function.as_native().clone());
        }
        if let Ok(function) = function.extract::<UpdateFunction>() {
            return Ok(self.as_native().mk_fn_update_true(function.as_native()));
        }
        if let Ok(variable) = function.extract::<VariableId>() {
            return Ok(self
                .as_native()
                .mk_state_variable_is_true(*variable.as_native()));
        }
        if let Ok(function) = function.extract::<&str>() {
            let fake_network = self.mk_fake_network();
            return match FnUpdate::try_from_str(function, &fake_network) {
                Ok(update) => Ok(self.as_native().mk_fn_update_true(&update)),
                Err(e) => throw_runtime_error(format!("Cannot parse function: {}", e)),
            };
        }
        throw_type_error("Expected `Bdd` or `UpdateFunction`.")
    }

    pub fn resolve_function(
        &self,
        function: &PyAny,
    ) -> PyResult<Either<biodivine_lib_param_bn::VariableId, biodivine_lib_param_bn::ParameterId>>
    {
        if let Ok(id) = function.extract::<ParameterId>() {
            return if id.__index__() < self.explicit_function_count() {
                Ok(Right(*id.as_native()))
            } else {
                throw_index_error(format!("Invalid parameter ID `{}`.", id.__index__()))
            };
        }
        if let Ok(id) = function.extract::<VariableId>() {
            let id_native = *id.as_native();
            return if id.__index__() < self.network_variable_count() {
                if self
                    .as_native()
                    .get_implicit_function_table(id_native)
                    .is_some()
                {
                    Ok(Left(id_native))
                } else {
                    throw_index_error(format!(
                        "Variable ID `{}` does not have an implicit function.",
                        id.__index__()
                    ))
                }
            } else {
                throw_index_error(format!("Invalid variable ID `{}`.", id.__index__()))
            };
        }
        if let Ok(name) = function.extract::<&str>() {
            let var_id = self.as_native().find_network_variable(name);
            let par_id = self.as_native().find_network_parameter(name);
            return match (var_id, par_id) {
                (None, None) => throw_index_error(format!(
                    "Name `{}` does not match any variable or parameter.",
                    name
                )),
                (Some(_), Some(_)) => unreachable!(),
                (_, Some(par_id)) => Ok(Right(par_id)),
                (Some(var_id), _) => {
                    if self
                        .as_native()
                        .get_implicit_function_table(var_id)
                        .is_some()
                    {
                        Ok(Left(var_id))
                    } else {
                        throw_index_error(format!(
                            "Variable ID `{}` does not have an implicit function.",
                            var_id.to_index()
                        ))
                    }
                }
            };
        }
        throw_type_error("Expected `ParameterId`, `VariableId`, or string name.")
    }

    pub fn resolve_network_variable(
        &self,
        variable: &PyAny,
    ) -> PyResult<biodivine_lib_param_bn::VariableId> {
        if let Ok(id) = variable.extract::<VariableId>() {
            return if id.__index__() < self.as_native().num_state_variables() {
                Ok(*id.as_native())
            } else {
                throw_index_error(format!("Invalid variable ID `{}`.", id.__index__()))
            };
        }
        if let Ok(id) = variable.extract::<BddVariable>() {
            return self
                .as_native()
                .find_state_variable(id.into())
                .ok_or_else(|| {
                    index_error(format!(
                        "BDD variable `{}` is not a network variable.",
                        id.__index__()
                    ))
                });
        }
        if let Ok(name) = variable.extract::<&str>() {
            return self
                .as_native()
                .find_network_variable(name)
                .ok_or_else(|| index_error(format!("Unknown variable name `{}`.", name)));
        }
        throw_type_error("Expected `VariableId`, `BddVariable` or `str`.")
    }

    /// Create a network that contains all relevant variables and parameters, but no regulations or update
    /// functions. I.e. all the information that is available in a symbolic context.
    ///
    /// Note that this does not preserve extra symbolic variables in any way.
    ///
    /// This is mostly used for functions that *need* the network, but won't actually use it for anything
    /// other than name and arity resolution.
    pub fn mk_fake_network(&self) -> biodivine_lib_param_bn::BooleanNetwork {
        let mut rg = biodivine_lib_param_bn::RegulatoryGraph::new(self.network_variable_names());
        // We have to make fake regulations to preserve the arity of the implicit parameters.
        // It does not really matter what regulations we create here, just make sure they are there.
        for target in self.as_native().network_implicit_parameters() {
            let arity = self
                .as_native()
                .get_network_implicit_parameter_arity(target);
            for regulator in rg.variables().take(arity as usize) {
                rg.add_regulation(
                    self.as_native()
                        .get_network_variable_name(regulator)
                        .as_str(),
                    self.as_native().get_network_variable_name(target).as_str(),
                    false,
                    None,
                )
                .unwrap();
            }
        }
        let mut bn = biodivine_lib_param_bn::BooleanNetwork::new(rg);
        // Copy explicit parameters.
        for param in self.as_native().network_parameters() {
            let arity = self.as_native().get_network_parameter_arity(param);
            bn.add_parameter(
                self.as_native().get_network_parameter_name(param).as_str(),
                arity as u32,
            )
            .unwrap();
        }
        // Add explicit functions for variables that don't have implicit parameters.
        for var in bn.variables() {
            if self.as_native().get_implicit_function_table(var).is_none() {
                bn.set_update_function(var, Some(FnUpdate::Const(false)))
                    .unwrap();
            }
        }
        bn
    }
}
