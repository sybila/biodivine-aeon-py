from .biodivine_aeon import *
from typing import TypeAlias, Literal, TypedDict, NotRequired

__doc__ = biodivine_aeon.__doc__
# For some reason, the following is recommned, but will cause the type aliases
# to disappear from documentation.
# if hasattr(biodivine_aeon, "__all__"):
#    __all__ = biodivine_aeon.__all__

BddVariableType: TypeAlias = BddVariable | str
"""
You can typically refer to a `Bdd` variable using its `BddVariable` ID object,
or you can use a "raw" `str` name. However, using names instead of IDs in frequently
running code incurs a performance penalty.
"""

VariableIdType: TypeAlias = VariableId | str
"""
You can typically refer to a network variable using its `VariableId` typed index,
or you can use a "raw" `str` name. However, using names instead of IDs in frequently
running code incurs a performance penalty.
"""

ParameterIdType: TypeAlias = ParameterId | str
"""
You can typically refer to a network parameter using its `ParameterId` typed index,
or you can use a "raw" `str` name. However, using names instead of IDs in frequently
running code incurs a performance penalty.
"""

BoolType: TypeAlias = Literal[0, 1] | bool
"""
Most methods can also accept `0`/`1` wherever `False`/`True` would be typically required.
"""

SignType: TypeAlias = Literal["positive", "+", "negative", "-"] | bool
"""
Sign is used in the context of regulatory networks to indicate positive/negative interaction,
but can be also used for more general graph concepts, like positive/negative cycle.
"""

VariableCollection: TypeAlias = VariableId | str | list[str] | list[VariableId] | set[str] | set[VariableId]
"""
Describes a "collection of network variables". This can be either a list of variables, or a set of variables, 
such that each variable is represented using either a string or a `VariableId`. In practice, you can even mix 
types in lists and sets, but this does not really work well with `mypy`, so we provide this simplified 
type signature instead.
"""

DynamicValuation: TypeAlias = dict[BddVariable, bool] | dict[BddVariable, Literal[0, 1]] | dict[str, bool] | dict[
    str, Literal[0, 1]]
"""
Describes types that can be converted into `BddPartialValuation`, or `BddValuation` (assuming the values of all 
variables are set). In practice, this is in fact implemented as `dict[BddVariableType, BoolType]`. 
But type inference in `mypy` gets confused by all the nested `Union` types and requires extra annotations even
in trivial cases. To avoid this, we instead list the most common cases explicitly.   
"""

BoolClauseType: TypeAlias = BddPartialValuation | BddValuation | DynamicValuation
"""
A Boolean clause represents a collection of literals. This can be either done through one of the valuation types, 
or through a regular dictionary. However, any representation other than `BddPartialValuation` incurs a performance
penalty due to conversion.
"""

BoolExpressionType: TypeAlias = BooleanExpression | str
"""
A `BooleanExpression` can be typically also substituted with its "raw" string representation. However, this
requires the expression to be repeatedly parsed whenever used and is thus slower and more error prone.
"""


class NamedRegulation(TypedDict):
    source: str
    target: str
    sign: NotRequired[SignType]
    essential: NotRequired[BoolType]
    """
    A typed dictionary that stores data about a single regulation.
    
    For backwards compatibility purposes, the `sign` key is also equivalent to `monotonicity` and `essential`
    is equivalent to `observable`. However, we do not include this in the type hints to discourage the
    usage of these deprecated dictionary keys.
    """

class IdRegulation(TypedDict):
    source: VariableId
    target: VariableId
    sign: NotRequired[SignType]
    essential: NotRequired[BoolType]
    """
    The same as `NamedRegulation`, but uses `VariableId` objects instead of string names when referring to variables.
    """