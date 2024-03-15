use crate::bindings::lib_bdd::boolean_expression::BooleanExpression;
use crate::bindings::lib_param_bn::boolean_network::BooleanNetwork;
use crate::bindings::lib_param_bn::parameter_id::ParameterId;
use crate::bindings::lib_param_bn::symbolic::set_color::ColorSet;
use crate::bindings::lib_param_bn::symbolic::symbolic_context::SymbolicContext;
use crate::bindings::lib_param_bn::update_function::UpdateFunction;
use crate::bindings::lib_param_bn::variable_id::VariableId;
use crate::{throw_index_error, throw_type_error, AsNative};
use biodivine_lib_bdd::BddPartialValuation;
use biodivine_lib_param_bn::FnUpdate;
use either::Either;
use pyo3::types::{PyDict, PyList, PyTuple};
use pyo3::{pyclass, pymethods, IntoPy, Py, PyAny, PyObject, PyResult, Python};
use std::sync::Arc;
use Either::{Left, Right};

/// Represents a single "color" stored in a `ColorSet` (or a `ColoredVertexSet`), or a projection
/// of said color to the chosen uninterpreted functions.
///
/// Behaves like an immutable dictionary: Uninterpreted functions can be queried using
/// a `VariableId`/`ParameterId` (implicit/explicit parameters), a string name, or
/// a `BddVariable` from the function table. The result is a `BooleanExpression`.
///
/// However, note that each function instantiation is by default represented as a
/// `BooleanExpression` using anonymous variable names `x_0 ... x_k` (where `k` is the arity
/// of the uninterpreted function). If you actually want to instantiate the function w.r.t.
/// a set of arguments, specific `UpdateFunction`, or a parametrized `BooleanNetwork`,
/// you can use the `ColorModel.instantiate` method.
#[pyclass(module = "biodivine_aeon", frozen)]
#[derive(Clone)]
pub struct ColorModel {
    ctx: Py<SymbolicContext>,
    native: BddPartialValuation,
    retained_implicit: Vec<biodivine_lib_param_bn::VariableId>,
    retained_explicit: Vec<biodivine_lib_param_bn::ParameterId>,
}

#[pymethods]
impl ColorModel {
    /// Access the underlying `SymbolicContext` connected to this `VertexModel`.
    pub fn __ctx__(&self) -> Py<SymbolicContext> {
        self.ctx.clone()
    }

    pub fn __str__(&self) -> PyResult<String> {
        let ctx = self.ctx.get();
        let mut items = Vec::new();
        for par in &self.retained_explicit {
            let expr = self.instantiate_expression(Right(*par))?;
            items.push(format!(
                "'{}': '{}'",
                ctx.as_native().get_network_parameter_name(*par),
                expr.__str__(),
            ));
        }
        for var in &self.retained_implicit {
            let expr = self.instantiate_expression(Left(*var))?;
            items.push(format!(
                "'{}': '{}'",
                ctx.as_native().get_network_variable_name(*var),
                expr.__str__(),
            ));
        }
        Ok(format!("ColorModel({{{}}})", items.join(", ")))
    }

    /// The number of actual uninterpreted functions in this `ColorModel`.
    pub fn __len__(&self) -> usize {
        self.retained_implicit.len() + self.retained_explicit.len()
    }

    pub fn __getitem__(&self, key: &PyAny) -> PyResult<BooleanExpression> {
        let ctx = self.ctx.get();
        match ctx.resolve_function(key)? {
            Left(variable) => self.instantiate_expression(Left(variable)),
            Right(parameter) => self.instantiate_expression(Right(parameter)),
        }
    }

    pub fn __contains__(&self, key: &PyAny) -> PyResult<bool> {
        let ctx = self.ctx.get();
        match ctx.resolve_function(key)? {
            Left(variable) => Ok(self.retained_implicit.contains(&variable)),
            Right(parameter) => Ok(self.retained_explicit.contains(&parameter)),
        }
    }

