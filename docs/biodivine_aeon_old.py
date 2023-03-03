from __future__ import annotations
from typing import TypeAlias, TypedDict


# This "hack" should probably become obsolete once [this](https://github.com/PyO3/pyo3/issues/2454) is finished.

##############################################################
# >>>>>>>>>>  Start of the `lib-bdd` documentation. <<<<<<<<<<
##############################################################

class BddVariable:
    """
    References a single BDD variable that is managed by some `BddVariableSet`.
    """


class BddVariableSetBuilder:
    """
    A simpler "builder" object that can be used to gradually construct a `BddVariableSet`.
    """

    def __init__(self):
        """
        Create an empty `BddVariableSetBuilder`.
        """

    def make(self, variable: str) -> BddVariable:
        """
        Declare a single new BDD variable.
        """

    def make_all(self, variables: list[str]) -> list[BddVariable]:
        """
        Declare all variables in the given list.
        """

    def build(self) -> BddVariableSet:
        """
        Turn the builder object into the actual `BddVariableSet`.
        """


class BddVariableSet:
    """
    A "variable manager" that keeps track of how individual `BddVariable` are named and where they appear in the
    variable ordering.
    """

    # noinspection PyUnusedLocal
    def __init__(self, variables: int | list[str]):
        """
        Create a new `BddVariableSet` by providing either the list of variable names, or the total number of variables
        (in which case, "default" names `x_0`, `x_1`, ... are used).
        """

    def eval_expression(self, expression: str | BooleanExpression) -> Bdd:
        """
        Create a `Bdd` function representation based on a Boolean expression.
        """

    def var_count(self) -> int:
        """
        Return the number of variables managed by this `BddVariableSet`.
        """

    def all_variables(self) -> list[BddVariable]:
        """
        Return the list of all `BddVariable` objects managed by this `BddVariableSet`.
        """

    def find_variable(self, variable: BddVariableType) -> BddVariable | None:
        """
        Find a `BddVariable` corresponding to the given name, or `None` if the variable does not exist.
        """

    def name_of(self, variable: BddVariable) -> str:
        """
        Return the name of the given `BddVariable`.
        """

    def mk_const(self, value: bool) -> Bdd:
        """
        Create a "constant" tautology or contradiction `Bdd` object.
        """

    def mk_literal(self, variable: BddVariableType, value: bool) -> Bdd:
        """
        Create a `Bdd` corresponding to the positive or negative literal of the given `variable` (so either `var` or
        `!var`).
        """

    def mk_valuation(self, valuation: BddValuationType) -> Bdd:
        """
        Create a `Bdd` representing a single valuation of variables.
        """

    def mk_conjunctive_clause(self, partial_valuation: BddPartialValuationType) -> Bdd:
        """
        Create a `Bdd` representing a single conjunctive clause described by a partial valuation.
        """

    def mk_disjunctive_clause(self, partial_valuation: BddPartialValuationType) -> Bdd:
        """
        Create a `Bdd` representing a single disjunctive clause described by a partial valuation.
        """

    def mk_cnf(self, formula: list[BddPartialValuationType]) -> Bdd:
        """
        Create a `Bdd` of a conjunctive-normal-form formula based on a list of clauses (given as partial valuations).
        """

    def mk_dnf(self, formula: list[BddPartialValuationType]) -> Bdd:
        """
        Create a `Bdd` of a disjunctive-normal-form formula based on a list of clauses (given as partial valuations).
        """


BddVariableType: TypeAlias = BddVariable | str
"""
A BDD variable type used by the `BddVariableSet` can be either a `BddVariable`, or a `str` representation of its name.

Note that `Bdd` objects cannot use this type, because they do not have information about variable names. 
"""


class BddValuation:
    """
    Maps `BddVariable` keys to `bool` values. Supports normal dictionary-like access through indexing,
    plus the `len` method. Always assumes a fixed number of variables and defaults to `False`
    for unknown values.
    """

    # noinspection PyUnusedLocal
    def __init__(self, num_vars: int):
        """
        Create a new `BddValuation` where all values are initially set to `False`.
        """

    @staticmethod
    def from_list(values: list[bool]) -> BddValuation:
        """
        Create a new `BddValuation` initialized with the given list of values.
        """

    def into_list(self) -> list[bool]:
        """
        Convert this `BddValuation` to a "raw" list of Boolean values.
        """

    def extends(self, partial_valuation: BddPartialValuationType) -> bool:
        """
        Check if this valuation agrees in all variables with the given partial valuation.
        """


class BddPartialValuation:
    """
    Maps `BddVariable` keys to `bool` values just like a dictionary. However, it admits missing elements, where it
    returns `None` instead of throwing an exception.
    """

    def __init__(self):
        """
        Create a new empty `BddPartialValuation`.
        """

    @staticmethod
    def from_data(values: BddPartialValuationType) -> BddPartialValuation:
        """
        Create a new `BddPartialValuation` object from any of the three representation types (`list`, `dict`,
        or `tuple`).
        """

    def into_dict(self) -> dict[BddVariable, bool]:
        """
        Export the values stored in this `BddPartialValuation` as a Python dictionary.
        """

    def into_list(self) -> list[tuple[BddVariable, bool]]:
        """
        Export the values stored in this `BddPartialValuation` as a Python list.
        """

    def extends(self, partial_valuation: BddPartialValuationType) -> bool:
        """
        Check if this partial valuation agrees in all variables with the given partial valuation.
        """


