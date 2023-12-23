use crate::bindings::lib_bdd_2::bdd::Bdd;
use crate::bindings::lib_bdd_2::bdd_valuation::{BddPartialValuation, BddValuation};
use crate::bindings::lib_bdd_2::bdd_variable::BddVariable;
use crate::bindings::lib_bdd_2::boolean_expression::BooleanExpression;
use crate::pyo3_utils::{resolve_boolean, richcmp_eq_inner};
use crate::{throw_index_error, throw_runtime_error, throw_type_error, AsNative};
use macros::Wrapper;
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};

/// Represents a collection of named `BddVariable` identifiers and is primarily used to create
/// "atomic" `Bdd` objects (constants, literals, etc.).
///
/// Note that `Bdd` objects created by different variable sets are inherently incompatible.
/// However, in many reasonable cases, you can convert between them using
/// `BddVariableSet.transfer_from`.
///
/// ```python
/// ctx = BddVariableSet(["a", "b", "c"])
/// assert str(ctx) == 'BddVariableSet(len = 3)'
/// assert ctx == eval(repr(ctx))
/// assert len(ctx) == 3
/// assert ctx.variable_count() == 3
/// assert ctx.variable_names() == ["a", "b", "c"]
/// assert ctx.variable_ids() == [BddVariable(i) for i in [0,1,2]]
///
/// var_b = ctx.find_variable("b")
/// assert var_b is not None
/// assert ctx.find_variable("x") is None
/// assert ctx.find_variable(BddVariable(10)) is None
/// assert ctx.get_variable_name(var_b) == "b"
/// assert ctx.get_variable_name("x") is None
///
/// ctx2 = BddVariableSet(["a", "c"])
/// not_c_1 = ctx.mk_literal("c", False)
/// not_c_2 = ctx2.transfer_from(not_c_1, ctx)
/// assert not_c_2 == ctx2.mk_literal("c", 0)
/// ```
///
/// See also `BddVariableSetBuilder`.
#[pyclass(module = "biodivine_aeon", frozen)]
#[derive(Clone, Wrapper)]
pub struct BddVariableSet(biodivine_lib_bdd::BddVariableSet);

#[pymethods]
impl BddVariableSet {
    #[new]
    fn new(variables: &PyAny) -> PyResult<BddVariableSet> {
        if let Ok(var_count) = variables.extract::<usize>() {
            let Ok(var_count) = u16::try_from(var_count) else {
                return throw_runtime_error("`BddVariableSet` only supports up to 65k variables.");
            };
            return Ok(BddVariableSet(
                biodivine_lib_bdd::BddVariableSet::new_anonymous(var_count),
            ));
        }
        if let Ok(variables) = variables.extract::<Vec<&str>>() {
            return Ok(BddVariableSet(biodivine_lib_bdd::BddVariableSet::new(
                &variables,
            )));
        }
        throw_type_error("Expected `int` or `list[str]`.")
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
        richcmp_eq_inner(op, &self, &other, |x| x.variable_names())
    }

    fn __len__(&self) -> usize {
        usize::from(self.0.num_vars())
    }

    fn __str__(&self) -> String {
        format!("BddVariableSet(len = {})", self.0.num_vars())
    }

    pub fn __repr__(&self) -> String {
        let names = self.variable_names();
        format!("BddVariableSet({:?})", names)
    }

