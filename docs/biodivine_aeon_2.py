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
     - Supports negation `!`, conjunction `&`, disjunction `|`, implication `=>`, equivalence `<=>` and xor `^`.

    Note that conjunction and disjunction are always binary. You cannot use an arbitrary number of arguments. Also
    note that we don't perform any kind of simplification or normalization of these expressions.

    ## Manipulation

    You can create a `BooleanExpression` either by parsing a string in the constructor, or using
    one of the associated constructor methods (i.e. `mk_*`). The default string conversion should produce a
    result that is compatible with the constructor and is thus suitable for serialization.

    Unfortunately, the constructor methods need to *copy* their arguments. If you plan to create
    very large expressions, it may thus be faster to parse them from a single string instead, where no copying
    takes place.

    You can also create Boolean expressions using our custom "infix" syntax:

    ```python
    a = Var("a")
    expression = (a /AND/ Var("b")) /IFF/ ~a)
    ```

    ## Special methods

    `BooleanExpression` implements standard syntactic equality (overriding `__eq__` and `__ne__`). You can also
    "evaluate" the expression to `True`/`False` by calling it (`__call__`) as a function. In such case, you need to
    supply either a dictionary that has a Boolean value for all variables from `BooleanExpression.support_set`,
    or use named function arguments:

    ```python
    expr = Var("a") /OR/ Var("b")
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

    @staticmethod
    def from_constant(value: bool) -> BooleanExpression:
        """
        Create a new constant expression.
        """

    @staticmethod
    def from_variable(variable: str) -> BooleanExpression:
        """
        Create a new variable expression from the given variable name.
        """

    @staticmethod
    def from_formula(operator: str, arguments: list[BooleanExpression]) -> BooleanExpression:
        """
        Create a new Boolean formula. Operator can be either `not`, `and`, `or`, `imp`, `iff`, or `xor`. The arguments
        represent either a single argument (in the case of `not`), or two arguments (for the remaining operators).

        Keep in mind that the "left" argument is the one on index zero. Also note that right now, we do not
        support `and`/`or` operators with multiple arguments.
        """

    def as_constant(self) -> bool | None:
        """
        If this `BooleanExpression` is a constant expression, return its value, otherwise return `None`.
        """

    def as_variable(self) -> str | None:
        """
        If this `BooleanExpression` is a variable expression, return the variable name, otherwise return `None`.
        """

    def as_formula(self) -> tuple[str, list[BooleanExpression]]:
        """
        If this `BooleanExpression` is a complex expression, return the operator name and its arguments.
        See `BooleanExpression.from_formula` for the list of supported operators.

        Keep in mind that the "left" argument is the one on index zero.
        """