BddValuationType: TypeAlias = BddValuation | list[bool]
"""
Either a `BddValuation`, or a `list[bool]`.
"""


class BddValuationIterator:
    """
    Iterates through the `BddValuation` objects representing individual satisfying valuations
    of the original `Bdd`. The iterator is lazy.
    """

    def __next__(self) -> BddValuation:
        pass


class BddClauseIterator:
    """
    Iterates through the `BddPartialValuation` objects representing individual satisfying clauses
    of the original `Bdd`. The iterator is lazy.
    """

    def __next__(self) -> BddPartialValuation:
        pass


############################################################
# >>>>>>>>>>  End of the `lib-bdd` documentation. <<<<<<<<<<
############################################################

###################################################################
# >>>>>>>>>>  Start of the `lib-parma-bn` documentation. <<<<<<<<<<
###################################################################

class VariableId:
    """
    A reference to a Boolean variable that is managed either by some `BooleanNetwork` or `RegulatoryGraph`.

    `VariableId` objects can be sorted based on their implicit ordering in the associated network.
    """

    @staticmethod
    def from_index(index: int) -> VariableId:
        """
        Convert a numeric index into a `VariableId`.

        **WARNING:** Note that this does not check if the index is actually valid in any `BooleanNetwork`.
        """

    def into_index(self) -> int:
        """
        Convert this `VariableId` into a numeric index.
        """


class ParameterId:
    """
    A reference to an explicit parameter (uninterpreted function) that is managed by some `BooleanNetwork`.

    `ParameterId` objects can be sorted based on their implicit ordering in the associated network.
    """

    @staticmethod
    def from_index(index: int) -> ParameterId:
        """
        Convert a numeric index into a `ParameterId`.

        **WARNING:** Note that this does not check if the index is actually valid in any `BooleanNetwork`.
        """

    def into_index(self) -> int:
        """
        Convert this `ParameterId` into a numeric index.
        """


class RegulationDict(TypedDict):
    """
    A dictionary that describes a single regulation within `RegulatoryGraph`.

    The `monotonicity` value can be either `activation` or `inhibition`.
    The `observable` value defaults to `True`.
    """
    source: str | VariableId
    target: str | VariableId
    observable: bool | None
    monotonicity: str | None


class ParameterDict(TypedDict):
    """
    A dictionary that describes a single explicit parameter of a `BooleanNetwork`.
    """
    name: str
    arity: int


VariableType: TypeAlias = str | VariableId
"""A network variable can be identified either using its `str` name, or its `VariableId`."""
ParameterType: TypeAlias = str | ParameterId
"""An explicit network parameter can be identified either using its `str` name, or its `ParameterId`."""


# RegulationType: TypeAlias = str | RegulationDict
# """A regulation is described either using an `.aeon` string, or using a `RegulationDict`."""