    fn __getnewargs__<'a>(&self, py: Python<'a>) -> &'a PyTuple {
        PyTuple::new(py, &[self.variable_names()])
    }

    /// Return the number of variables managed by this `BddVariableSet`.
    pub fn variable_count(&self) -> usize {
        usize::from(self.0.num_vars())
    }

    /// Return the list of all `BddVariable` identifiers managed by this `BddVariableSet`.
    pub fn variable_ids(&self) -> Vec<BddVariable> {
        self.0.variables().into_iter().map(|it| it.into()).collect()
    }

    /// Return the list of all names for all variables managed by this `BddVariableSet`.
    ///
    /// The ordering should match the standard ordering of `BddVariable` identifiers.
    pub fn variable_names(&self) -> Vec<String> {
        self.0
            .variables()
            .into_iter()
            .map(|it| self.0.name_of(it))
            .collect()
    }

    /// Return the `BddVariable` identifier of the requested `variable`, or `None` if the
    /// variable does not exist in this `BddVariableSet`.
    fn find_variable(&self, variable: &PyAny) -> PyResult<Option<BddVariable>> {
        if let Ok(id) = variable.extract::<BddVariable>() {
            return if id.__index__() < self.__len__() {
                Ok(Some(id))
            } else {
                Ok(None)
            };
        }
        if let Ok(name) = variable.extract::<String>() {
            return Ok(self.0.var_by_name(name.as_str()).map(Into::into));
        }
        throw_type_error("Expected `BddVariable` or `str`.")
    }

    /// Return the string name of the requested `variable`, or throw `RuntimeError` if
    /// such variable does not exist.
    pub fn get_variable_name(&self, variable: &PyAny) -> PyResult<String> {
        let var = self.resolve_variable(variable)?;
        Ok(self.0.name_of(var))
    }

    /// Create a new `Bdd` representing the Boolean function $\mathit{false}$.
    fn mk_false(self_: PyRef<'_, Self>) -> Bdd {
        let value = self_.0.mk_false();
        Bdd::new(self_, value)
    }

    /// Create a new `Bdd` representing the Boolean function $\mathit{true}$.
    fn mk_true(self_: PyRef<'_, Self>) -> Bdd {
        let value = self_.0.mk_true();
        Bdd::new(self_, value)
    }

    /// Create a new `Bdd` representing the constant Boolean function given by `value`.
    fn mk_const(self_: PyRef<'_, Self>, value: &PyAny) -> PyResult<Bdd> {
        let value = resolve_boolean(value)?;
        let value = if value {
            self_.0.mk_true()
        } else {
            self_.0.mk_false()
        };
        Ok(Bdd::new(self_, value))
    }

    /// Create a new `Bdd` representing the literal $variable$ or $\neg variable$, depending
    /// on the given `value`.
    fn mk_literal(self_: PyRef<'_, Self>, variable: &PyAny, value: &PyAny) -> PyResult<Bdd> {
        let value = resolve_boolean(value)?;
        let variable = self_.resolve_variable(variable)?;
        let value = self_.0.mk_literal(variable, value);
        Ok(Bdd::new(self_, value))
    }

    /// Create a new `Bdd` representing a conjunction of positive/negative literals
    /// (e.g. $x \land y \land \neg z$).
    ///
    /// See also `BoolClauseType`.
    fn mk_conjunctive_clause(self_: PyRef<'_, Self>, clause: &PyAny) -> PyResult<Bdd> {
        let value = if let Ok(valuation) = clause.extract::<BddPartialValuation>() {
            // This is useful because there is no need to copy the inner valuation.
            self_.0.mk_conjunctive_clause(valuation.as_native())
        } else {
            let valuation = self_.resolve_partial_valuation(clause)?;
            self_.0.mk_conjunctive_clause(&valuation)
        };
        Ok(Bdd::new(self_, value))
    }

    /// Create a new `Bdd` representing a disjunction of positive/negative literals
    /// (e.g. $x \lor y \lor \neg z$).
    ///
    /// See also `BoolClauseType`.
    fn mk_disjunctive_clause(self_: PyRef<'_, Self>, clause: &PyAny) -> PyResult<Bdd> {
        let value = if let Ok(valuation) = clause.extract::<BddPartialValuation>() {
            // This is useful because there is no need to copy the inner valuation.
            self_.0.mk_disjunctive_clause(valuation.as_native())
        } else {
            let valuation = self_.resolve_partial_valuation(clause)?;
            self_.0.mk_disjunctive_clause(&valuation)
        };
        Ok(Bdd::new(self_, value))
    }

    /// Create a new `Bdd` representing a conjunction of disjunctive clauses
    /// (e.g. $(x \lor y) \land (\neg x \lor z)$).
    ///
    /// This method uses a special algorithm that is typically faster than combining
    /// the clauses one by one.
    ///
    /// See also `BddVariableSet.mk_disjunctive_clause` and `BoolClauseType`.
    fn mk_cnf(self_: PyRef<'_, Self>, clauses: Vec<&PyAny>) -> PyResult<Bdd> {
        let clauses: PyResult<Vec<biodivine_lib_bdd::BddPartialValuation>> = clauses
            .into_iter()
            .map(|it| self_.resolve_partial_valuation(it))
            .collect();
        let value = self_.0.mk_cnf(&clauses?);
        Ok(Bdd::new(self_, value))
    }

    /// Create a new `Bdd` representing a disjunction of conjunctive clauses
    /// (e.g. $(x \land y) \lor (\neg x \land z)$).
    ///
    /// This method uses a special algorithm that is typically faster than combining
    /// the clauses one by one.
    ///
    /// See also `BddVariableSet.mk_conjunctive_clause` and `BoolClauseType`.
    fn mk_dnf(self_: PyRef<'_, Self>, clauses: Vec<&PyAny>) -> PyResult<Bdd> {
        let clauses: PyResult<Vec<biodivine_lib_bdd::BddPartialValuation>> = clauses
            .into_iter()
            .map(|it| self_.resolve_partial_valuation(it))
            .collect();
        let value = self_.0.mk_dnf(&clauses?);
        Ok(Bdd::new(self_, value))
    }

    /// Compute a `Bdd` which is satisfied by (and only by) valuations where exactly `k`
    /// out of the specified `variables` are `True`.
    ///
    /// If `variables` are `None`, the result is computed w.r.t. all variables
    /// managed by this `BddVariableSet`.
    #[pyo3(signature = (k, variables = None))]
    fn mk_sat_exactly_k(
        self_: PyRef<'_, Self>,
        k: usize,
        variables: Option<Vec<&PyAny>>,
    ) -> PyResult<Bdd> {
        let variables = if let Some(variables) = variables {
            self_.resolve_variables(variables)?
        } else {
            self_.0.variables()
        };
        let value = self_.0.mk_sat_exactly_k(k, &variables);
        Ok(Bdd::new(self_, value))
    }

    /// Compute a `Bdd` which is satisfied by (and only by) valuations where up to `k`
    /// out of the specified `variables` are `True`.
    ///
    /// If `variables` are `None`, the result is computed w.r.t. all variables
    /// managed by this `BddVariableSet`.
    #[pyo3(signature = (k, variables = None))]
    fn mk_sat_up_to_k(
        self_: PyRef<'_, Self>,
        k: usize,
        variables: Option<Vec<&PyAny>>,
    ) -> PyResult<Bdd> {
        let variables = if let Some(variables) = variables {
            self_.resolve_variables(variables)?
        } else {
            self_.0.variables()
        };
        let value = self_.0.mk_sat_up_to_k(k, &variables);
        Ok(Bdd::new(self_, value))
    }

    /// Evaluate the provided `BoolExpressionType` into a `Bdd`, or throw an error if the
    /// expression is invalid in this context (e.g. has unknown variables).
    fn eval_expression(self_: PyRef<'_, Self>, expression: &PyAny) -> PyResult<Bdd> {
        let expression = BooleanExpression::resolve_expression(expression)?;
        match self_.0.safe_eval_expression(expression.as_native()) {
            Some(value) => Ok(Bdd::new(self_, value)),
            None => throw_runtime_error("Expression contains unknown variables."),
        }
    }

    /// Translate a `Bdd` between two `BddVariableSet` objects.
    ///
    /// The translation is only valid if the `Bdd` relies on variables that are in both
    /// variable set, and their ordering is mutually compatible. If this is not satisfied,
    /// i.e. some of the variables don't exist in the new context, or would have to be reordered,
    /// the method returns `None`.
    fn transfer_from(
        self_: PyRef<'_, Self>,
        value: &Bdd,
        original_ctx: &BddVariableSet,
    ) -> Option<Bdd> {
        let value = self_
            .0
            .transfer_from(value.as_native(), original_ctx.as_native());
        value.map(|it| Bdd::new(self_, it))
    }
}

