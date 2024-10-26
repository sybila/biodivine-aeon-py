use super::regulatory_graph::RegulatoryGraph;
use crate::bindings::lib_param_bn::parameter_id::ParameterId;
use crate::bindings::lib_param_bn::symbolic::symbolic_context::SymbolicContext;
use crate::bindings::lib_param_bn::update_function::UpdateFunction;
use crate::bindings::lib_param_bn::variable_id::VariableId;
use crate::bindings::lib_param_bn::NetworkVariableContext;
use crate::pyo3_utils::richcmp_eq_by_key;
use crate::{runtime_error, throw_index_error, throw_runtime_error, throw_type_error, AsNative};
use biodivine_lib_param_bn::Sign::{Negative, Positive};
use biodivine_lib_param_bn::{FnUpdate, Monotonicity};
use macros::Wrapper;
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::collections::HashMap;
use std::sync::Arc;

/// A `BooleanNetwork` extends a `RegulatoryGraph` with the ability to reference logical
/// parameters (Boolean uninterpreted functions), and with the ability to store an
/// `UpdateFunction` for each network variable.
///
/// Logical parameters come in two varieties:
///
/// An **explicit parameter** is an uninterpreted function with a fixed name that can appear within
/// larger expressions, for example `a | f(b, !c & d)`. An explicit parameter has to be declared
/// using `BooleanNetwork.add_explicit_parameter` and can be referenced using a `ParameterId`.
/// A single explicit parameter can appear in multiple update functions, in which case it is
/// always instantiated to the same Boolean function.
///
/// Meanwhile, an **implicit parameter** arises when a variable has an unknown (unspecified)
/// update function. In such case, we assume the update function of the variable is an
/// anonymous uninterpreted function which depends on all regulators of said variable. Such
/// implicit parameter does not have a `ParameterId`. We instead refer to it using the `VariableId`
/// of the associated variable.
#[pyclass(module="biodivine_aeon", extends=RegulatoryGraph)]
#[derive(Clone, Wrapper)]
pub struct BooleanNetwork(biodivine_lib_param_bn::BooleanNetwork);

impl NetworkVariableContext for BooleanNetwork {
    fn resolve_network_variable(
        &self,
        variable: &Bound<'_, PyAny>,
    ) -> PyResult<biodivine_lib_param_bn::VariableId> {
        if let Ok(id) = variable.extract::<VariableId>() {
            return if id.__index__() < self.as_native().num_vars() {
                Ok(*id.as_native())
            } else {
                throw_index_error(format!("Unknown variable ID `{}`.", id.__index__()))
            };
        }
        if let Ok(name) = variable.extract::<String>() {
            return if let Some(var) = self.as_native().as_graph().find_variable(name.as_str()) {
                Ok(var)
            } else {
                throw_index_error(format!("Unknown variable name `{}`.", name))
            };
        }
        throw_type_error("Expected `VariableId` or `str`.")
    }

    fn get_network_variable_name(&self, variable: biodivine_lib_param_bn::VariableId) -> String {
        self.as_native().get_variable_name(variable).to_string()
    }
}