class RegulatoryGraph:
    """
    A directed graph describing the dependencies (regulations) between network variables. Each regulation can be
    marked as **monotonous** (`activation` or `inhibition`) and as **observable** (essential).
    """

    # noinspection PyUnusedLocal
    def __init__(self, variables: list[str]):
        """
        Create a new `RegulatoryGraph` using the provided list of variable names.
        """

    @staticmethod
    def from_regulations(regulations: list[str]) -> RegulatoryGraph:
        """
        Create a new `RegulatoryGraph` using a list of regulations, where each regulation is given as a string in
        the `.aeon` format.

        The graph variables will be created in alphabetic order based on these regulations.
        """

    def add_regulation(self, regulation: RegulationDict | str):
        """
        Try to add a new regulation to this `RegulatoryGraph`. The regulation is given either as a dictionary
        (see `RegulationDict`) or an `.aeon` regulation string.

        If there is a problem with adding the regulation (e.g. a variable does not exist), the method throws a
        runtime error.
        """

    def find_variable(self, variable: VariableType) -> VariableId | None:
        """
        Resolve a variable name into a `VariableId`, or `None` if such variable does not exist.
        """

    def get_variable_name(self, variable: VariableType) -> str:
        """
        Get a variable name string for the specified `VariableId`.
        """

    def set_variable_name(self, variable: VariableId, name: str):
        """
        Change the name of the given variable. Throws runtime error if the name is invalid.
        """

    def num_vars(self) -> int:
        """
        The number of variables in this `RegulatoryGraph`.
        """

    def find_regulation(self, source: VariableType, target: VariableType) -> RegulationDict | None:
        """
        Return a `RegulationDict` representing the regulation between `source` and `target`, assuming such
        regulation exists (otherwise, return `None`).

        Both `source` and `target` must exist in this graph.
        """

    def regulators(self, target: VariableType) -> set[VariableId]:
        """
        Return the set of variables that appear as regulators of the given `target` variable.
        """

    def regulators_transitive(self, target: VariableType) -> set[VariableId]:
        """
        Return the set of variables that appear as either direct or transitive regulators of the `target` variable.
        """

    def targets(self, source: VariableType) -> set[VariableId]:
        """
        Return the set of variables that appear as regulation targets of the given `source` variable.
        """

    def targets_transitive(self, source: VariableType) -> set[VariableId]:
        """
        Return the set of variables that appear as either direct or transitive regulation targets of the `source`
        variable.
        """

    def variables(self) -> list[VariableId]:
        """
        Return the list of all graph variables in their implicit ordering.
        """

    def regulations(self) -> list[RegulationDict]:
        """
        Return the list of all regulations in this graph (the order is unspecified but deterministic).
        """

    def to_dot(self) -> str:
        """
        Create a Graphviz-compatible `.dot` document which represents this `RegulatoryGraph`.

        The representation uses dashed arrows to display non-monotonous regulations and green/red arrows to
        display activation/inhibition.
        """

    def strongly_connected_components(self, restriction: list[VariableType] | None = None) -> list[set[VariableId]]:
        """
        Partition the variables of this `RegulatoryGraph` into **non-trivial strongly connected components**.

         - The result is sorted by component size.
         - A component is non-trivial if it contains more than one regulation (a self-loop counts).
         - If a `restriction` is provided, the algorithm only operates within the sub-graph of the `RegulatoryGraph`
            induced by the `restriction`.
        """

    def shortest_cycle(self, pivot: VariableType, parity: str | None = None) -> list[VariableId] | None:
        """
        Compute the shortest cycle that contains the `pivot` variable, or `None` if the variable does not belong
        to any cycle. The result is sorted in the order in which the variables appear on the cycle.
        If there are multiple shortest cycles, the algorithm should be deterministic.

         - If `parity` is specified, the algorithm only considers `positive` or `negative` cycles.
        """

    def feedback_vertex_set(
            self,
            parity: str | None = None,
            restriction: list[VariableType] | None = None
    ) -> set[VariableId]:
        """
        Compute a feedback vertex (FVS) set of this `RegulatoryGraph`.

        **Feedback vertex set** is a set of vertices (variables), which once removed, causes the `RegulatoryGraph` to
        become acyclic. The method uses a greedy algorithm to return a "reasonably small" FVS, but this can still
        be far from a minimum FVS (the FVS is always correct though, it just may not be minimal).
        You can use `RegulatoryGraph.independent_cycles` to compute a lower bound on the
        size of the minimum FVS. The algorithm should be deterministic.

         - If `parity` is specified, the algorithm only considers `positive` or `negative` cycles.
         - If a `restriction` is provided, the algorithm only operates within the sub-graph of the `RegulatoryGraph`
            induced by the `restriction`.
        """

    def independent_cycles(
            self,
            parity: str | None = None,
            restriction: list[VariableType] | None = None
    ) -> list[list[VariableId]]:
        """
        Compute the independent cycles (IC) of this `RegulatoryGraph`.

        **Independent cycles** is a set of cycles IC such that every cycle in this `RegulatoryGraph` intersects with
        some cycle in IC. The method uses a greedy algorithm to return a "reasonably large" set of independent
        cycles, but this can still be far from the maximum IC set (the set is always correct though, it just may not
        be maximal). The algorithm should be deterministic.

         - If `parity` is specified, the algorithm only considers `positive` or `negative` cycles.
         - If a `restriction` is provided, the algorithm only operates within the sub-graph of the `RegulatoryGraph`
            induced by the `restriction`.
        """


