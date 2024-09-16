from typing import Literal, TypedDict, Mapping, Union, Optional

# Notes on Python version updates:
#  - TODO: If we ever move to 3.10, we can start using `TypeAlias`.
#  - TODO: If we ever move to 3.10, we can start using `|` instead of `Union` (and stop using `Optional`).
#  - TODO: If we ever move to 3.11, we can start using `Generic` and `TypedDict` together.
#  - TODO: If we ever move to 3.12, we can start using `type` instead of `TypeAlias`.
#  - TODO: If we ever move to 3.12, we can start using special syntax for generics.

# Counterintuitively, these two lines should actually reexport the native PyO3 module here. But it is a bit of a hack
# which does not work super reliably. Refer to the PyO3 guide for how this should be handled if it stops working.
import biodivine_aeon
from .biodivine_aeon import *

# Replace module-level documentation.
__doc__ = biodivine_aeon.__doc__

# The "__all__" list allows us to control what items are exported from the module. It has to be built as a single
# assignment, because the interpreter will cache its contents (i.e. subsequent updates are not taken into account).
# We include all Python-only types that we have in this file, plus all classes from the native module as long as
# they do not start with an underscore. We can also use this to influence the order in which items appear
# in documentation.
assert hasattr(biodivine_aeon, "__all__")
__all__ = [
              'LOG_NOTHING',
              'LOG_ESSENTIAL',
              'LOG_VERBOSE',
              'LOG_LEVEL',
              'BddVariableType',
              'VariableIdType',
              'ParameterIdType',
              'BoolType',
              'SignType',
              'BinaryOperator',
              'TemporalBinaryOperator',
              'TemporalUnaryOperator',
              'HybridOperator',
              'PhenotypeOscillation',
              'BoolClauseType',
              'BoolExpressionType',
              'Regulation',
              'IdRegulation',
              'NamedRegulation',
          ] + [x for x in biodivine_aeon.__all__ if not x.startswith("_")]

LOG_NOTHING: Literal[0] = 0
"""
No progress messages are printed.
"""
LOG_ESSENTIAL: Literal[1] = 1
"""
Progress messages are printed only for operations of "non-trivial" complexity.
"""
LOG_VERBOSE: Literal[2] = 2
"""
All progress messages are printed.
"""

LOG_LEVEL: Literal[0, 1, 2] = biodivine_aeon.LOG_LEVEL
"""
A global variable which specifies what logging messages should be printed to the standard output. These are mainly 
used to communicate progress during long-running algorithms. I.e. they typically do not communicate any new
errors or warnings, just a rough estimate of resources being used. Note that these messages introduce some overhead
into every algorithm. While we try to reduce this overhead as much as possible, especially `LOG_VERBOSE` can have
measurable impact in shorter computations. For longer computations, the overhead should tend towards zero. 

 - `LOG_NOTHING`: No logging messages are printed. 
 - `LOG_ESSENTIAL`: Logging messages are printed when resource 
 consumption exceeds what is considered "trivial" in the context of a particular algorithm. 
 - `LOG_VERBOSE`: Prints all progress messages. This setting is useful for in-depth comparisons between algorithms, 
 but can be overwhelming under normal circumstances.
   
The default value is `LOG_NOTHING`. 
"""

BddVariableType = Union[BddVariable, str]
"""
You can typically refer to a `Bdd` variable using its `BddVariable` ID object,
or you can use a "raw" `str` name. However, using names instead of IDs in frequently
running code incurs a performance penalty.
"""

VariableIdType = Union[VariableId, str]
"""
You can typically refer to a network variable using its `VariableId` typed index,
or you can use a "raw" `str` name. However, using names instead of IDs in frequently
running code incurs a performance penalty.
"""

ParameterIdType = Union[ParameterId, str]
"""
You can typically refer to a network parameter using its `ParameterId` typed index,
or you can use a "raw" `str` name. However, using names instead of IDs in frequently
running code incurs a performance penalty.
"""