#[pymethods]
impl BooleanNetwork {
    /// A new `BooleanNetwork` is constructed in a similar fashion to `RegulatoryGraph`, but additionally
    /// allows a list (or dictionary) of string update functions and a list of explicit parameters (a list
    /// is necessary to have the option to specify parameter ordering).
    ///
    /// If variables are not specified, they can be inferred from the list of regulations. However, either
    /// variables *or* regulations need to be specified in a non-empty network. That is, variables and regulations
    /// cannot be currently inferred from functions alone. Similarly, explicit parameters are not inferred from
    /// update functions automatically.
    #[new]
    #[pyo3(signature = (variables = None, regulations = None, parameters = None, functions = None))]
    fn new(
        variables: Option<&Bound<'_, PyAny>>,
        regulations: Option<&Bound<'_, PyList>>,
        parameters: Option<Vec<(String, u32)>>,
        functions: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<(BooleanNetwork, RegulatoryGraph)> {
        let rg = if let Some(x) = variables {
            if let Ok(rg) = x.extract::<RegulatoryGraph>() {
                rg
            } else if let Ok(list) = x.extract::<Vec<String>>() {
                RegulatoryGraph::new(Some(list), regulations)?
            } else {
                return throw_type_error(
                    "Expected `RegulatoryGraph` or `list[str]` of variable names.",
                );
            }
        } else {
            RegulatoryGraph::new(None, regulations)?
        };
        // The functions could be either a `Vec<Option<String>>` or `HashMap<String, String>`.
        // Technically we could also allow FnUpdate, but that would be a huge pain to resolve
        // correctly and probably won't be used.
        let mut native_bn = biodivine_lib_param_bn::BooleanNetwork::new(rg.as_native().clone());
        if let Some(parameters) = parameters {
            for (name, arity) in parameters {
                native_bn
                    .add_parameter(name.as_str(), arity)
                    .map_err(runtime_error)?;
            }
        }
        if let Some(functions) = functions {
            if let Ok(functions) = functions.extract::<Vec<Option<String>>>() {
                for (i, value) in functions.into_iter().enumerate() {
                    if let Some(value) = value {
                        let var = biodivine_lib_param_bn::VariableId::from_index(i);
                        let name = native_bn.get_variable_name(var).clone();
                        native_bn
                            .add_string_update_function(name.as_str(), value.as_str())
                            .map_err(runtime_error)?;
                    }
                }
            } else if let Ok(functions) = functions.extract::<HashMap<String, String>>() {
                for (name, value) in functions {
                    native_bn
                        .add_string_update_function(name.as_str(), value.as_str())
                        .map_err(runtime_error)?;
                }
            } else {
                return throw_type_error("Expected `list[str | None]` or `dict[str, str]`.");
            }
        }
        Ok((BooleanNetwork(native_bn), rg))
    }

    pub fn __str__(self_: PyRef<'_, Self>) -> String {
        format!(
            "BooleanNetwork(variables={}, regulations={}, explicit_parameters={}, implicit_parameters={})",
            self_.as_ref().variable_count(),
            self_.as_ref().regulation_count(),
            self_.as_native().num_parameters(),
            self_.as_native().num_implicit_parameters(),
        )
    }

    pub fn __richcmp__(&self, py: Python, other: &Self, op: CompareOp) -> Py<PyAny> {
        // The BN and its underlying RG should be up-to-date, hence it should be ok to just compare the BN.
        richcmp_eq_by_key(py, op, &self, &other, |x| x.as_native())
    }

    pub fn __repr__(self_: PyRef<'_, Self>) -> String {
        let (names, regulations, parameters, functions) = BooleanNetwork::__getnewargs__(self_);
        let functions = functions
            .into_iter()
            .map(|it| match it {
                None => "None".to_string(),
                Some(fun) => format!("\"{}\"", fun),
            })
            .collect::<Vec<_>>();
        format!(
            "BooleanNetwork({:?}, {:?}, {:?}, [{}])",
            names,
            regulations,
            parameters,
            functions.join(", ")
        )
    }

    #[allow(clippy::type_complexity)]
    pub fn __getnewargs__(
        self_: PyRef<'_, Self>,
    ) -> (
        Vec<String>,
        Vec<String>,
        Vec<(String, u32)>,
        Vec<Option<String>>,
    ) {
        let (names, regulations) = self_.as_ref().__getnewargs__();
        let mut functions = Vec::new();
        for var in self_.as_native().variables() {
            let function = self_.as_native().get_update_function(var).as_ref();
            let function = function.map(|it| it.to_string(self_.as_native()));
            functions.push(function);
        }
        let parameters = self_
            .as_native()
            .parameters()
            .map(|id| {
                let param = self_.as_native().get_parameter(id);
                (param.get_name().clone(), param.get_arity())
            })
            .collect::<Vec<_>>();
        (names, regulations, parameters, functions)
    }

    pub fn __copy__(&self, py: Python) -> PyResult<Py<BooleanNetwork>> {
        self.clone().export_to_python(py)
    }

    pub fn __deepcopy__(
        &self,
        py: Python,
        _memo: &Bound<'_, PyAny>,
    ) -> PyResult<Py<BooleanNetwork>> {
        self.__copy__(py)
    }

    /* First, "override" all methods from RegulatoryGraph that need to behave differently. */

    /// Read a `BooleanNetwork` from a file path.
    ///
    /// Supported file formats are `.aeon`, `.sbml`, or `.bnet`.
    ///
    /// By default, the method reads the underlying regulatory graph just as it is described in the input file.
    /// However, such graph may not always be logically consistent with the actual update functions. If you set
    /// `repair_graph=True`, the underlying graph is instead inferred correctly from the actual update functions.
    #[staticmethod]
    #[pyo3(signature = (file_path, repair_graph = false))]
    pub fn from_file(
        py: Python,
        file_path: &str,
        repair_graph: bool,
    ) -> PyResult<Py<BooleanNetwork>> {
        let bn = biodivine_lib_param_bn::BooleanNetwork::try_from_file(file_path)
            .map_err(runtime_error)?;
        let bn = if repair_graph {
            bn.infer_valid_graph().map_err(runtime_error)?
        } else {
            bn
        };
        BooleanNetwork(bn).export_to_python(py)
    }

    /// Try to read a `BooleanNetwork` from a string representing the contents of an `.aeon` file.
    #[staticmethod]
    pub fn from_aeon(py: Python, file_contents: &str) -> PyResult<Py<BooleanNetwork>> {
        let bn = biodivine_lib_param_bn::BooleanNetwork::try_from(file_contents)
            .map_err(runtime_error)?;
        BooleanNetwork(bn).export_to_python(py)
    }

    /// Convert this `BooleanNetwork` to a string representation of a valid `.aeon` file.
    pub fn to_aeon(&self) -> String {
        self.as_native().to_string()
    }

    /// Update the variable name of the provided `variable`. This does not change the
    /// corresponding `VariableId`.
    pub fn set_variable_name(
        mut self_: PyRefMut<'_, Self>,
        variable: &Bound<'_, PyAny>,
        name: &str,
    ) -> PyResult<()> {
        let var = self_.as_ref().resolve_network_variable(variable)?;
        self_
            .as_mut()
            .as_native_mut()
            .set_variable_name(var, name)
            .map_err(runtime_error)?;
        self_
            .as_native_mut()
            .as_graph_mut()
            .set_variable_name(var, name)
            .map_err(runtime_error)
    }

    /// Add a new regulation to the underlying `RegulatoryGraph` of this `BooleanNetwork`, either
    /// using a `NamedRegulation`, `IdRegulation`, or a string representation compatible
    /// with the `.aeon` format.
    pub fn add_regulation(
        mut self_: PyRefMut<'_, Self>,
        regulation: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let (s, m, o, t) = RegulatoryGraph::resolve_regulation(Some(self_.as_ref()), regulation)?;
        let m = m.as_ref().map(|it| match it {
            Positive => Monotonicity::Activation,
            Negative => Monotonicity::Inhibition,
        });
        self_
            .as_mut()
            .as_native_mut()
            .add_regulation(s.as_str(), t.as_str(), o, m)
            .map_err(runtime_error)?;
        self_
            .as_native_mut()
            .as_graph_mut()
            .add_regulation(s.as_str(), t.as_str(), o, m)
            .map_err(runtime_error)
    }

    /// Remove a regulation that is currently present in the underlying `RegulatoryGraph` of this
    /// `BooleanNetwork`. Returns the `IdRegulation` dictionary that represents the removed
    /// regulation, or throws a `RuntimeError` if the regulation does not exist. Also throws
    /// a `RuntimeError` if the regulation exists, but is used by the corresponding update
    /// function and so cannot be safely removed.
    pub fn remove_regulation<'a>(
        mut self_: PyRefMut<'_, Self>,
        py: Python<'a>,
        source: &Bound<'a, PyAny>,
        target: &Bound<'a, PyAny>,
    ) -> PyResult<Bound<'a, PyDict>> {
        let source = self_.as_ref().resolve_network_variable(source)?;
        let target = self_.as_ref().resolve_network_variable(target)?;