class BooleanNetwork(RegulatoryGraph):
    """
    `BooleanNetwork` represents a single logical model based on a specific `RegulatoryGraph`. The model describes
     update functions for individual variables. It inherits all methods from the `RegulatoryGraph`.

    The model can also contain **explicit** and **implicit parameters**. Explicit parameters are *uninterpreted
    functions* of fixed arity that can appear in the individual update functions. Implicit parameters come from
    "erased" update functions which are completely unknown.

    The model can be loaded and saved using `.aeon`, `.bnet` or `.sbml` format.
    """

    # noinspection PyUnusedLocal
    def __init__(self, rg: RegulatoryGraph, variables: list[str]):
        """
        Creates a new `BooleanNetwork` whose internally structure is based on the provided `RegulatoryGraph`.

        Initially, all update functions are implicit parameters (i.e. they are unknown).
        """
        super().__init__(variables)

    @staticmethod
    def from_aeon(model: str) -> BooleanNetwork:
        """
        Read a `BooleanNetwork` from the string contents of an `.aeon` model.
        """

    @staticmethod
    def from_bnet(model: str) -> BooleanNetwork:
        """
        Read a `BooleanNetwork` from the string contents of a `.bnet` model.
        """

    @staticmethod
    def from_sbml(model: str) -> BooleanNetwork:
        """
        Read a `BooleanNetwork` from the string contents of an `.sbml` model.
        """

    @staticmethod
    def from_file(path: str) -> BooleanNetwork:
        """
        Read a `BooleanNetwork` from the given file path. The format is automatically inferred from
        the file extension.
        """

    def to_sbml(self) -> str:
        """
        Convert this `BooleanNetwork` to a string representation using the `.sbml` model format.
        """

    def to_bnet(self, rename_if_necessary: bool = True) -> str:
        """
        Convert this `BooleanNetwork` to a string representation using the `.bnet` model format.

        If the network contains names that are not supported in the `.bnet` format, the names are automatically
        prefixed with `_`, which should resolve the incompatibility. You can disable this behaviour using
        `rename_if_necessary=False`.

        Also note that this conversion fails when the network contains explicit or implicit parameters,
        as these are not supported in `.bnet`.
        """

    def to_aeon(self) -> str:
        """
        Convert this `BooleanNetwork` to a string representation using the `.aeon` model format.
        """

    def graph(self) -> RegulatoryGraph:
        """
        Obtain a copy of the underlying `RegulatoryGraph`.
        """

    def set_update_function(self, variable: VariableType, expression: str | None):
        """
        Set the update function of the given `variable` to the provided expression. You can "clear" the update
        function by setting `expression=None`, which turns the update function into an implicit parameter.
        """

    def add_parameter(self, parameter: ParameterDict) -> ParameterId:
        """
        Create a new explicit parameter in this `BooleanNetwork`.

        Fails if the parameter already exists.
        """

    def num_parameters(self) -> int:
        """
        Return the number of explicit parameters (uninterpreted functions) in this `BooleanNetwork`.
        """

    def num_implicit_parameters(self) -> int:
        """
        Return the number of implicit parameters (erased update functions) in this `BooleanNetwork`.
        """

    def parameters(self) -> list[ParameterId]:
        """
        Get a list of all `ParameterId` objects tracked by this `BooleanNetwork`.
        """

    def implicit_parameters(self) -> list[VariableId]:
        """
        Get a list of variables whose update functions are unknown (i.e. implicit parameters).
        """

    def parameter_appears_in(self, parameter: ParameterType) -> list[VariableId]:
        """
        Get a list of variables whose update functions contain a specific parameter.

        Note: The inclusion test is purely syntactic. The fact that a function *contains* a parameter
        does not necessarily mean that the function's output *depends* on said parameter.
        """

    def get_update_function(self, variable: VariableType) -> str | None:
        """
        Get the update function of the given `variable`, or `None` if the update function is unknown.
        """

    def find_parameter(self, parameter: ParameterType) -> ParameterId | None:
        """
        Resolve a parameter name into a `VariableId`, or `None` if such parameter does not exist.
        """

    def get_parameter(self, parameter: ParameterType) -> ParameterDict:
        """
        Get the data for a particular explicit `parameter`.
        """

    def get_parameter_name(self, parameter: ParameterType) -> str:
        """
        Get the name of a particular explicit `parameter`.
        """

    def get_parameter_arity(self, parameter: ParameterType) -> int:
        """
        Get the arity of a particular explicit `parameter`.
        """

    def infer_regulatory_graph(self) -> BooleanNetwork:
        """
        Infer a new `BooleanNetwork` with identical update functions and a regulatory graph
        that is maximally consistent with these functions.

         > This method can be used to "force resolve" any consistency errors reported by `SymbolicAsyncGraph`.
        However, note that it also uses symbolic operations, and as such can take non-trivial amount of time
        to finish on networks with functions of large arity.
        """

    def inline_inputs(self) -> BooleanNetwork:
        """
        A "best effort" method which converts as many variables into parameters without impacting
        the behaviour of the resulting network (i.e. the result should have an isomorphic asynchronous
        state-transition graph).

         > Turning constant variables into parameters can help to significantly improve the speed of some analysis
        methods. However, keep in mind that the network is to some extent different from the original. For example,
        you can no longer use such "inlined variables" in atomic propositions during model checkin.
        """


class ModelAnnotation:
    """
    Annotations represent structured model metadata stored within the `.aeon` file comments.

    Each annotation can store a multi-line value plus a subtree of associated annotations.
    To learn more about the format, see the
    [tutorial](https://docs.rs/biodivine-lib-param-bn/latest/biodivine_lib_param_bn/tutorial/p05_model_annotations/index.html)
    in the underlying Rust library.

    """

    # noinspection PyUnusedLocal
    def __init__(self, value: str | None = None):
        """
        ---
        A new tree is empty by default, but can be initialized with a string `value`.
        """

    @staticmethod
    def from_model_string(model: str) -> ModelAnnotation:
        """
        Parse the `ModelAnnotation` tree from an AEON model string.
        """

    @staticmethod
    def from_model_path(path: str) -> ModelAnnotation:
        """
        Parse the `ModelAnnotation` tree from a file at a given `path`.
        """

    def get_value(self, path: list[str]) -> str | None:
        """
        Read an annotation value at the given tree `path`, or `None` if either the child does not exist,
        or its value is not set.
        """

    def ensure_value(self, path: list[str], value: str) -> bool:
        """
        Ensure that the given string `value` is present at the given tree `path`. If the `path` does not exist,
        it is created. If a value already exists at that `path`, it is overwritten.

        Returns `True` if the tree changed (i.e. a value was created or updated).
        """

    def clear_value(self, path: list[str]) -> str | None:
        """
        Erase any value that is present at the given tree `path`.

        Returns the erased value, or `None` if no value was present at the given `path`.
        """

    def append_value(self, path: list[str], extra: str):
        """
        Append the `extra` string to any value that is present at the given tree `path`. If there is initially
        no value at that `path`, a new empty value is created.

        Note that this method does not add any newline characters to the value: if you want to separate
        the appended value onto a new line, you must add `\\n` manually.
        """

    def clone_child(self, path: list[str]) -> ModelAnnotation | None:
        """
        Obtain a copy of a child `ModelAnnotation` at the given tree `path`, or `None` if such child does not exist.
        """

    def ensure_child(self, path: list[str], child: ModelAnnotation) -> bool:
        """
        Ensure that the given `child` tree is present at the given tree `path`.

        Returns `true` if the tree was modified (i.e. a new annotation was created or is different from the previous
        value).
        """

    def clear_child(self, path: list[str]) -> ModelAnnotation | None:
        """
        Erase the model annotation subtree that is present at the given `path`.

        Returns the erased subtree, or `None` if there was no such subtree. Also note that the method will fail
        if given an empty path, as the tree root cannot be erased.
        """

    def value(self) -> str | None:
        """
        Obtain a copy of the value in this annotation node, or `None` if no value is present.
        """

    def clone_children(self) -> dict[str, ModelAnnotation]:
        """
        Obtain a copy of all direct children in this annotation node.
        """