impl BddVariableSet {
    pub fn resolve_variable(&self, variable: &PyAny) -> PyResult<biodivine_lib_bdd::BddVariable> {
        if let Ok(id) = variable.extract::<BddVariable>() {
            return if id.__index__() < self.__len__() {
                Ok(*id.as_native())
            } else {
                throw_index_error(format!("Unknown variable ID `{}`.", id.__index__()))
            };
        }
        if let Ok(name) = variable.extract::<String>() {
            return if let Some(var) = self.0.var_by_name(name.as_str()) {
                Ok(var)
            } else {
                throw_index_error(format!("Unknown variable name `{}`.", name))
            };
        }
        throw_type_error("Expected `BddVariable` or `str`.")
    }

    pub fn resolve_partial_valuation(
        &self,
        valuation: &PyAny,
    ) -> PyResult<biodivine_lib_bdd::BddPartialValuation> {
        if let Ok(valuation) = valuation.extract::<BddPartialValuation>() {
            return Ok(valuation.as_native().clone());
        }
        if let Ok(valuation) = valuation.extract::<BddValuation>() {
            let valuation = valuation.as_native().clone();
            return Ok(biodivine_lib_bdd::BddPartialValuation::from(valuation));
        }
        if let Ok(values) = valuation.downcast::<PyDict>() {
            let mut result = biodivine_lib_bdd::BddPartialValuation::empty();
            for (key, value) in values {
                let key = self.resolve_variable(key)?;
                let value = resolve_boolean(value)?;
                result[key] = Some(value);
            }
            return Ok(result);
        }
        throw_type_error(
            "Expected `BddPartialValuation`, `BddValuation`, or `dict[BddVariable, BoolType]`.",
        )
    }

    pub fn resolve_variables(
        &self,
        variables: Vec<&PyAny>,
    ) -> PyResult<Vec<biodivine_lib_bdd::BddVariable>> {
        variables
            .into_iter()
            .map(|it| self.resolve_variable(it))
            .collect()
    }
}