    /// The actual "retained" uninterpreted functions in this `ColorModel`.
    ///
    /// This is the list of all `ParameterId` and `VariableId` objects that admit an associated
    /// uninterpreted function and said function is present in this `ColorModel`.
    pub fn keys<'a>(&self, py: Python<'a>) -> PyResult<&'a PyList> {
        let result = PyList::empty(py);
        for x in &self.retained_explicit {
            result.append(ParameterId::from(*x).into_py(py))?;
        }
        for x in &self.retained_implicit {
            result.append(VariableId::from(*x).into_py(py))?;
        }
        Ok(result)
    }

    /// The list of `BooleanExpression` objects representing the individual uninterpreted functions
    /// from `ColorModel.keys`.
    pub fn values(&self) -> PyResult<Vec<BooleanExpression>> {
        let mut result = Vec::new();
        for x in &self.retained_explicit {
            result.push(self.instantiate_expression(Right(*x))?);
        }
        for x in &self.retained_implicit {
            result.push(self.instantiate_expression(Left(*x))?);
        }
        Ok(result)
    }

    /// The list of key-value pairs represented in this model.
    pub fn items<'a>(&self, py: Python<'a>) -> PyResult<&'a PyList> {
        let result = PyList::empty(py);
        for x in &self.retained_explicit {
            let k = ParameterId::from(*x).into_py(py);
            let v = self.instantiate_expression(Right(*x))?.into_py(py);
            result.append(PyTuple::new(py, [k, v]))?;
        }
        for x in &self.retained_implicit {
            let k = VariableId::from(*x).into_py(py);
            let v = self.instantiate_expression(Left(*x))?.into_py(py);
            result.append(PyTuple::new(py, [k, v]))?;
        }
        Ok(result)
    }

    /// The same as `VertexModel.items`, but returns a dictionary instead.
    pub fn to_dict<'a>(&self, py: Python<'a>) -> PyResult<&'a PyDict> {
        let result = PyDict::new(py);
        for x in &self.retained_explicit {
            let k = ParameterId::from(*x).into_py(py);
            let v = self.instantiate_expression(Right(*x))?.into_py(py);
            result.set_item(k, v)?;
        }
        for x in &self.retained_implicit {
            let k = VariableId::from(*x).into_py(py);
            let v = self.instantiate_expression(Left(*x))?.into_py(py);
            result.set_item(k, v)?;
        }
        Ok(result)
    }

    /// Return the underlying `BddPartialValuation` for this symbolic model.
    pub fn to_valuation(&self) -> crate::bindings::lib_bdd::bdd_valuation::BddPartialValuation {
        crate::bindings::lib_bdd::bdd_valuation::BddPartialValuation::new_raw(
            self.ctx.get().bdd_variable_set(),
            self.native.clone(),
        )
    }

    /// Return a `ColorSet` where all the implicit and explicit parameters are fixed according
    /// to the values in this `ColorModel`. Parameters that are not present in the `ColorModel`
    /// are unrestricted.
    ///
    /// Note that this does not apply any constraints that may be relevant in the
    /// `AsynchronousGraph` that was used to create this model.
    pub fn to_symbolic(&self) -> ColorSet {
        let ctx = self.ctx.get();
        let bdd = ctx
            .as_native()
            .bdd_variable_set()
            .mk_conjunctive_clause(&self.native);
        let native =
            biodivine_lib_param_bn::symbolic_async_graph::GraphColors::new(bdd, ctx.as_native());
        ColorSet::mk_native(self.ctx.clone(), native)
    }

    /// The `ColorModel.instantiate` method is used to "fill in" the actual implementation of
    /// the uninterpreted functions defined in this `ColorModel` into an object that depends on
    /// this implementation.
    ///
    /// Specifically, there are three supported modes of operation:
    ///  - If `item` is an `UpdateFunction`, the result is a new `UpdateFunction` that only depends
    ///    on network variables and is the interpretation of the original function under this model.
    ///  - If `item` is a `BooleanNetwork`, the result is a new `BooleanNetwork` with no parameters
    ///    where each function has been instantiated.
    ///  - If `item` identifies an uninterpreted function (by `ParameterId`, `VariableId`, or
    ///    a string name), the method returns an `UpdateFunction` that is an interpretation of the
    ///    uninterpreted function with specified `args` under this model. This is equivalent to
    ///    computing `SymbolicContext.mk_function` and then instantiating this function. Note that
    ///    in this situation, the `args` argument is required, and it must match the arity of the
    ///    uninterpreted function.
    ///
    /// *Note that in some cases, instantiating an `UpdateFunction` with two different
    /// interpretations can lead to the same `UpdateFunction`. This happens if parts of the
    /// function are redundant. For example, `f(x) | !f(x)` always instantiates to `true`,
    /// regardless of the interpretation of `f`. Hence, you can assume that while interpretations
    /// (i.e. `model["f"]`) are unique within a set, the instantiations of more complex functions
    /// that depend on them are not.*
    #[pyo3(signature = (item, args = None))]
    pub fn instantiate(
        &self,
        py: Python,
        item: &PyAny,
        args: Option<Vec<&PyAny>>,
    ) -> PyResult<PyObject> {
        let ctx = self.ctx.get();
        if let Ok(update_function) = item.extract::<UpdateFunction>() {
            // For an update function, we just return a version of the same function with all
            // explicit parameters instantiated.
            if args.is_some() {
                return throw_type_error(
                    "Argument `args` not expected when `item` is an `UpdateFunction`.",
                );
            }
            return self.instantiate_update_function(py, update_function);
        }
        if let Ok(network) = item.extract::<Py<BooleanNetwork>>() {
            // For a Boolean network, we try to instantiate every update function separated
            // and then remove all unused parameters.
            if args.is_some() {
                return throw_type_error(
                    "Argument `args` not expected when `item` is a `BooleanNetwork`.",
                );
            }
            let mut bn = network.borrow(py).as_native().clone();
            for var in bn.variables() {
                let function = if let Some(function) = bn.get_update_function(var) {
                    self.instantiate_fn_update(function)?
                } else {
                    let args = bn.regulators(var);
                    let function_bdd = ctx.as_native().mk_implicit_function_is_true(var, &args);
                    let instantiated_bdd = function_bdd.restrict(&self.to_values());
                    FnUpdate::build_from_bdd(ctx.as_native(), &instantiated_bdd)
                };
                bn.set_update_function(var, Some(function)).unwrap();
            }

            let bn = bn.prune_unused_parameters();
            assert_eq!(bn.num_parameters(), 0);

            return Ok(BooleanNetwork::from(bn).export_to_python(py)?.into_py(py));
        }
        if let Ok(function) = ctx.resolve_function(item) {
            let Some(args) = args else {
                return throw_type_error(
                    "Argument `args` is mandatory when `item` is a function identifier.",
                );
            };
            let arguments = args
                .into_iter()
                .map(|it| ctx.resolve_function_bdd(it))
                .collect::<PyResult<Vec<_>>>()?;
            let table = match function {
                Left(var) => ctx.as_native().get_implicit_function_table(var).unwrap(),
                Right(par) => ctx.as_native().get_explicit_function_table(par),
            };
            let function_bdd = ctx.as_native().mk_function_table_true(table, &arguments);
            let instantiated_bdd = function_bdd.restrict(&self.to_values());
            let instantiated_function =
                FnUpdate::build_from_bdd(ctx.as_native(), &instantiated_bdd);
            let fake_ctx = ctx.mk_fake_network();
            let fake_ctx = BooleanNetwork::from(fake_ctx).export_to_python(py)?;
            return Ok(
                UpdateFunction::new_raw(fake_ctx, Arc::new(instantiated_function)).into_py(py),
            );
        }
        throw_type_error("Expected `UpdateFunction`, `BooleanNetwork`, or a valid function identifier (`VariableId`, `ParameterId`, or a string name) with an `args` collection.")
    }
}