class GraphColors:
    """
    A `Bdd`-backed symbolic representation of a set of `SymbolicAsyncGraph` colors:
    i.e. a subset of parameter valuations.
    """

    # noinspection PyUnusedLocal
    def __init__(self, graph: SymbolicAsyncGraph, bdd: Bdd):
        """
        ---
        `GraphColors` can be initialized using a `SymbolicAsyncGraph` and a "raw" `Bdd`.

        **WARNING:** This constructor performs basic integrity checks, but it cannot ensure that
        the given `bdd` always represents a valid set of colors within the given `graph`. Whenever possible, try
        to avoid using this method directly and use the factory methods on `SymbolicAsyncGraph` instead.
        """

    def to_bdd(self) -> Bdd:
        """
        Obtain a copy of the underlying `Bdd`.
        """

    def copy_with(self, bdd: Bdd) -> GraphColors:
        """
        Create a new copy of this set using a raw `Bdd`.
        This method has the same integrity caveats as the constructor.
        """

    def copy_with_raw_string(self, string_value: Bdd) -> GraphColors:
        """
        Create a new copy of this set using a raw `Bdd` parsed from the given `str` value using `Bdd.from_raw_string`.
        This method has the same integrity caveats as the constructor.
        """

    def symbolic_size(self) -> int:
        """
        Get the approximate memory consumption of this set (in bytes).
        """

    def to_dot(self, graph: SymbolicAsyncGraph) -> str:
        """
        Obtain a Graphviz-compatible `.dot` representation of the underlying `Bdd` graph.

        The method needs the `SymbolicAsyncGraph` to properly resolve parameter names.
        """

    def cardinality(self) -> float:
        """
        Compute the approximate size of this set (the number of elements).

        Since the result is a floating-point number, it may not be precise for large values, or it can be even infinite.
        """

    def pick_singleton(self) -> GraphColors:
        """
        Return `GraphColors` set that contains a single value from this set. If this set is empty, the result
        is also empty.
        """

    def union(self, other: GraphColors) -> GraphColors:
        """
        Union of two symbolic sets.
        """

    def intersect(self, other: GraphColors) -> GraphColors:
        """
        Intersection of two symbolic sets.
        """

    def minus(self, other: GraphColors) -> GraphColors:
        """
        A difference of two symbolic sets.
        """

    def is_empty(self) -> bool:
        """
        True if this symbolic set is empty.
        """

    def is_subset(self, other: GraphColors) -> bool:
        """
        True if this set is the subset of the `other` set.
        """

    def is_subspace(self) -> bool:
        """
        True if this set represents a subspace, i.e. a hypercube in the parameter space.
        """

    def is_singleton(self) -> bool:
        """
        True if this set contains a single value from the parameter space.
        """


