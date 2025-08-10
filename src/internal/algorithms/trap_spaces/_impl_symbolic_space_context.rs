use biodivine_lib_bdd::{bdd, Bdd};
use biodivine_lib_param_bn::{trap_spaces::SymbolicSpaceContext, VariableId};
use log::debug;

use crate::{internal::algorithms::cancellation::CancellationHandler, is_cancelled};

use super::TrapSpacesError;

// TODO: This trait exists only because I don't have access to impl `SymbolicSpaceContext` directly, as
// it comes from another crate. Once the lib_param_bn methods use the new approach, remove this trait
pub trait SymbolicSpaceContextExt {
    fn mk_can_go_to_true_ext<C: CancellationHandler>(
        &self,
        function: &Bdd,
        cancellation_handler: &C,
    ) -> Result<Bdd, TrapSpacesError>;

    fn mk_super_spaces_ext<C: CancellationHandler>(
        &self,
        spaces: &Bdd,
        cancellation_handler: &C,
    ) -> Result<Bdd, TrapSpacesError>;

    fn mk_sub_spaces_ext<C: CancellationHandler>(
        &self,
        spaces: &Bdd,
        cancellation_handler: &C,
    ) -> Result<Bdd, TrapSpacesError>;
}

impl SymbolicSpaceContextExt for SymbolicSpaceContext {
    /// Compute a [Bdd] which encodes all spaces in which the value of `function` can be
    /// `true` for some state. We assume that `function` can depend on state variables and
    /// parameter variables, but not on the dual variables used for space encoding.
    ///
    /// In other words, a space `S` satisfies the result [Bdd] if and only if there exists
    /// a state `x \in S` such that `function(x) = true`.
    ///
    /// To compute this, we evaluate the following (informal) expression:
    ///     `exists s_1...s_n: [(s_i => s_i_T) & (!s_i => s_i_F)] & function(s_1, ..., s_n)`
    ///
    /// **WARNING:** The resulting BDD also allows invalid space encodings, mostly because
    /// these are to some extent still interesting in some applications.
    ///
    fn mk_can_go_to_true_ext<C: CancellationHandler>(
        &self,
        function: &Bdd,
        cancellation_handler: &C,
    ) -> Result<Bdd, TrapSpacesError> {
        let bdd_vars = self.inner_context().bdd_variable_set();
        // Only constrain variables that are relevant to `functions`.
        let support_set = {
            let mut s = function.support_set().into_iter().collect::<Vec<_>>();
            s.sort();
            s
        };

        let mut result = function.clone();
        for var in support_set.into_iter().rev() {
            let index = self
                .inner_context()
                .state_variables()
                .iter()
                .position(|it| *it == var);
            let Some(index) = index else {
                // Skip non-state variables.
                continue;
            };

            let state_var = var;
            let (t_var, f_var) = self.get_dual_variable_pair(VariableId::from_index(index));
            let is_in_space = bdd!(bdd_vars, (state_var => t_var) & ((!state_var) => f_var));

            result = result.and(&is_in_space).var_exists(state_var);
            is_cancelled!(cancellation_handler, || result.clone())?;

            debug!(
                "Computing can-go-to-true: {}[nodes:{}].",
                result.cardinality(),
                result.size(),
            );
        }

        Ok(result)
    }

    /// Compute a [Bdd] of all spaces that are a super-space of the elements in `spaces`.
    ///
    /// The process should also preserve any "extra" variables, such as colors/parameters.
    /// Also note that this is a simple iterative algorithm that may need `O(n)` iterations
    /// and `O(n)` BDD ops to converge (`n` being the number of network variables).
    ///
    fn mk_super_spaces_ext<C: CancellationHandler>(
        &self,
        spaces: &Bdd,
        cancellation_handler: &C,
    ) -> Result<Bdd, TrapSpacesError> {
        let vars = self.bdd_variable_set();
        let mut result = spaces.clone();
        for (t_var, f_var) in self.get_dual_variables().iter().rev() {
            // Select every space in which we have `t_var=false`, but there is
            // no equivalent space with `t_var=true`. Flips `t_var` on output,
            // meaning we actually get the set of super spaces where `true` is added.
            let t_var_bdd = vars.mk_literal(*t_var, false);
            let adds_true = Bdd::fused_ternary_flip_op(
                (&result, None),
                (&t_var_bdd, None),
                (&result, Some(*t_var)),
                Some(*t_var),
                and_and_not,
            );
            is_cancelled!(cancellation_handler, || result.clone())?;

            // Symmetrically for `t_false`.
            let f_var_bdd = vars.mk_literal(*f_var, false);
            let adds_false = Bdd::fused_ternary_flip_op(
                (&result, None),
                (&f_var_bdd, None),
                (&result, Some(*f_var)),
                Some(*f_var),
                and_and_not,
            );
            is_cancelled!(cancellation_handler, || result.clone())?;

            if !adds_true.is_false() || !adds_false.is_false() {
                result = bdd!(vars, result | (adds_true | adds_false));
                is_cancelled!(cancellation_handler, || result.clone())?;
                debug!(
                    "Computing super-spaces: {}[nodes:{}].",
                    result.cardinality(),
                    result.size(),
                );
            }
        }

        Ok(result)
    }

    /// Compute a [Bdd] of all spaces that are a subspace of the elements in `spaces`.
    ///
    fn mk_sub_spaces_ext<C: CancellationHandler>(
        &self,
        spaces: &Bdd,
        cancellation_handler: &C,
    ) -> Result<Bdd, TrapSpacesError> {
        let vars = self.bdd_variable_set();
        let mut result = spaces.clone();
        for (t_var, f_var) in self.get_dual_variables().clone().into_iter().rev() {
            // A value can go down only in subspaces where both variables are set.
            // If only one variable is set, going down will just break the encoding.
            let can_go_down = bdd!(vars, t_var & f_var);
            // Has `t_var=true`, but the flipped valuation is not present. We return
            // the flipped valuation.
            let removes_true = Bdd::fused_ternary_flip_op(
                (&result, None),
                (&can_go_down, None),
                (&result, Some(t_var)),
                Some(t_var),
                and_and_not,
            );
            is_cancelled!(cancellation_handler, || result.clone())?;

            // Symmetrically for `t_false`.
            let removes_false = Bdd::fused_ternary_flip_op(
                (&result, None),
                (&can_go_down, None),
                (&result, Some(f_var)),
                Some(f_var),
                and_and_not,
            );
            is_cancelled!(cancellation_handler, || result.clone())?;

            if !removes_true.is_false() || !removes_false.is_false() {
                result = bdd!(vars, result | (removes_true | removes_false));
                is_cancelled!(cancellation_handler, || result.clone())?;

                debug!(
                    "Computing sub-spaces: {}[nodes:{}].",
                    result.cardinality(),
                    result.size(),
                );
            }
        }

        Ok(result)
    }
}

fn and_and_not(a: Option<bool>, b: Option<bool>, c: Option<bool>) -> Option<bool> {
    // Just `a & b & !c`:
    match (a, b, c) {
        (Some(true), Some(true), Some(false)) => Some(true),
        (Some(false), _, _) => Some(false),
        (_, Some(false), _) => Some(false),
        (_, _, Some(true)) => Some(false),
        (_, _, _) => None,
    }
}
