use crate::bindings::lib_bdd::boolean_expression::BooleanExpression;
use crate::bindings::lib_param_bn::argument_types::VariableOrParameterIdType;
use crate::bindings::lib_param_bn::argument_types::bool_type::BoolType;
use crate::bindings::lib_param_bn::argument_types::update_function_type::UpdateFunctionType;
use crate::bindings::lib_param_bn::argument_types::variable_id_type::VariableIdType;
use crate::bindings::lib_param_bn::boolean_network::BooleanNetwork;
use crate::bindings::lib_param_bn::parameter_id::ParameterId;
use crate::bindings::lib_param_bn::variable_id::{VariableId, VariableIdResolvable};
use crate::pyo3_utils::richcmp_eq_by_key;
use crate::{AsNative, runtime_error, throw_runtime_error, throw_type_error};
use biodivine_lib_bdd::boolean_expression::BooleanExpression as RsExpression;
use biodivine_lib_param_bn::{BinaryOp, FnUpdate};
use either::Either;
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// Describes a single update function that is used to describe the dynamics of a `BooleanNetwork`.
///
/// It is similar to a `BooleanExpression`, but additionally admits the use of *uninterpreted functions* (also called
/// explicit parameters in the context of a `BooleanNetwork`). These are Boolean functions with unknown but fixed
/// specification that stand in for any unknown behaviour in the corresponding `BooleanNetwork`.
///
/// Additionally, compared to a `BooleanExpression`, the `UpdateFunction` refers to network variables and parameters
/// using `VariableId` and `ParameterId`. To that end, every `UpdateFunction` has an underlying `BooleanNetwork`
/// which is used to resolve names to IDs and vice versa.
///
#[pyclass(module = "biodivine_aeon", frozen)]
#[derive(Clone)]
pub struct UpdateFunction {
    ctx: Py<BooleanNetwork>,
    root: Arc<FnUpdate>,
    value: &'static FnUpdate,
}

#[pymethods]
impl UpdateFunction {
    /// Build a new `UpdateFunction` in the context of the specified `BooleanNetwork`.
    /// The `value` can be either a string (in which case it is parsed), an `UpdateFunction`, in which case it
    /// is "translated" into the given context (IDs are updated based on matching names), or a `BooleanExpression`,
    /// in which case it also translated using variable names.
    #[new]
    fn new(
        py: Python,
        ctx: Py<BooleanNetwork>,
        value: &Bound<'_, PyAny>,
    ) -> PyResult<UpdateFunction> {
        let fun = if let Ok(value) = value.extract::<String>() {
            FnUpdate::try_from_str(value.as_str(), ctx.borrow(py).as_native())
                .map_err(runtime_error)?
        } else if let Ok(value) = value.extract::<UpdateFunction>() {
            if value.ctx.as_ptr() == ctx.as_ptr() {
                return Ok(value);
            }
            // TODO:
            //  This is a very bad method which tries to avoid translating
            //  IDs from one function to the other. But it is not a great idea, because
            //  the string conversion can be a little finicky, especially if parameters
            //  are involved.
            let value = value.__str__(py);
            FnUpdate::try_from_str(value.as_str(), ctx.borrow(py).as_native())
                .map_err(runtime_error)?
        } else if let Ok(expression) = value.extract::<BooleanExpression>() {
            let Some(fun) = FnUpdate::try_from_expression(
                expression.as_native().clone(),
                ctx.borrow(py).as_native().as_graph(),
            ) else {
                return throw_runtime_error("Expression contains unknown variables.");
            };
            fun
        } else {
            return throw_type_error("Expected `str` or `UpdateFunction`.");
        };
        Ok(Self::new_raw(ctx, Arc::new(fun)))
    }

    fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.value.hash(&mut hasher);
        hasher.finish()
    }

    fn __richcmp__(&self, py: Python, other: &Self, op: CompareOp) -> PyResult<Py<PyAny>> {
        richcmp_eq_by_key(py, op, &self, &other, |it| it.as_native())
    }

    fn __str__(&self, py: Python) -> String {
        self.value.to_string(self.ctx.borrow(py).as_native())
    }

    fn __repr__(&self, py: Python) -> String {
        let ctx = BooleanNetwork::__repr__(self.ctx.borrow(py));
        format!("UpdateFunction({}, {:?})", ctx, self.__str__(py))
    }

    fn __getnewargs__(&self, py: Python) -> (Py<BooleanNetwork>, String) {
        // Technically, this is a "different" expression because it is created with a completely new `root`,
        // but it is much easier (and more transparent) than serializing the root expression and trying to figure
        // out how to serialize a pointer into its AST.
        (self.ctx.clone(), self.__str__(py))
    }

    /// An update function representing the "root" of the expression in which this `UpdateFunction` resides.
    fn __root__(&self) -> UpdateFunction {
        Self::new_raw(self.ctx.clone(), self.root.clone())
    }

    /// A reference to the underlying `BooleanNetwork` that serves as a context for this `UpdateFunction`.
    pub fn __ctx__(&self) -> Py<BooleanNetwork> {
        self.ctx.clone()
    }

    /// Test if a variable or a parameter is used by this `UpdateFunction`.
    fn __contains__(&self, py: Python, item: VariableOrParameterIdType) -> PyResult<bool> {
        let ctx = self.ctx.borrow(py);
        match item.resolve(ctx.as_native()) {
            Ok(Either::Left(var)) => Ok(self.as_native().contains_variable(var)),
            Ok(Either::Right(par)) => Ok(self.as_native().contains_parameter(par)),
            Err(_) => Ok(false),
        }
    }

    /// Return an `UpdateFunction` for a constant value.
    #[staticmethod]
    pub fn mk_const(ctx: Py<BooleanNetwork>, value: BoolType) -> PyResult<UpdateFunction> {
        let fun = if value.bool() {
            FnUpdate::mk_true()
        } else {
            FnUpdate::mk_false()
        };
        Ok(Self::new_raw(ctx, Arc::new(fun)))
    }

    /// Return an `UpdateFunction` of a single variable.
    #[staticmethod]
    pub fn mk_var(
        py: Python,
        ctx: Py<BooleanNetwork>,
        variable: VariableIdType,
    ) -> PyResult<UpdateFunction> {
        let variable = variable.resolve(ctx.borrow(py).as_native())?;
        let fun = FnUpdate::mk_var(variable);
        Ok(Self::new_raw(ctx, Arc::new(fun)))
    }

    #[staticmethod]
    pub fn mk_param(
        py: Python,
        ctx: Py<BooleanNetwork>,
        parameter: &Bound<'_, PyAny>,
        arguments: &Bound<'_, PyList>,
    ) -> PyResult<UpdateFunction> {
        let parameter = ctx.borrow(py).resolve_parameter(parameter)?;
        let mut args = Vec::new();
        for arg in arguments {
            if let Ok(arg_var) = arg.extract::<VariableId>() {
                args.push(FnUpdate::mk_var(*arg_var.as_native()));
            } else if let Ok(arg_str) = arg.extract::<String>() {
                let Some(variable) = ctx
                    .borrow(py)
                    .as_ref()
                    .as_native()
                    .find_variable(arg_str.as_str())
                else {
                    return throw_runtime_error(format!("Unknown variable `{arg_str}`."));
                };
                args.push(FnUpdate::mk_var(variable));
            } else if let Ok(arg_fun) = arg.extract::<UpdateFunction>() {
                args.push(arg_fun.as_native().clone());
            } else {
                return throw_type_error(
                    "Expected `UpdateFunction`, `VariableId`, or variable name.",
                );
            }
        }
        let fun = FnUpdate::mk_param(parameter, &args);
        Ok(Self::new_raw(ctx, Arc::new(fun)))
    }

    /// Return a negation of an `UpdateFunction`.
    #[staticmethod]
    pub fn mk_not(value: &UpdateFunction) -> UpdateFunction {
        let fun = value.as_native().clone().negation();
        Self::new_raw(value.ctx.clone(), Arc::new(fun))
    }

    /// Return an `and` of two `UpdateFunction` values.
    #[staticmethod]
    pub fn mk_and(left: &UpdateFunction, right: &UpdateFunction) -> UpdateFunction {
        let fun = FnUpdate::mk_binary(
            BinaryOp::And,
            left.as_native().clone(),
            right.as_native().clone(),
        );
        Self::new_raw(left.ctx.clone(), Arc::new(fun))
    }

    /// Return an `or` of two `UpdateFunction` values.
    #[staticmethod]
    pub fn mk_or(left: &UpdateFunction, right: &UpdateFunction) -> UpdateFunction {
        let fun = FnUpdate::mk_binary(
            BinaryOp::Or,
            left.as_native().clone(),
            right.as_native().clone(),
        );
        Self::new_raw(left.ctx.clone(), Arc::new(fun))
    }

    /// Return an `imp` of two `BooleanExpression` values.
    #[staticmethod]
    pub fn mk_imp(left: &UpdateFunction, right: &UpdateFunction) -> UpdateFunction {
        let fun = FnUpdate::mk_binary(
            BinaryOp::Imp,
            left.as_native().clone(),
            right.as_native().clone(),
        );
        Self::new_raw(left.ctx.clone(), Arc::new(fun))
    }

    /// Return an `iff` of two `BooleanExpression` values.
    #[staticmethod]
    pub fn mk_iff(left: &UpdateFunction, right: &UpdateFunction) -> UpdateFunction {
        let fun = FnUpdate::mk_binary(
            BinaryOp::Iff,
            left.as_native().clone(),
            right.as_native().clone(),
        );
        Self::new_raw(left.ctx.clone(), Arc::new(fun))
    }

    /// Return a `xor` of two `BooleanExpression` values.
    #[staticmethod]
    pub fn mk_xor(left: &UpdateFunction, right: &UpdateFunction) -> UpdateFunction {
        let fun = FnUpdate::mk_binary(
            BinaryOp::Xor,
            left.as_native().clone(),
            right.as_native().clone(),
        );
        Self::new_raw(left.ctx.clone(), Arc::new(fun))
    }

    /// Construct a binary expression using the given operator (`and`/`or`/`imp`/`iff`/`xor`).
    #[staticmethod]
    pub fn mk_binary(
        op: &str,
        left: &UpdateFunction,
        right: &UpdateFunction,
    ) -> PyResult<UpdateFunction> {
        let op = match op {
            "and" => BinaryOp::And,
            "or" => BinaryOp::Or,
            "imp" => BinaryOp::Imp,
            "iff" => BinaryOp::Iff,
            "xor" => BinaryOp::Xor,
            _ => return throw_type_error("Expected one of `and`/`or`/`imp`/`iff`/`xor`."),
        };
        let fun = FnUpdate::mk_binary(op, left.as_native().clone(), right.as_native().clone());
        Ok(Self::new_raw(left.ctx.clone(), Arc::new(fun)))
    }

    /// Build a conjunction of multiple expressions.
    #[staticmethod]
    pub fn mk_conjunction(
        py: Python,
        ctx: Py<BooleanNetwork>,
        args: &Bound<'_, PyList>,
    ) -> PyResult<UpdateFunction> {
        let mut native_args = Vec::new();
        for arg in args.iter() {
            native_args.push(
                UpdateFunction::new(py, ctx.clone(), &arg)?
                    .as_native()
                    .clone(),
            );
        }
        let fun = FnUpdate::mk_conjunction(&native_args);
        Ok(Self::new_raw(ctx.clone(), Arc::new(fun)))
    }

    /// Build a disjunction of multiple expressions.
    #[staticmethod]
    pub fn mk_disjunction(
        py: Python,
        ctx: Py<BooleanNetwork>,
        args: &Bound<'_, PyList>,
    ) -> PyResult<UpdateFunction> {
        let mut native_args = Vec::new();
        for arg in args.iter() {
            native_args.push(
                UpdateFunction::new(py, ctx.clone(), &arg)?
                    .as_native()
                    .clone(),
            );
        }
        let fun = FnUpdate::mk_disjunction(&native_args);
        Ok(Self::new_raw(ctx.clone(), Arc::new(fun)))
    }

    /// Return true if the root of this expression is a constant.
    pub fn is_const(&self) -> bool {
        matches!(self.as_native(), &FnUpdate::Const(_))
    }

    /// Return true if the root of this expression is a variable.
    pub fn is_var(&self) -> bool {
        matches!(self.as_native(), &FnUpdate::Var(_))
    }

    /// Return true if the root of this expression is a call to an uninterpreted function.
    pub fn is_param(&self) -> bool {
        matches!(self.as_native(), &FnUpdate::Param(_, _))
    }

    /// Return true if the root of this expression is a `not`.
    pub fn is_not(&self) -> bool {
        matches!(self.as_native(), &FnUpdate::Not(_))
    }

    /// Return true if the root of this expression is an `and`.
    pub fn is_and(&self) -> bool {
        matches!(self.as_native(), &FnUpdate::Binary(BinaryOp::And, _, _))
    }

    /// Return true if the root of this expression is an `or`.
    pub fn is_or(&self) -> bool {
        matches!(self.as_native(), &FnUpdate::Binary(BinaryOp::Or, _, _))
    }

    /// Return true if the root of this expression is an `imp`.
    pub fn is_imp(&self) -> bool {
        matches!(self.as_native(), &FnUpdate::Binary(BinaryOp::Imp, _, _))
    }

    /// Return true if the root of this expression is an `iff`.
    pub fn is_iff(&self) -> bool {
        matches!(self.as_native(), &FnUpdate::Binary(BinaryOp::Iff, _, _))
    }

    /// Return true if the root of this expression is a `xor`.
    pub fn is_xor(&self) -> bool {
        matches!(self.as_native(), &FnUpdate::Binary(BinaryOp::Xor, _, _))
    }

    /// Return true if the root of this expression is a literal (`var`/`!var`).
    pub fn is_literal(&self) -> bool {
        match self.as_native() {
            FnUpdate::Var(_) => true,
            FnUpdate::Not(inner) => {
                matches!(**inner, FnUpdate::Var(_))
            }
            _ => false,
        }
    }

    /// Return true if the root of this expression is a binary operator (`and`/`or`/`imp`/`iff`/`xor`).
    pub fn is_binary(&self) -> bool {
        matches!(self.as_native(), &FnUpdate::Binary(_, _, _))
    }

    /// If the root of this expression is a constant, return its value, or `None` otherwise.
    pub fn as_const(&self) -> Option<bool> {
        match self.as_native() {
            FnUpdate::Const(x) => Some(*x),
            _ => None,
        }
    }

    /// If the root of this expression is a `var`, return its ID, or `None` otherwise.
    pub fn as_var(&self) -> Option<VariableId> {
        match self.as_native() {
            FnUpdate::Var(x) => Some(VariableId::from(*x)),
            _ => None,
        }
    }

    /// If the root of this expression is a call to an uninterpreted function, return its ID and arguments, or
    /// `None` otherwise.
    pub fn as_param(&self) -> Option<(ParameterId, Vec<UpdateFunction>)> {
        match self.as_native() {
            FnUpdate::Param(id, args) => {
                let id = ParameterId::from(*id);
                let args = args
                    .iter()
                    .map(|arg| self.mk_child_ref(arg))
                    .collect::<Vec<_>>();
                Some((id, args))
            }
            _ => None,
        }
    }

    /// If the root of this expression is a `not`, return its operand, or `None` otherwise.
    pub fn as_not(&self) -> Option<UpdateFunction> {
        match self.as_native() {
            FnUpdate::Not(x) => Some(self.mk_child_ref(x)),
            _ => None,
        }
    }

    /// If the root of this expression is an `and`, return its two operands, or `None` otherwise.
    pub fn as_and(&self) -> Option<(UpdateFunction, UpdateFunction)> {
        match self.as_native() {
            FnUpdate::Binary(BinaryOp::And, l, r) => {
                Some((self.mk_child_ref(l), self.mk_child_ref(r)))
            }
            _ => None,
        }
    }

    /// If the root of this expression is an `or`, return its two operands, or `None` otherwise.
    pub fn as_or(&self) -> Option<(UpdateFunction, UpdateFunction)> {
        match self.as_native() {
            FnUpdate::Binary(BinaryOp::Or, l, r) => {
                Some((self.mk_child_ref(l), self.mk_child_ref(r)))
            }
            _ => None,
        }
    }

    /// If the root of this expression is an `imp`, return its two operands, or `None` otherwise.
    pub fn as_imp(&self) -> Option<(UpdateFunction, UpdateFunction)> {
        match self.as_native() {
            FnUpdate::Binary(BinaryOp::Imp, l, r) => {
                Some((self.mk_child_ref(l), self.mk_child_ref(r)))
            }
            _ => None,
        }
    }

    /// If the root of this expression is an `iff`, return its two operands, or `None` otherwise.
    pub fn as_iff(&self) -> Option<(UpdateFunction, UpdateFunction)> {
        match self.as_native() {
            FnUpdate::Binary(BinaryOp::Iff, l, r) => {
                Some((self.mk_child_ref(l), self.mk_child_ref(r)))
            }
            _ => None,
        }
    }

    /// If the root of this expression is `xor`, return its two operands, or `None` otherwise.
    pub fn as_xor(&self) -> Option<(UpdateFunction, UpdateFunction)> {
        match self.as_native() {
            FnUpdate::Binary(BinaryOp::Xor, l, r) => {
                Some((self.mk_child_ref(l), self.mk_child_ref(r)))
            }
            _ => None,
        }
    }

    /// If this expression is either `var` or `!var`, return the ID of the variable and whether it is positive.
    /// Otherwise, return `None`.
    pub fn as_literal(&self) -> Option<(VariableId, bool)> {
        match self.as_native() {
            FnUpdate::Var(id) => Some((VariableId::from(*id), true)),
            FnUpdate::Not(inner) => match inner.as_ref() {
                FnUpdate::Var(id) => Some((VariableId::from(*id), false)),
                _ => None,
            },
            _ => None,
        }
    }

    /// If the root of this expression is one of the `and`/`or`/`imp`/`iff`/`xor` operators, return the name of the
    /// operator and its two operands. Returns `None` if the root is not a binary operator.
    pub fn as_binary(&self) -> Option<(String, UpdateFunction, UpdateFunction)> {
        match self.as_native() {
            FnUpdate::Binary(op, l, r) => {
                let op = match op {
                    BinaryOp::And => "and",
                    BinaryOp::Or => "or",
                    BinaryOp::Imp => "imp",
                    BinaryOp::Iff => "iff",
                    BinaryOp::Xor => "xor",
                };
                Some((op.to_string(), self.mk_child_ref(l), self.mk_child_ref(r)))
            }
            _ => None,
        }
    }

    /// Return the set of variable IDs that are used in this `UpdateFunction`.
    pub fn support_variables(&self) -> HashSet<VariableId> {
        self.value
            .collect_arguments()
            .into_iter()
            .map(VariableId::from)
            .collect()
    }

    /// Return the set of parameter IDs that are used in this `UpdateFunction`.
    pub fn support_parameters(&self) -> HashSet<ParameterId> {
        self.value
            .collect_parameters()
            .into_iter()
            .map(ParameterId::from)
            .collect()
    }

    /// Create a copy of this `UpdateFunction` with every occurrence of the specified variables substituted
    /// for the corresponding function.
    ///
    ///  > Note that at the moment, there is no substitution method for parameters, since we don't have a concept
    ///    of an "expression with holes" which we could use here.
    pub fn substitute_all(
        &self,
        py: Python,
        substitution: HashMap<VariableIdType, UpdateFunctionType>,
    ) -> PyResult<UpdateFunction> {
        let mut vars = HashMap::new();
        let bn = self.ctx.borrow(py);
        for (k, v) in substitution {
            let k = k.resolve(bn.as_native())?;
            let v = v.resolve(bn.as_native())?;
            vars.insert(k, v);
        }

        fn rec(
            _self: &FnUpdate,
            vars: &HashMap<biodivine_lib_param_bn::VariableId, FnUpdate>,
        ) -> FnUpdate {
            match _self {
                FnUpdate::Const(value) => FnUpdate::Const(*value),
                FnUpdate::Var(var) => vars.get(var).cloned().unwrap_or(FnUpdate::Var(*var)),
                FnUpdate::Param(param, args) => {
                    let args = args.iter().map(|it| rec(it, vars)).collect::<Vec<_>>();
                    FnUpdate::Param(*param, args)
                }
                FnUpdate::Not(inner) => FnUpdate::mk_not(rec(inner, vars)),
                FnUpdate::Binary(op, left, right) => {
                    FnUpdate::mk_binary(*op, rec(left, vars), rec(right, vars))
                }
            }
        }

        let subst = rec(self.value, &vars);
        Ok(UpdateFunction::new_raw(self.ctx.clone(), Arc::new(subst)))
    }

    /// Rename all occurrences of the specified `variables` and `parameters` to new IDs.
    ///
    /// The `variables` and `parameters` dictionaries map the old variables (using `VariableId` or a string name)
    /// to new variables (again, using IDs or names). If the new names or IDs are not valid in the currently
    /// referenced `BooleanNetwork`, you can supply a `new_ctx`. If `new_ctx` is given, dictionary *keys* are resolved
    /// in the current context, but dictionary *values* are resolved in the newly provided context.
    ///
    #[pyo3(signature=(new_ctx = None, variables = None, parameters = None))]
    pub fn rename_all(
        &self,
        py: Python,
        new_ctx: Option<Py<BooleanNetwork>>,
        variables: Option<HashMap<VariableIdType, VariableIdType>>,
        parameters: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<UpdateFunction> {
        let mut rename_variables = HashMap::new();
        let mut rename_parameters = HashMap::new();

        let old_ctx = self.ctx.borrow(py);
        if let Some(new_ctx) = new_ctx.as_ref() {
            let new_ctx = new_ctx.borrow(py);
            if let Some(variables) = variables {
                for (k, v) in variables {
                    let k = k.resolve(old_ctx.as_native())?;
                    let v = v.resolve(new_ctx.as_native())?;
                    rename_variables.insert(k, v);
                }
            }
            if let Some(parameters) = parameters {
                for (k, v) in parameters {
                    let k = old_ctx.resolve_parameter(&k)?;
                    let v = new_ctx.resolve_parameter(&v)?;
                    rename_parameters.insert(k, v);
                }
            }
        } else {
            let new_ctx = self.ctx.borrow(py);
            if let Some(variables) = variables {
                for (k, v) in variables {
                    let k = k.resolve(old_ctx.as_native())?;
                    let v = v.resolve(new_ctx.as_native())?;
                    rename_variables.insert(k, v);
                }
            }
            if let Some(parameters) = parameters {
                for (k, v) in parameters {
                    let k = old_ctx.resolve_parameter(&k)?;
                    let v = new_ctx.resolve_parameter(&v)?;
                    rename_parameters.insert(k, v);
                }
            }
        }
        let fun = self.value.rename_all(&rename_variables, &rename_parameters);
        if let Some(new_ctx) = new_ctx {
            Ok(UpdateFunction::new_raw(new_ctx, Arc::new(fun)))
        } else {
            Ok(UpdateFunction::new_raw(self.ctx.clone(), Arc::new(fun)))
        }
    }

    /// Creates a copy of this `UpdateFunction` with all constant values propagated into the remaining
    /// expression. Consequently, the result contains constant values only if (a) the whole function is a constant,
    /// or (b) a constant value is used as an argument of an uninterpreted function (parameter).
    pub fn simplify_constants(&self) -> UpdateFunction {
        let transformed = self.value.simplify_constants();
        UpdateFunction::new_raw(self.ctx.clone(), Arc::new(transformed))
    }

    /// Creates a copy of this `UpdateFunction` where the negation operation is distributed to the propositions
    /// (constants, network variables, and function invocations).
    ///
    /// The operation is also applied to each expression that is an argument of an uninterpreted function. However,
    /// we cannot distribute the negation of an uninterpreted function to its arguments without further knowledge
    /// about its behaviour.
    pub fn distribute_negation(&self) -> UpdateFunction {
        let transformed = self.value.distribute_negation();
        UpdateFunction::new_raw(self.ctx.clone(), Arc::new(transformed))
    }

    /// Creates a copy of this `UpdateFunction` where all binary operators are expanded into their `and`/`or`
    /// equivalents.
    ///
    /// Note that in extreme cases, the result can be an exponentially larger formula compared to the original.
    pub fn to_and_or_normal_form(&self) -> UpdateFunction {
        let transformed = self.value.to_and_or_normal_form();
        UpdateFunction::new_raw(self.ctx.clone(), Arc::new(transformed))
    }

    /// Convert the `UpdateFunction` to a `BooleanExpression`, as long as the function contains no uninterpreted
    /// functions (otherwise throws a `RuntimeError`).
    pub fn as_expression(&self, py: Python) -> PyResult<BooleanExpression> {
        fn rec(
            self_: &FnUpdate,
            ctx: &biodivine_lib_param_bn::BooleanNetwork,
        ) -> PyResult<RsExpression> {
            match self_ {
                FnUpdate::Const(value) => Ok(RsExpression::Const(*value)),
                FnUpdate::Var(id) => Ok(RsExpression::Variable(ctx.get_variable_name(*id).clone())),
                FnUpdate::Param(_, _) => throw_runtime_error("Function contains parameters."),
                FnUpdate::Not(inner) => Ok(RsExpression::Not(Box::new(rec(inner, ctx)?))),
                FnUpdate::Binary(op, left, right) => {
                    let left = rec(left, ctx)?;
                    let right = rec(right, ctx)?;
                    Ok(match *op {
                        BinaryOp::And => RsExpression::And(Box::new(left), Box::new(right)),
                        BinaryOp::Or => RsExpression::Or(Box::new(left), Box::new(right)),
                        BinaryOp::Xor => RsExpression::Xor(Box::new(left), Box::new(right)),
                        BinaryOp::Iff => RsExpression::Iff(Box::new(left), Box::new(right)),
                        BinaryOp::Imp => RsExpression::Imp(Box::new(left), Box::new(right)),
                    })
                }
            }
        }

        let result = rec(self.value, self.ctx.borrow(py).as_native())?;
        Ok(BooleanExpression::new_raw(Arc::new(result)))
    }
}

impl UpdateFunction {
    pub fn new_raw(ctx: Py<BooleanNetwork>, root: Arc<FnUpdate>) -> UpdateFunction {
        let value: &'static FnUpdate =
            unsafe { (root.as_ref() as *const FnUpdate).as_ref().unwrap() };
        UpdateFunction { ctx, root, value }
    }

    pub fn as_native(&self) -> &FnUpdate {
        self.value
    }

    pub fn mk_child_ref(&self, child: &FnUpdate) -> UpdateFunction {
        let value: &'static FnUpdate = unsafe { (child as *const FnUpdate).as_ref().unwrap() };
        UpdateFunction {
            ctx: self.ctx.clone(),
            root: self.root.clone(),
            value,
        }
    }
}