class GraphVertices:
    """
    A `Bdd`-backed symbolic representation of a set of `SymbolicAsyncGraph` vertices:
    i.e. a subset of variable valuations.
    """

    # noinspection PyUnusedLocal
    def __init__(self, graph: SymbolicAsyncGraph, bdd: Bdd):
        """
        ---
        `GraphVertices` can be initialized using a `SymbolicAsyncGraph` and a "raw" `Bdd`.

        **WARNING:** This constructor performs basic integrity checks, but it cannot ensure that
        the given `bdd` always represents a valid set of vertices within the given `graph`. Whenever possible, try
        to avoid using this method directly and use the factory methods on `SymbolicAsyncGraph` instead.
        """

    def to_bdd(self) -> Bdd:
        """
        Obtain a copy of the underlying `Bdd`.
        """

    def copy_with(self, bdd: Bdd) -> GraphVertices:
        """
        Create a new copy of this set using a raw `Bdd`.
        This method has the same integrity caveats as the constructor.
        """

    def copy_with_raw_string(self, string_value: Bdd) -> GraphVertices:
        """
        Create a new copy of this set using a raw `Bdd` parsed from the given `str` value using `Bdd.from_raw_string`.
        This method has the same integrity caveats as the constructor.
        """

    def symbolic_size(self) -> int:
        """
        Get the approximate memory consumption of this set (in bytes).
        """

    def to_dot(self, graph: SymbolicAsyncGraph) -> str:
        """
        Obtain a Graphviz-compatible `.dot` representation of the underlying `Bdd` graph.

        The method needs the `SymbolicAsyncGraph` to properly resolve parameter names.
        """

    def cardinality(self) -> float:
        """
        Compute the approximate size of this set (the number of elements).

        Since the result is a floating-point number, it may not be precise for large values, or it can be even infinite.
        """

    def pick_singleton(self) -> GraphVertices:
        """
        Return `GraphColors` set that contains a single value from this set. If this set is empty, the result
        is also empty.
        """

    def union(self, other: GraphVertices) -> GraphVertices:
        """
        Union of two symbolic sets.
        """

    def intersect(self, other: GraphVertices) -> GraphVertices:
        """
        Intersection of two symbolic sets.
        """

    def minus(self, other: GraphVertices) -> GraphVertices:
        """
        A difference of two symbolic sets.
        """

    def is_empty(self) -> bool:
        """
        True if this symbolic set is empty.
        """

    def is_subset(self, other: GraphVertices) -> bool:
        """
        True if this set is the subset of the `other` set.
        """

    def is_subspace(self) -> bool:
        """
        True if this set represents a subspace, i.e. a hypercube in the parameter space.
        """

    def is_singleton(self) -> bool:
        """
        True if this set contains a single value from the parameter space.
        """

    def fix_network_variable(self, variable: VariableId, value: bool) -> GraphVertices:
        """
        Return a subset of this set where `variable=value`.
        """

    def restrict_network_variable(self, variable: VariableId, value: bool) -> GraphVertices:
        """
        Restrict the set to `variable=value` and then eliminate `variable` from the set.

        The result `Y` is a superset of the original set `X`, where `x ∈ Y` iff `x[variable=value] ∈ X`.
        """

    def iterator(self) -> GraphVertexIterator:
        """
        Create an iterator over all vertices in this symbolic set (each vertex is a `list[bool]` indexed by
        the variables of the original `BooleanNetwork`).
        """


class GraphVertexIterator:
    """
    An iterator over all vertices in a specific `GraphVertices` set.

    Each vertex is represented as a `list[bool]` and can be indexed using `VariableId.into_index` values.
    """


class GraphColoredVertices:
    """
    A `Bdd`-backed symbolic representation of a relation over vertices and colors of a `SymbolicAsyncGraph`.
    """

    # noinspection PyUnusedLocal
    def __init__(self, graph: SymbolicAsyncGraph, bdd: Bdd):
        """
        ---
        `GraphColoredVertices` can be initialized using a `SymbolicAsyncGraph` and a "raw" `Bdd`.

        **WARNING:** This constructor performs basic integrity checks, but it cannot ensure that
        the given `bdd` always represents a valid colored vertex relation within the given `graph`. Whenever
        possible, try to avoid using this method directly and use the factory methods on `SymbolicAsyncGraph` instead.
        """

    def to_bdd(self) -> Bdd:
        """
        Obtain a copy of the underlying `Bdd`.
        """

    def copy_with(self, bdd: Bdd) -> GraphColoredVertices:
        """
        Create a new copy of this set using a raw `Bdd`.
        This method has the same integrity caveats as the constructor.
        """

    def copy_with_raw_string(self, string_value: Bdd) -> GraphColoredVertices:
        """
        Create a new copy of this set using a raw `Bdd` parsed from the given `str` value using `Bdd.from_raw_string`.
        This method has the same integrity caveats as the constructor.
        """

    def symbolic_size(self) -> int:
        """
        Get the approximate memory consumption of this set (in bytes).
        """

    def to_dot(self, graph: SymbolicAsyncGraph) -> str:
        """
        Obtain a Graphviz-compatible `.dot` representation of the underlying `Bdd` graph.

        The method needs the `SymbolicAsyncGraph` to properly resolve parameter names.
        """

    def cardinality(self) -> float:
        """
        Compute the approximate size of this set (the number of elements).

        Since the result is a floating-point number, it may not be precise for large values, or it can be even infinite.
        """

    def pick_singleton(self) -> GraphColoredVertices:
        """
        Return `GraphColors` set that contains a single value from this set. If this set is empty, the result
        is also empty.
        """

    def union(self, other: GraphColoredVertices) -> GraphColoredVertices:
        """
        Union of two symbolic sets.
        """

    def intersect(self, other: GraphColoredVertices) -> GraphColoredVertices:
        """
        Intersection of two symbolic sets.
        """

    def minus(self, other: GraphColoredVertices) -> GraphColoredVertices:
        """
        A difference of two symbolic sets.
        """

    def is_empty(self) -> bool:
        """
        True if this symbolic set is empty.
        """

    def is_subset(self, other: GraphColoredVertices) -> bool:
        """
        True if this set is the subset of the `other` set.
        """

    def is_subspace(self) -> bool:
        """
        True if this set represents a subspace, i.e. a hypercube in the parameter space.
        """

    def is_singleton(self) -> bool:
        """
        True if this set contains a single value from the parameter space.
        """

    def fix_network_variable(self, variable: VariableId, value: bool) -> GraphColoredVertices:
        """
        Return a subset of this relation where `variable=value`.
        """

    def restrict_network_variable(self, variable: VariableId, value: bool) -> GraphColoredVertices:
        """
        Restrict the relation to `variable=value` and then eliminate `variable` from the set.

        The result `Y` is a superset of the original relation `X`, where `x ∈ Y` iff `x[variable=value] ∈ X`.
        """

    def vertices(self) -> GraphVertices:
        """
        Compute an existential projection of this relation to the set of underlying `GraphVertices`.

        That is, a vertex appears in the result if it appears in this relation for *at least one* color.
        """

    def colors(self) -> GraphColors:
        """
        Compute an existential projection of this relation to the set of underlying `GraphColors`.

        That is, a color appears in the result if it appears in this relation for *at least one* vertex.
        """

    def pick_color(self) -> GraphColoredVertices:
        """
        Compute a subset of this relation that contains exactly one color for each vertex in the original relation.
        """

    def pick_vertex(self) -> GraphColoredVertices:
        """
        Compute a subset of this relation that contains exactly one vertex for each color in the original relation.
        """

    def minus_colors(self, color_set: GraphColors) -> GraphColoredVertices:
        """
        Remove all color-vertex pairs from this set where the color appears in the given `color_set`.
        """

    def intersect_colors(self, color_set: GraphColors) -> GraphColoredVertices:
        """
        Retain only those color-vertex pairs from this set where the color appears in the given `color_set`.
        """

    def minus_vertices(self, vertex_set: GraphVertices) -> GraphColoredVertices:
        """
        Remove all color-vertex pairs from this set where the vertex appears in the given `vertex_set`.
        """

    def intersect_vertices(self, vertex_set: GraphVertices) -> GraphColoredVertices:
        """
        Retain only those color-vertex pairs from this set where the vertex appears in the given `vertex_set`.
        """