        if let Some(update) = self_.as_native().get_update_function(target) {
            if update.collect_arguments().contains(&source) {
                return throw_runtime_error("Cannot remove regulation that is in use.");
            }
        }

        // Remove from RG.
        self_
            .as_mut()
            .as_native_mut()
            .remove_regulation(source, target)
            .map_err(runtime_error)?;
        // Remove from BN.
        let removed = self_
            .as_native_mut()
            .as_graph_mut()
            .remove_regulation(source, target)
            .map_err(runtime_error)?;
        RegulatoryGraph::encode_regulation(py, &removed)
    }

    /// Update the `sign` and `essential` flags of a regulation in the underlying
    /// `RegulatoryGraph`. If the regulation does not exist, it is created.
    ///
    /// Returns the previous state of the regulation as an `IdRegulation` dictionary,
    /// assuming the regulation already existed.
    pub fn ensure_regulation<'a>(
        mut self_: PyRefMut<'_, Self>,
        py: Python<'a>,
        regulation: &Bound<'a, PyAny>,
    ) -> PyResult<Option<Bound<'a, PyDict>>> {
        // This is a bit inefficient, but should be good enough for now.
        let (s, m, o, t) = RegulatoryGraph::resolve_regulation(Some(self_.as_ref()), regulation)?;
        let source = self_
            .as_ref()
            .as_native()
            .find_variable(s.as_str())
            .unwrap();
        let target = self_
            .as_ref()
            .as_native()
            .find_variable(t.as_str())
            .unwrap();
        let m = m.as_ref().map(|it| match it {
            Positive => Monotonicity::Activation,
            Negative => Monotonicity::Inhibition,
        });
        // Remove old regulation from both BN and RN.
        let _ignore = self_
            .as_native_mut()
            .as_graph_mut()
            .remove_regulation(source, target);
        let old = self_
            .as_mut()
            .as_native_mut()
            .remove_regulation(source, target)
            .ok();

        // Add new regulation to both BN and RN.
        self_
            .as_native_mut()
            .as_graph_mut()
            .add_regulation(s.as_str(), t.as_str(), o, m)
            .map_err(runtime_error)?;
        self_
            .as_mut()
            .as_native_mut()
            .add_regulation(s.as_str(), t.as_str(), o, m)
            .map_err(runtime_error)?;

        old.map(|it| RegulatoryGraph::encode_regulation(py, &it))
            .transpose()
    }

    /// Create a copy of this `BooleanNetwork` that is extended with the given list of `variables`.
    ///
    /// The new variables are added *after* the existing ones, so any previously used
    /// `VariableId` references are still valid. However, the added names must still be unique
    /// within the new network.
    pub fn extend(
        self_: PyRef<'_, Self>,
        py: Python,
        variables: Vec<String>,
    ) -> PyResult<Py<BooleanNetwork>> {
        let extended_rg = self_.as_ref().extend(variables)?;
        let mut extended_bn =
            biodivine_lib_param_bn::BooleanNetwork::new(extended_rg.as_native().clone());
        for param in self_.as_native().parameters() {
            let param = self_.as_native().get_parameter(param);
            extended_bn
                .add_parameter(param.get_name(), param.get_arity())
                .unwrap_or_else(|_e| {
                    unreachable!("Parameter copy is guaranteed to be valid in the new BN.");
                });
        }
        for var in self_.as_native().variables() {
            if let Some(fun) = self_.as_native().get_update_function(var) {
                extended_bn
                    .set_update_function(var, Some(fun.clone()))
                    .unwrap_or_else(|_e| {
                        unreachable!("Function copy is guaranteed to be valid in the new BN.");
                    });
            }
        }
        Py::new(py, (BooleanNetwork(extended_bn), extended_rg))
    }

    /// Create a copy of this `BooleanNetwork` with all the specified variables
    /// (and their associated regulations) removed.
    ///
    /// If the removed variable also appears in some update function, the update function
    /// is set to `None`, but only if the variable actually appears in the update function
    /// (i.e. just having a removed regulator does not automatically cause the function to
    /// be removed). If a parameter becomes unused by removing said variables, the parameter
    /// is removed as well.
    ///
    /// Throws `RuntimeError` if one of the variables does not exist.
    ///
    /// The new graph follows the variable ordering of the old graph, but since there are now
    /// variables that are missing in the new graph, the `VariableId` objects are not compatible
    /// with the original graph.
    pub fn drop(
        self_: PyRef<'_, Self>,
        py: Python,
        variables: &Bound<'_, PyAny>,
    ) -> PyResult<Py<BooleanNetwork>> {
        let removed = self_.as_ref().resolve_variables(variables)?;

        let drop_rg = self_.as_ref().drop(variables)?;
        let mut drop_bn = biodivine_lib_param_bn::BooleanNetwork::new(drop_rg.as_native().clone());
        for param in self_.as_native().parameters() {
            let param = self_.as_native().get_parameter(param);
            drop_bn
                .add_parameter(param.get_name(), param.get_arity())
                .unwrap();
        }

        for var in self_.as_native().variables() {
            if removed.contains(&var) {
                // Do not copy removed variables.
                continue;
            }

            let var_name = self_.as_native().get_variable_name(var);
            if let Some(fun) = self_.as_native().get_update_function(var) {
                let has_removed_variable = fun
                    .collect_arguments()
                    .into_iter()
                    .any(|var| removed.contains(&var));
                if !has_removed_variable {
                    drop_bn
                        .add_string_update_function(
                            var_name,
                            fun.to_string(self_.as_native()).as_str(),
                        )
                        .unwrap_or_else(|e| {
                            unreachable!(
                                "Function copy is guaranteed to be valid in the new BN: {}",
                                e
                            );
                        });
                }
            }
        }

        Py::new(
            py,
            (BooleanNetwork(drop_bn.prune_unused_parameters()), drop_rg),
        )
    }

    /// Produce a new `BooleanNetwork` where the given variable has been eliminated by inlining
    /// its update function into all downstream variables.
    ///
    /// Note that the inlining operation is purely syntactic. This means that even if we create
    /// new regulations when relevant, the resulting regulatory graph may be inconsistent with the
    /// update functions. If you set `repair_graph` to `True`, the method will perform semantic
    /// analysis on the new functions and repair regulatory graph where relevant. If `repair_graph`
    /// is set to `False`, the operation does not perform any such post-processing.
    ///
    /// A simple example where "inconsistent" regulatory graph is produced is the inlining of a
    /// constant input variable `f_a = true` into the update function `f_c = a | b`. Here, we have
    /// regulations `a -> c` and `b -> c`. However, for the inlined function, we have
    /// `f_c = (true | b) = true`. As such, `b` is no longer essential in `f_c` and the resulting
    /// model is thus not "logically consistent". We need to either fix this manually, or using
    /// `BooleanNetwork.infer_valid_graph`.
    ///
    /// #### Limitations
    /// At the moment, the reduced variable cannot have a self-regulation. If such a variable is
    /// targeted with reduction, the function throws a `RuntimeError`. If a variable is inlined
    /// into a missing function, the function is given a name as a new parameter, and the variable
    /// is inlined into this new parameter function.
    ///
    /// Note that variables that don't regulate anything (outputs) are simply removed by this
    /// reduction. However, this is correct behaviour, just not super intuitive.
    ///
    /// Also note that because the set of variables is different between this and the resulting
    /// `BooleanNetwork`, any `VariableId` that is valid in this network is not valid in the
    /// resulting network.
    ///
    /// #### Logical parameters
    ///
    /// Finally, note the set of admissible parameter instantiations (interpretations of
    /// uninterpreted functions) can change between the original and the reduced model. The reason
    /// for this is the same as in the example of a "logically inconsistent" system described
    /// above. For example, consider `a -> b` and `b -> c`, but also `a -| c`. Then, let's have
    /// `f_b = f(a)` and `f_c = b & !a`. Then `f(a) = a` is the only admissible interpretation
    /// of `f`. However, suppose we inline variable `b`, obtaining `f_c = f(a) & !a` with
    /// regulation `a -? c` (because `a -> b -> c` and `a -| c` in the original system). Then `f`
    /// can actually be `false`, `a`, or `!a`.
    ///
    /// This does not mean you cannot use reduction on systems with uninterpreted functions at all,
    /// but be careful about the new meaning of the static constraints on these functions.
    ///
    #[pyo3(signature = (variable, repair_graph = false))]
    pub fn inline_variable(
        self_: PyRef<'_, Self>,
        py: Python,
        variable: &Bound<'_, PyAny>,
        repair_graph: bool,
    ) -> PyResult<Py<BooleanNetwork>> {
        let variable = self_.as_ref().resolve_network_variable(variable)?;
        let Some(bn) = self_.as_native().inline_variable(variable, repair_graph) else {
            return throw_runtime_error("Variable has a self-regulation.");
        };
        BooleanNetwork(bn).export_to_python(py)
    }

    /// Return *a copy* of the underlying `RegulatoryGraph` for this `BooleanNetwork`.
    #[allow(clippy::wrong_self_convention)]
    pub fn to_graph(self_: PyRef<'_, Self>) -> RegulatoryGraph {
        self_.as_ref().clone()
    }

    /// Try to load a Boolean network from the contents of a `.bnet` model file.
    ///
    /// Note that this is currently only a "best effort" implementation, and you may encounter
    /// unsupported `.bnet` models.
    ///
    /// We also support some features that `.bnet` does not, in particular, you can use
    /// Boolean constants (`true`/`false`). However, there are other things that we do not
    /// support, since `.bnet` can essentially use R syntax to define more complex functions,
    /// but in practice this is not used anywhere.
    ///
    /// Note that `.bnet` files do not have any information about regulations. As such, by default regulations
    /// are loaded as non-essential with no fixed sign. If you set `repair_graph=True`, then we use a symbolic
    /// method that infers these annotations automatically.
    #[staticmethod]
    #[pyo3(signature = (file_contents, repair_graph = false))]
    pub fn from_bnet(
        py: Python,
        file_contents: &str,
        repair_graph: bool,
    ) -> PyResult<Py<BooleanNetwork>> {
        let bn = biodivine_lib_param_bn::BooleanNetwork::try_from_bnet(file_contents)
            .map_err(runtime_error)?;
        let bn = if repair_graph {
            bn.infer_valid_graph().map_err(runtime_error)?
        } else {
            bn
        };
        BooleanNetwork(bn).export_to_python(py)
    }

    /// Produce a `.bnet` string representation of this `BooleanNetwork`.
    ///
    /// Returns an error if the network is parametrised and thus cannot be converted to `.bnet`.
    ///
    /// By default, the method will rename any variables whose names are not supported in `.bnet`.
    /// However, you can instead ask it to throw a `RuntimeError` using the
    /// `rename_if_necessary` flag.
    ///
    #[pyo3(signature = (rename_if_necessary = true))]
    pub fn to_bnet(&self, rename_if_necessary: bool) -> PyResult<String> {
        self.as_native()
            .to_bnet(rename_if_necessary)
            .map_err(runtime_error)
    }

    /// Try to load a `BooleanNetwork` from the contents of an `.sbml` model file.
    #[staticmethod]
    pub fn from_sbml(py: Python, file_contents: &str) -> PyResult<Py<BooleanNetwork>> {
        let (bn, _) = biodivine_lib_param_bn::BooleanNetwork::try_from_sbml(file_contents)
            .map_err(runtime_error)?;
        BooleanNetwork(bn).export_to_python(py)
    }

    /// Produce a `.sbml` string representation of this `BooleanNetwork`.
    pub fn to_sbml(&self) -> String {
        self.as_native().to_sbml(None)
    }

    /// The number of *explicit parameters*, i.e. named uninterpreted functions in this network.
    pub fn explicit_parameter_count(&self) -> usize {
        self.as_native().num_parameters()
    }

    /// The number of *implicit parameters*, i.e. anonymous uninterpreted functions in this network.
    pub fn implicit_parameter_count(&self) -> usize {
        self.as_native().num_implicit_parameters()
    }

    /// Return the dictionary of `ParameterId` identifiers of explicit parameters and their
    /// corresponding arities.
    ///
    /// Keep in mind that the iteration order of keys of this dictionary is not deterministic.
    /// If you need deterministic iteration, you have to sort the keys.
    pub fn explicit_parameters(&self) -> HashMap<ParameterId, u32> {
        self.as_native()
            .parameters()
            .map(|it| {
                (
                    ParameterId::from(it),
                    self.as_native().get_parameter(it).get_arity(),
                )
            })
            .collect()
    }

    /// Return the dictionary of `VariableId` identifiers for variables with anonymous update
    /// functions and their corresponding arities.
    ///
    /// Keep in mind that the iteration order of keys of this dictionary is not deterministic.
    /// If you need deterministic iteration, you have to sort the keys.
    pub fn implicit_parameters(&self) -> HashMap<VariableId, usize> {
        self.as_native()
            .implicit_parameters()
            .into_iter()
            .map(|it| (VariableId::from(it), self.as_native().regulators(it).len()))
            .collect()
    }

    /// Return the list of explicit parameter names. The names as sorted in accordance with the
    /// `ParameterId` identifiers.
    pub fn explicit_parameter_names(&self) -> Vec<String> {
        self.as_native()
            .parameters()
            .map(|it| self.as_native().get_parameter(it).get_name().clone())
            .collect()
    }

    /// Return the name of the given explicit parameter.
    pub fn get_explicit_parameter_name(&self, parameter: &Bound<'_, PyAny>) -> PyResult<String> {
        let parameter = self.resolve_parameter(parameter)?;
        Ok(self.as_native().get_parameter(parameter).get_name().clone())
    }

    /// Return the arity of an explicit parameter.
    pub fn get_explicit_parameter_arity(&self, parameter: &Bound<'_, PyAny>) -> PyResult<u32> {
        let parameter = self.resolve_parameter(parameter)?;
        Ok(self.as_native().get_parameter(parameter).get_arity())
    }

    /// Return a `ParameterId` identifier of the requested `parameter`, or `None` if the
    /// uninterpreted function does not exist in this `BooleanNetwork`.
    pub fn find_explicit_parameter(
        &self,
        parameter: &Bound<'_, PyAny>,
    ) -> PyResult<Option<ParameterId>> {
        if let Ok(id) = parameter.extract::<ParameterId>() {
            return if id.__index__() < self.explicit_parameter_count() {
                Ok(Some(id))
            } else {
                Ok(None)
            };
        }
        if let Ok(name) = parameter.extract::<String>() {
            return Ok(self
                .as_native()
                .find_parameter(name.as_str())
                .map(Into::into));
        }
        throw_type_error("Expected `ParameterId` or `str`.")
    }

    /// Create a new explicit parameter.
    ///
    /// the parameter name must be unique among existing variables and parameters.
    pub fn add_explicit_parameter(&mut self, name: &str, arity: u32) -> PyResult<ParameterId> {
        self.as_native_mut()
            .add_parameter(name, arity)
            .map_err(runtime_error)
            .map(ParameterId::from)
    }

    /// Get the `UpdateFunction` of a particular network variable, or `None` if the function
    /// is unknown (i.e. na implicit parameter).
    pub fn get_update_function(
        self_: Py<BooleanNetwork>,
        py: Python,
        variable: &Bound<'_, PyAny>,
    ) -> PyResult<Option<UpdateFunction>> {
        let fun = {
            let self_ref = self_.borrow(py);
            let variable = self_ref.as_ref().resolve_network_variable(variable)?;
            self_ref.as_native().get_update_function(variable).clone()
        };
        if let Some(fun) = fun {
            Ok(Some(UpdateFunction::new_raw(self_, Arc::new(fun))))
        } else {
            Ok(None)
        }
    }

    /// Update the current update function of the specified `variable` and return the previous
    /// function.
    ///
    /// All variables and parameters used in the new function must already exist in the
    /// `BooleanNetwork`. Furthermore, the function can only use variables that are declared
    /// as regulators of the target `variable`.
    pub fn set_update_function(
        self_: Py<BooleanNetwork>,
        py: Python,
        variable: &Bound<'_, PyAny>,
        function: &Bound<'_, PyAny>,
    ) -> PyResult<Option<UpdateFunction>> {
        let old_fun = Self::get_update_function(self_.clone(), py, variable)?;
        let new_fun = if function.is_none() {
            None
        } else if let Ok(fun) = function.extract::<UpdateFunction>() {
            Some(fun.as_native().clone())
        } else if let Ok(fun) = function.extract::<String>() {
            let bn = self_.borrow(py);
            Some(FnUpdate::try_from_str(fun.as_str(), bn.as_native()).map_err(runtime_error)?)
        } else {
            return throw_type_error("Expected `UpdateFunction` or `str`.");
        };
        let mut bn = self_.borrow_mut(py);
        let variable = bn.as_ref().resolve_network_variable(variable)?;
        bn.as_native_mut()
            .set_update_function(variable, new_fun)
            .map_err(runtime_error)?;
        Ok(old_fun)
    }

    /// Infer a *sufficient local regulatory graph* based on the update functions of
    /// this `BooleanNetwork`.
    ///
    /// #### Notes
    ///
    ///  - The method will simplify the update functions when it detects that a variable
    ///    is not an essential input of the update function (for any instantiation).
    ///  - This also removes any unused parameters (uninterpreted functions), or parameters that
    ///    become unused due to the transformation.
    ///  - For fully specified update functions, the method simply updates the regulations to
    ///    match the actual behaviour of the function (i.e. any non-essential regulations are
    ///    removed, sign is always specified unless the input is truly non-monotonic).
    ///  - For regulations that interact with uninterpreted functions:
    ///     - Essentiality: If there are no instantiations where the regulation is essential,
    ///       we remove the regulation completely. Otherwise, we preserve the original
    ///       essentiality constraint.
    ///     - Monotonicity: Similarly, if every instantiation of the function has a known
    ///       monotonicity, then we use the corresponding sign. Otherwise, if the original
    ///       constraint is still satisfied for a subset of all instantiations, we use the
    ///       original one.
    ///
    /// #### Limitations
    ///
    /// The result only guarantees that the constraints are *locally consistent**, meaning they
    /// are valid when applied to the singular update function, not necessarily the whole model.
    /// In other words, uninterpreted functions that are used by multiple variables can still cause
    /// the model to be invalid if they use contradictory constraints, but this usually cannot be
    /// resolved deterministically because we would have to pick which constraint should be
    /// relaxed to resolve the conflict.
    ///
    /// #### Output
    ///
    /// The function can fail if, for whatever reason, it cannot create `SymbolicContext` for
    /// the input network.
    ///
    pub fn infer_valid_graph(&self, py: Python) -> PyResult<Py<BooleanNetwork>> {
        self.as_native()
            .infer_valid_graph()
            .map_err(runtime_error)
            .and_then(|it| BooleanNetwork(it).export_to_python(py))
    }

    /// Make a copy of this `BooleanNetwork` with all constraints on the regulations removed.
    /// In particular, every regulation is set to non-essential with an unknown sign.
    pub fn remove_regulation_constraints(&self, py: Python) -> PyResult<Py<BooleanNetwork>> {
        let native = self.as_native().remove_static_constraints();
        BooleanNetwork(native).export_to_python(py)
    }

    /// Similar to `BooleanNetwork.inline_inputs`, but inlines constant values (i.e. `x=true` or
    /// `x=false`).
    ///
    /// By default, the constant check is purely syntactic, but we do perform basic constant
    /// propagation (e.g. `x | true = true`) in order to catch the most obvious non-trivial cases.
    /// However, you can set `infer_constants` to `True`, in which case a symbolic method will
    /// be used to check if the variable has a constant function. Note that this can also
    /// eliminate parameters in some cases (e.g. inlining `x=true` into `y=x | f(z)`).
    ///
    /// Furthermore, similar to `BooleanNetwork.inline_inputs`, you can set `repair_graph` to
    /// `true` to fix any inconsistent regulations that arise due to the inlining
    /// (this is highly recommended if you are using `infer_constants`).
    ///
    /// **The inlining is performed iteratively, meaning if the inlining produces a new constant,
    /// it is eventually also inlined.**
    ///
    /// The method can fail if `infer_constants` or `repair_graph` is specified and the network
    /// does not have a valid symbolic representation.
    ///
    #[pyo3(signature = (infer_constants = false, repair_graph = false))]
    pub fn inline_constants(
        &self,
        py: Python,
        infer_constants: bool,
        repair_graph: bool,
    ) -> PyResult<Py<BooleanNetwork>> {
        let bn = self
            .as_native()
            .inline_constants(infer_constants, repair_graph);
        BooleanNetwork(bn).export_to_python(py)
    }

    /// Try to inline the input nodes (variables) of the network as logical parameters
    /// (uninterpreted functions of arity 0).
    ///
    /// Here, an "input" is a variable `x` such that:
    ///   - `x` has no incoming regulations and a missing update function.
    ///   - `x` has an identity update function. This is either checked syntactically (default),
    ///     or semantically using BDDs (if `infer_inputs` is set to `True`).
    ///
    /// Note that this does not include constant nodes (e.g. `x=true`). These are handled
    /// separately by `BooleanNetwork.inline_constants`. Also note that input variables that
    /// do not influence any other variable are removed.
    ///
    /// Variables with update functions `x=f(true, false)` or `x=a` (where `a` is a zero-arity
    /// parameter) are currently not considered to be inputs, although their role is conceptually
    /// similar.
    ///
    /// This method is equivalent to calling `BooleanNetwork.inline_variable` on each input,
    /// but the current implementation of `BooleanNetwork.inline_variable` does not permit
    /// inlining of self-regulating variables, hence we have a special method for inputs only.
    ///
    /// Finally, just as `BooleanNetwork.inline_variable`, the method can generate an
    /// inconsistent regulatory graph. If `repair_graph` is set to `True`, the static properties
    /// of relevant regulations are inferred using BDDs.
    ///
    #[pyo3(signature = (infer_inputs = false, repair_graph = false))]
    pub fn inline_inputs(
        &self,
        py: Python,
        infer_inputs: bool,
        repair_graph: bool,
    ) -> PyResult<Py<BooleanNetwork>> {
        let bn = self.as_native().inline_inputs(infer_inputs, repair_graph);
        BooleanNetwork(bn).export_to_python(py)
    }

    /// Return a copy of this network where all unused explicit parameters (uninterpreted functions)
    /// are removed.
    ///
    /// Note that `VariableId` objects are still valid for the result network, but `ParameterId`
    /// objects can now refer to different parameters.
    pub fn prune_unused_parameters(&self, py: Python) -> PyResult<Py<BooleanNetwork>> {
        let bn = self.as_native().prune_unused_parameters();
        BooleanNetwork(bn).export_to_python(py)
    }

    /// Replaces an implicit parameter (i.e. an anonymous update function) with an explicit
    /// parameter of the given name. If no name is provided, a default name is generated instead.
    ///
    /// The arguments of the newly created update function follow the ordering of variable IDs.
    #[pyo3(signature = (variable, name = None))]
    pub fn assign_parameter_name(
        &mut self,
        variable: &Bound<'_, PyAny>,
        name: Option<String>,
    ) -> PyResult<ParameterId> {
        let variable = self.resolve_network_variable(variable)?;
        let name = name.as_deref();
        self.as_native_mut()
            .assign_parameter_name(variable, name)
            .map_err(runtime_error)
            .map(|it| it.into())
    }

    /// Replaces all implicit parameters with explicit counterparts using default names.
    ///
    /// See also `BooleanNetwork.assign_parameter_name`.
    pub fn name_implicit_parameters(&self, py: Python) -> PyResult<Py<BooleanNetwork>> {
        let new_bn = self.as_native().name_implicit_parameters();
        BooleanNetwork(new_bn).export_to_python(py)
    }

    /// Returns `True` if the given `variable` is an input of the `BooleanNetwork`.
    ///
    /// Input can be either:
    ///  - A variable with no incoming regulations and no update function.
    ///  - A variable with a positive self-regulation and an update function that is equivalent
    ///    to an identity function.
    ///
    /// Note that by default, function equivalent is tested only syntactically (with basic
    /// simplifications applied). To test equivalence semantically, you have to provide
    /// a `SymbolicContext` object as the second argument.
    #[pyo3(signature = (variable, ctx = None))]
    pub fn is_variable_input(
        &self,
        variable: &Bound<PyAny>,
        ctx: Option<&SymbolicContext>,
    ) -> PyResult<bool> {
        let variable = self.resolve_network_variable(variable)?;
        let ctx = ctx.map(|it| it.as_native());
        Ok(self.as_native().is_var_input(variable, ctx))
    }

    /// Tests whether the given `variable` is a constant of the `BooleanNetwork`. A variable
    /// is a constant if it's update function is equivalent to `True` or `False`. Note that
    /// this differs from *inputs*, whose update function is equivalent to identity.
    ///
    /// If the variable is not constant, the function returns `None`. If it is constant,
    /// it returns its constant value.
    ///
    /// By default, function equivalent is tested only syntactically (with basic
    /// simplifications applied). To test equivalence semantically, you have to provide
    /// a `SymbolicContext` object as the second argument.
    #[pyo3(signature = (variable, ctx = None))]
    pub fn is_variable_constant(
        &self,
        variable: &Bound<PyAny>,
        ctx: Option<&SymbolicContext>,
    ) -> PyResult<Option<bool>> {
        let variable = self.resolve_network_variable(variable)?;
        let ctx = ctx.map(|it| it.as_native());
        Ok(self.as_native().is_var_constant(variable, ctx))
    }

    /// Return the list of all inputs that are present in this `BooleanNetwork`. See also
    /// `BooleanNetwork.is_var_input`.
    ///
    /// If `infer=True`, the method will use symbolic equivalence check for identifying
    /// input variables, which is more accurate but also more resource intensive.
    #[pyo3(signature = (infer = false))]
    pub fn inputs(&self, infer: bool) -> Vec<VariableId> {
        self.as_native()
            .inputs(infer)
            .into_iter()
            .map(|it| it.into())
            .collect()
    }

    /// Same as `BooleanNetwork.inputs`, but returns a list of variable names instead.
    #[pyo3(signature = (infer = false))]
    pub fn input_names(&self, infer: bool) -> Vec<String> {
        self.as_native()
            .inputs(infer)
            .into_iter()
            .map(|it| self.as_native().get_variable_name(it).clone())
            .collect()
    }

    /// Return the dictionary of all constants that are present in this `BooleanNetwork`. See also
    /// `BooleanNetwork.is_var_constant`.
    ///
    /// If `infer=True`, the method will use symbolic equivalence check for identifying
    /// constant variables, which is more accurate but also more resource intensive.
    #[pyo3(signature = (infer = false))]
    pub fn constants(&self, infer: bool) -> HashMap<VariableId, bool> {
        self.as_native()
            .constants(infer)
            .into_iter()
            .map(|(a, b)| (a.into(), b))
            .collect()
    }

    /// Same as `BooleanNetwork.constants`, but the keys in the dictionary are variable names,
    /// not IDs.
    #[pyo3(signature = (infer = false))]
    pub fn constant_names(&self, infer: bool) -> HashMap<String, bool> {
        self.as_native()
            .constants(infer)
            .into_iter()
            .map(|(a, b)| (self.as_native().get_variable_name(a).clone(), b))
            .collect()
    }
}