BoolType = Union[bool, int]
"""
Most methods can also accept `0`/`1` wherever `False`/`True` would be typically required.

 > Note that `typing.Literal` is not used here due to how it behaves when typechecking in mappings/collections.
"""

SignType = Union[bool, Literal["positive", "+", "negative", "-"]]
"""
Sign is used in the context of regulatory networks to indicate positive/negative interaction,
but can be also used for more general graph concepts, like positive/negative cycle.
"""

BinaryOperator = Literal["and", "or", "imp", "iff", "xor"]
"""
Lists the supported Boolean binary operators.
"""

TemporalBinaryOperator = Literal["exist_until", "all_until", "exist_weak_until", "all_weak_until"]
"""
List of temporal binary operators supported by the HCTL model checker.
"""

TemporalUnaryOperator = Literal["exist_next", "all_next", "exist_future", "all_future", "exist_global", "all_global"]
"""
List of temporal unary operators supported by the HCTL model checker.
"""

HybridOperator = Literal["exists", "forall", "bind", "jump"]
"""
List of hybrid quantifiers supported by the HCTL model checker.
"""

PhenotypeOscillation = Literal["required", "allowed", "forbidden"]
"""
The possible modes of oscillation in a phenotype set:
 - `required`: To satisfy the phenotype, an attractor must visit the phenotype set intermittently
 (i.e. it cannot be proper subset).
 - `forbidden`: To satisfy the phenotype, an attractor must fully reside in the phenotype set.
 - `allowed`: To satisfy the phenotype, an attractor must intersect the phenotype set, but it does not matter whether
 it is fully contained in it or simply visits it intermittently.
"""

BoolClauseType = Union[BddPartialValuation, BddValuation, Mapping[str, BoolType], Mapping[BddVariable, BoolType]]
"""
A Boolean clause represents a collection of literals. This can be either done through one of the valuation types, 
or through a regular dictionary. However, any representation other than `BddPartialValuation` incurs a performance
penalty due to conversion.
"""

BoolExpressionType = Union[BooleanExpression, str]
"""
A `BooleanExpression` can be typically also substituted with its "raw" string representation. However, this
requires the expression to be repeatedly parsed whenever used and is thus slower and more error prone.
"""


# IDT = TypeVar('IDT', covariant=True)
# class Regulation(TypedDict, Generic[IDT]):
#     source: IDT
#     target: IDT
#     sign: Optional[SignType]
#     essential: BoolType
#     """
#     A typed dictionary that stores data about a single regulation. Parametrized by an "identifier type" which 
#     can be either `str` or `VariableId`.

#     Typically both `str` and `VariableId` are accepted as inputs, but only `VariableId` is provided as output.

#     For backwards compatibility purposes, the `sign` key is also equivalent to `monotonicity` and `essential`
#     is equivalent to `observable`. However, we do not include this in the type hints to discourage the
#     usage of these deprecated dictionary keys.
#     """

class IdRegulation(TypedDict):
    source: VariableId
    target: VariableId
    sign: Optional[SignType]
    essential: BoolType
    """
    See `Regulation` type alias.
    """


class NamedRegulation(TypedDict):
    source: str
    target: str
    sign: Optional[SignType]
    essential: BoolType
    """
    See `Regulation` type alias.
    """


Regulation = Union[IdRegulation, NamedRegulation]
"""
A typed dictionary that stores data about a single regulation. Parametrized by an "identifier type" which 
can be either `str` or `VariableId`.
    
Typically both `str` and `VariableId` are accepted as inputs, but only `VariableId` is provided as output.
    
For backwards compatibility purposes, the `sign` key is also equivalent to `monotonicity` and `essential`
is equivalent to `observable`. However, we do not include this in the type hints to discourage the
usage of these deprecated dictionary keys.

 > For backwards compatibility, the type is currently not generic, but provided as two separate aliases.
"""