# FunctionTable: TypeAlias = list[tuple[list[bool], BddVariable]]
# """
# Function table maps the truth values of function's inputs to the symbolic variables that represent the function
# output for this input.

# At the moment, this type alis is broken due to https://github.com/pdoc3/pdoc/issues/286. Once that bug is fixed,
# we can actually use the alias.
# """

class SymbolicContext:
    """
    `SymbolicContext` implements a mapping between the variables and parameters of `BooleanNetwork` and `BddVariable`
    objects in a symbolic encoding. It provides various low-level methods for safely creating `Bdd` objects
    that represent various parts of the underlying `BooleanNetwork`.
    """

    # noinspection PyUnusedLocal
    def __init__(self, network: BooleanNetwork, extra_state_variables: dict[VariableId, int] | None = None):
        """
        ---
        `SymbolicContext` is created from a `BooleanNetwork`, but additionally, it can also contain "extra"
        symbolic variables associated with individual network variable. These extra state variables are "anonymous"
        and can be accessed using their associated `VariableId` and *offset*.
        """

    def num_state_variables(self) -> int:
        """
        The number of symbolic variables used for encoding `BooleanNetwork` variables.
        """

    def num_parameter_variables(self) -> int:
        """
        The number of symbolic variables used for encoding `BooleanNetwork` parameters
        (both implicit and explicit).
        """

    def num_extra_state_variables(self) -> int:
        """
        The number of symbolic variables used for encoding extra state variables.
        """

    def bdd_variable_set(self) -> BddVariableSet:
        """
        Return a copy of the `BddVariableSet` used by the underlying encoding.
        """

    def state_variables(self) -> list[BddVariable]:
        """
        The list of symbolic variables used for encoding `BooleanNetwork` variables.
        """

    def all_extra_state_variables(self) -> list[BddVariable]:
        """
        The list of symbolic variables used for encoding the extra state variables supplied in the constructor.
        """

    def extra_state_variables(self, variable: VariableId) -> list[BddVariable]:
        """
        The list of symbolic variables used for encoding the extra state variables of
        a specific `BooleanNetwork` variable.
        """

    def extra_state_variables_by_offset(self, offset: int) -> list[tuple[VariableId, BddVariable]]:
        """
        Symbolic variables used for encoding the extra state variables at a particular offset. For network variables
        that have fewer than `offset` extra state variables, nothing is returned.
        """

    def get_state_variable(self, variable: VariableId) -> BddVariable:
        """
        The symbolic variable that corresponds to a particular `BooleanNetwork` variable.
        """

    def get_extra_state_variable(self, variable: VariableId, offset: int) -> BddVariable:
        """
        The symbolic variable that corresponds to the extra state variable at a given offset.

        The method fails if the offset is larger than the number of state variables declared for this network variable.
        """

    def get_implicit_function_table(self, variable: VariableId) -> list[tuple[list[bool], BddVariable]]:
        """
        The `FunctionTable` that is used to encode the implicit parameter corresponding to the given `variable`.

        The method fails if the given `variable` does not have an implicit parameter (i.e. erased update function).
        """

    def get_explicit_function_table(self, parameter: ParameterId) -> list[tuple[list[bool], BddVariable]]:
        """
        The `FunctionTable` that is used to encode the explicit parameter corresponding to the given `parameter`.
        """

    def mk_constant(self, value: bool) -> Bdd:
        """
        Build a constant `True`/`False` function `Bdd`.
        """

    def mk_state_variable_is_true(self, variable: VariableId) -> Bdd:
        """
        Build a `Bdd` function that is true if and only if the given state variable is true.
        """

    def mk_extra_state_variable_is_true(self, variable: VariableId, offset: int) -> Bdd:
        """
        Build a `Bdd` function that is true if and only if the specified extra state variable is true.

        The function fails if the `offset` is out of range.
        """

    def mk_uninterpreted_function_is_true(self, parameter: ParameterId, arguments: list[VariableId]) -> Bdd:
        """
        Build a `Bdd` function that is true if and only if the specified explicit parameter function is true
        for the given argument list.
        """

    def mk_implicit_function_is_true(self, variable: VariableId, arguments: list[VariableId]) -> Bdd:
        """
        Build a `Bdd` function that is true if and only if the specified implicit parameter function is true
        for the given argument list.
        """

    def mk_update_function_is_true(self, function: UpdateFunction) -> Bdd:
        """
        Build a `Bdd` function that is true exactly for the valuations of state and parameter variables
        where the given `function` evaluates to `True`.
        """

    def instantiate_implicit_function(
            self,
            valuation: BddValuationType | BddPartialValuationType,
            variable: VariableId,
            arguments: list[VariableId]
    ) -> Bdd:
        """
        Build a `Bdd` function which corresponds to the *instantiation* of the given implicit parameter function
        (under the given function arguments).

        A function *instantiation* is a concrete function that only depends on the argument variables, i.e. it is
        no longer parametrised. This method can be therefore used for example to enumerate the specific functions
        that correspond to individual color valuations of some `Bdd` (or a `Bdd`-encoded set).

        If the supplied `valuation` is a partial valuation, then it must specify the values of all BDD variables
        in the corresponding implicit function table (see `SymbolicContext.get_implicit_function_table`).
        """

    def instantiate_uninterpreted_function(
            self,
            valuation: BddValuationType | BddPartialValuationType,
            variable: VariableId,
            arguments: list[VariableId]
    ) -> Bdd:
        """
        Build a `Bdd` function which corresponds to the *instantiation* of the given explicit uninterpreted function
        (under the given function arguments).

        A function *instantiation* is a concrete function that only depends on the argument variables, i.e. it is
        no longer parametrised. This method can be therefore used for example to enumerate the specific functions
        that correspond to individual color valuations of some `Bdd` (or a `Bdd`-encoded set).

        If the supplied `valuation` is a partial valuation, then it must specify the values of all BDD variables
        in the corresponding explicit function table (see `SymbolicContext.get_explicit_function_table`).
        """

    def instantiate_fn_update(
            self,
            valuation: BddValuationType | BddPartialValuationType,
            function: UpdateFunction
    ) -> Bdd:
        """
        Build a `Bdd` function which corresponds to the *instantiation* of the given update function
        (under the given function arguments).

        A function *instantiation* is a concrete function that only depends on the argument variables, i.e. it is
        no longer parametrised. This method can be therefore used for example to enumerate the specific functions
        that correspond to individual color valuations of some `Bdd` (or a `Bdd`-encoded set).

        If the supplied `valuation` is a partial valuation, then it must specify the values of all BDD variables
        that are used within the function tables of all explicit parameters that appear in the given update function
        (i.e. the BDD variables which appear in `SymbolicContext.get_explicit_function_table` for each used
        uninterpreted function).

        If the supplied `function` does not depend on any parameters, the result of this function is equivalent
        to `SymbolicContext.mk_update_function_is_true`.
        """


class UpdateFunction:
    """
    A syntactic representation of an update function within a `BooleanNetwork`.

    This is similar to the `BooleanExpression`, but it has two main differences:

      - An update function can contain uninterpreted functions (explicit parameters). A `BooleanExpression` only
      admits variables and logical connectives.
      - All entities in an update function are directly tied to some `BooleanNetwork` (i.e. they use `VariableId`
      and `ParameterId` instead of string names).

    ---
    You cannot construct an `UpdateFunction` directly. Instead, you should use one of the constructor methods (`mk_*`
    and `from_*`).
    """

    @staticmethod
    def from_expression(
            expression: str | BooleanExpression,
            network: BooleanNetwork | RegulatoryGraph
    ) -> UpdateFunction:
        """
        Translate the string representation of an update function into the `UpdateFunction` object, using the provided
        `BooleanNetwork` as context.

         - Alternatively, you can provide a `BooleanExpression` instead of a raw string.
         - If the expression does not contain any parameters (uninterpreted functions), it is also sufficient to
         provide a `RegulatoryGraph` instead of a `BooleanNetwork`.
        """


class SymbolicAsyncGraph:
    """
    TODO
    """