impl BooleanNetwork {
    /// Export a `BooleanNetwork` to something PyO3 will accept because it respects
    /// the class inheritance hierarchy.
    pub fn export_to_python(self, py: Python) -> PyResult<Py<BooleanNetwork>> {
        let graph = self.as_native().as_graph().clone();
        let tuple = (self, RegulatoryGraph::from(graph));
        Py::new(py, tuple)
    }

    /// Try to find a [biodivine_lib_param_bn::ParameterId] that matches the given `parameter` object in this
    /// Boolean network. If the parameter does not exist, returns an `IndexError`. If the `parameter` object is
    /// not [ParameterId] or [String], returns a `TypeError`.
    pub fn resolve_parameter(
        &self,
        parameter: &Bound<'_, PyAny>,
    ) -> PyResult<biodivine_lib_param_bn::ParameterId> {
        if let Ok(id) = parameter.extract::<ParameterId>() {
            return if id.__index__() < self.explicit_parameter_count() {
                Ok(*id.as_native())
            } else {
                throw_index_error(format!("Unknown parameter ID `{}`.", id.__index__()))
            };
        }
        if let Ok(name) = parameter.extract::<String>() {
            return if let Some(var) = self.0.find_parameter(name.as_str()) {
                Ok(var)
            } else {
                throw_index_error(format!("Unknown parameter name `{}`.", name))
            };
        }
        throw_type_error("Expected `ParameterId` or `str`.")
    }
}