impl ColorModel {
    fn to_values(&self) -> Vec<(biodivine_lib_bdd::BddVariable, bool)> {
        // Only return state variables:
        let mut result = Vec::new();
        for var in self.ctx.get().as_native().parameter_variables() {
            if let Some(value) = self.native.get_value(*var) {
                result.push((*var, value))
            }
        }
        result
    }

    pub fn new_native(
        ctx: Py<SymbolicContext>,
        native: BddPartialValuation,
        retained_implicit: Vec<biodivine_lib_param_bn::VariableId>,
        retained_explicit: Vec<biodivine_lib_param_bn::ParameterId>,
    ) -> ColorModel {
        ColorModel {
            ctx,
            native,
            retained_implicit,
            retained_explicit,
        }
    }

    pub fn instantiate_update_function(
        &self,
        py: Python,
        update_function: UpdateFunction,
    ) -> PyResult<PyObject> {
        let instantiated_function = self.instantiate_fn_update(update_function.as_native())?;
        let update =
            UpdateFunction::new_raw(update_function.__ctx__(), Arc::new(instantiated_function));
        Ok(update.into_py(py))
    }

    pub fn instantiate_fn_update(&self, fn_update: &FnUpdate) -> PyResult<FnUpdate> {
        let ctx = self.ctx.get().as_native();

        let mut missing_support = fn_update.collect_parameters();
        missing_support.retain(|x| !self.retained_explicit.contains(x));
        if !missing_support.is_empty() {
            return throw_index_error(format!(
                "Function(s) `{:?}` are not available in this projection.",
                missing_support
            ));
        }

        let update_function_bdd = ctx.mk_fn_update_true(fn_update);
        let instantiated_bdd = update_function_bdd.restrict(&self.to_values());
        Ok(FnUpdate::build_from_bdd(ctx, &instantiated_bdd))
    }

