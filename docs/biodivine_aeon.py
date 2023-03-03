from __future__ import annotations
from typing import TypeAlias, TypedDict


###
# [Section 1] lib-bdd
#
# Use `api-coverage/lib-bdd.md` to map individual items to their Rust counterparts.
###

###
# [Section 1.1] Type aliases
###

###
# [Section 1.2] Classes
###

class BooleanExpression:
    """
    An abstract syntax tree of a Boolean expression.

     - Boolean variables are referenced using raw strings.
     - Supported Boolean operators are negation `!`, conjunction `&`, disjunction `|`, implication `=>`, equivalence `<=>` and xor `^`.

    Note that conjunction and disjunction are always binary. You cannot use an arbitrary number of arguments. Also
    note that we do not automatically perform any kind of simplification or normalization of these expressions.

    ## Manipulation

    You can create a `BooleanExpression` either by parsing a string in the constructor, or using
    one of the associated constructor methods (i.e. `mk_*`). The default string conversion should produce a
    result that is compatible with the constructor and is thus suitable for serialization.

    Unfortunately, the constructor methods need to *copy* their arguments. If you plan to create
    very large expressions, it may thus be faster to parse them from a single string instead, where no copying
    takes place.

    You can also create Boolean expressions using our custom "infix" syntax (plus `~` for negation),
    but the limitations regarding copying apply here as well:

    ```python
    a = VAR("a")
    expression = (a /AND/ VAR("b")) /IFF/ ~a)
    ```

    ## Special methods

    `BooleanExpression` implements standard syntactic equality (overrides `__eq__`). You can also
    "evaluate" the expression to `True`/`False` by calling it (overrides `__call__`) as a function. In such case,
    you need to supply either a dictionary that has a value for all variables from
    `BooleanExpression.support_set`, or use named function arguments:

    ```python
    expr = VAR("a") /OR/ VAR("b")
    assert expr({ "a": True, "b": False })
    assert not expr(a=False, b=False)
    ```

    """

    # noinspection PyUnusedLocal
    def __init__(self, expression: str):
        pass

    def __repr__(self) -> str:
        """
        Outputs `BooleanExpression("<str>")`, where `<str>` is the string representation of this expression.
        """

    def __str__(self) -> str:
        """
        Outputs the string representation of this expression.
        """

    def __call__(self, valuation: None | dict[str, bool], **kwargs):
        """
        Evaluates this `BooleanExpression` to `True`/`False` based on the provided variable valuation.

        The valuation can be given either as a dictionary, or using named arguments.
        """

    def __eq__(self, other: BooleanExpression):
        """
        Standard syntactic equality test.
        """

    @staticmethod
    def mk_const(value: bool) -> BooleanExpression:
        """
        Create a new `BooleanExpression` representing a Boolean **constant**.
        """

    @staticmethod
    def mk_var(name: str) -> BooleanExpression:
        """
        Create a new `BooleanExpression` representing a single Boolean **variable**.
        """

    @staticmethod
    def mk_not(inner: BooleanExpression) -> BooleanExpression:
        """
        Create a new `BooleanExpression` representing a **negation** of the `inner` expression.
        """

    @staticmethod
    def mk_and(left: BooleanExpression, right: BooleanExpression) -> BooleanExpression:
        """
        Create a new `BooleanExpression` representing a **conjunction** of two expressions.
        """

    @staticmethod
    def mk_or(left: BooleanExpression, right: BooleanExpression) -> BooleanExpression:
        """
        Create a new `BooleanExpression` representing a **disjunction** of two expressions.
        """

    @staticmethod
    def mk_imp(left: BooleanExpression, right: BooleanExpression) -> BooleanExpression:
        """
        Create a new `BooleanExpression` representing an **implication** of two expressions.
        """

    @staticmethod
    def mk_iff(left: BooleanExpression, right: BooleanExpression) -> BooleanExpression:
        """
        Create a new `BooleanExpression` representing an **equivalence** of two expressions.
        """

    @staticmethod
    def mk_xor(left: BooleanExpression, right: BooleanExpression) -> BooleanExpression:
        """
        Create a new `BooleanExpression` representing a **xor** of two expressions.
        """

    def is_const(self) -> bool:
        """
        True if this `BooleanExpression` is a constant.
        """

    def is_var(self) -> bool:
        """
        True if this `BooleanExpression` is a variable.
        """

    def is_not(self) -> bool:
        """
        True if this `BooleanExpression` is a negation.
        """

    def is_and(self) -> bool:
        """
        True if this `BooleanExpression` is a conjunction.
        """

    def is_or(self) -> bool:
        """
        True if this `BooleanExpression` is a disjunction.
        """

    def is_imp(self) -> bool:
        """
        True if this `BooleanExpression` is an implication.
        """

    def is_iff(self) -> bool:
        """
        True if this `BooleanExpression` is an equivalence.
        """

    def is_xor(self) -> bool:
        """
        True if this `BooleanExpression` is a xor.
        """

    def is_literal(self) -> bool:
        """
        True if this `BooleanExpression` is a literal, i.e. either a variable, or a negated variable.
        """

    def is_binary(self) -> bool:
        """
        True if this `BooleanExpression` represents one of the binary operations.
        """

    def as_const(self) -> bool | None:
        """
        If this `BooleanExpression` is a constant, return its value, `None` otherwise.
        """

    def as_var(self) -> str | None:
        """
        If this `BooleanExpression` is a variable, return its name, `None` otherwise.
        """

    def as_not(self) -> BooleanExpression | None:
        """
        If this `BooleanExpression` is a negation, return its inner argument, `None` otherwise.
        """

    def as_and(self) -> tuple[BooleanExpression, BooleanExpression] | None:
        """
        If this `BooleanExpression` is a conjunction, return its arguments, `None` otherwise.
        """

    def as_or(self) -> tuple[BooleanExpression, BooleanExpression] | None:
        """
        If this `BooleanExpression` is a disjunction, return its arguments, `None` otherwise.
        """

    def as_imp(self) -> tuple[BooleanExpression, BooleanExpression] | None:
        """
        If this `BooleanExpression` is an implication, return its arguments, `None` otherwise.
        """

    def as_iff(self) -> tuple[BooleanExpression, BooleanExpression] | None:
        """
        If this `BooleanExpression` is an equivalence, return its arguments, `None` otherwise.
        """

    def as_xor(self) -> tuple[BooleanExpression, BooleanExpression] | None:
        """
        If this `BooleanExpression` is a xor, return its arguments, `None` otherwise.
        """

    def as_literal(self) -> tuple[str, bool] | None:
        """
        If this `BooleanExpression` is a literal, return its name and value, `None` otherwise.
        """

    def as_binary(self) -> tuple[BooleanExpression, BooleanExpression] | None:
        """
        If this `BooleanExpression` is a binary operation, return its arguments, `None` otherwise.
        """

    def support_set(self) -> set[str]:
        """
        Compute the set of variables that appear in this `BooleanExpression`.

        Note that this computation is purely *syntactic*. Even if a variable appears in the expression, it may not
        actually have a *semantic* effect on the truthfulness of the expression.
        """