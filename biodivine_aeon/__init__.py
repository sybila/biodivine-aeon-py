from .biodivine_aeon import *
from typing import TypeAlias, Literal

__doc__ = biodivine_aeon.__doc__
# For some reason, the following is recommned, but will cause the type aliases
# to disappear from documentation.
#if hasattr(biodivine_aeon, "__all__"):
#    __all__ = biodivine_aeon.__all__

BddVariableType: TypeAlias = BddVariable | str
"""
You can typically refer to a `Bdd` variable using its `BddVariable` ID object,
or you can use a "raw" `str` name. However, using names instead of IDs in frequently
running code incurs a performance penalty.
"""

BoolType: TypeAlias = Literal[0, 1] | bool
"""
Most methods can also accept `0`/`1` wherever `False`/`True` would be typically required.
"""

BoolClauseType: TypeAlias = BddPartialValuation | BddValuation | dict[BddVariableType, BoolType]
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