    /// Turn a function into a `BooleanExpression` using anonymous variable names.
    pub fn instantiate_expression(
        &self,
        function: Either<biodivine_lib_param_bn::VariableId, biodivine_lib_param_bn::ParameterId>,
    ) -> PyResult<BooleanExpression> {
        let ctx = self.ctx.get();
        let table = match &function {
            Left(var) => ctx.as_native().get_implicit_function_table(*var).unwrap(),
            Right(par) => ctx.as_native().get_explicit_function_table(*par),
        };

        // This is loosely based on `mk_function_table_true`, but uses the `mk_dnf` method.
        let mut dnf = Vec::new();
        let custom_ctx = biodivine_lib_bdd::BddVariableSet::new_anonymous(table.arity);
        for (input_row, output) in table {
            if let Some(value) = self.native.get_value(output) {
                if value {
                    let valuation = biodivine_lib_bdd::BddValuation::new(input_row);
                    let valuation = biodivine_lib_bdd::BddPartialValuation::from(valuation);
                    dnf.push(valuation);
                }
            } else {
                let name = match function {
                    Left(var) => ctx.as_native().get_network_variable_name(var),
                    Right(par) => ctx.as_native().get_network_parameter_name(par),
                };
                return throw_index_error(format!(
                    "Function `{}` is not available in this projection.",
                    name
                ));
            }
        }
        let custom_bdd = custom_ctx.mk_dnf(&dnf);
        let custom_dnf = custom_bdd.to_dnf();
        let custom_vars = custom_ctx
            .variables()
            .into_iter()
            .map(|it| {
                let name = custom_ctx.name_of(it);
                BooleanExpression::mk_var(name)
            })
            .collect::<Vec<_>>();
        let clauses = custom_dnf
            .into_iter()
            .map(|clause| {
                let args = clause
                    .to_values()
                    .into_iter()
                    .map(|(var, value)| {
                        if value {
                            custom_vars[var.to_index()].clone()
                        } else {
                            BooleanExpression::mk_not(&custom_vars[var.to_index()])
                        }
                    })
                    .collect::<Vec<_>>();
                BooleanExpression::mk_conjunction(args)
            })
            .collect::<Vec<_>>();
        Ok(BooleanExpression::mk_disjunction(clauses))
    }
}
