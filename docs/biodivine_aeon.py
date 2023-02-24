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


BddPartialValuationType: TypeAlias = BddPartialValuation | tuple[BddVariable, bool] | list[tuple[BddVariable, bool]] | \
                                     dict[BddVariable, bool]
"""
Either a `BddPartialValuation`, or a `dict[BddVariable, bool]`, `list[tuple[BddVariable, bool]]`,
or a single `tuple[BddVariable, bool]`.
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


class BooleanExpression:
    """
    An abstract syntax tree of a Boolean expression.
    """

    # noinspection PyUnusedLocal
    def __init__(self, expression: str):
        """
        Parse the `BooleanExpression` form a provided string representation.
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


class Bdd:
    """
    Binary decision diagram (BDD) represents a Boolean function through the means of a directed acyclic graph. To
    learn more about BDDs in general, you can read the [tutorial](
    https://docs.rs/biodivine-lib-bdd/0.4.2/biodivine_lib_bdd/tutorial/index.html) of the Biodivine BDD Rust library.

    ### BDD Equality

    Note that `Bdd` objects can be compared using equality (i.e. `==`), which works on the underlying graph
    representation. In most instances, this is the same as function equality, as the underlying representation should
    be canonical. However, in some rare instances, the `Bdd` may not be canonical (e.g. if it is read from a file
    generated using an older version of the library). As such, we recommend to always use the `Bdd.sem_eq` method
    to test *semantic equality* of BDDs.

    ### String representation

    The default string representation of a `Bdd` only contains the number of nodes and the BDD cardinality (number of
    satisfying variable assignments). However, you can dump the underlying directed graph using `Bdd.to_raw_string`
    and restore it using `Bdd.from_raw_string`. Additionally, you can dump the `Bdd` as a Graphviz-compatible `.dot`
    file using `Bdd.to_dot`.
    """

    def l_and(self, other: Bdd, limit: int | None = None) -> Bdd:
        """
        Compute the logical **conjunction** of two `Bdd` functions. This operation also represents **set intersection**.

         - The operation fails if the node count of the resulting `Bdd` exceeds the specified `limit`
        (if given).
        """

    def l_or(self, other: Bdd, limit: int | None = None) -> Bdd:
        """
        Compute the logical **disjunction** of two `Bdd` functions. This operation also represents **set union**.

         - The operation fails if the node count of the resulting `Bdd` exceeds the specified `limit`
        (if given).
        """
        pass

    def l_imp(self, other: Bdd, limit: int | None = None) -> Bdd:
        """
        Compute the logical **implication** of two `Bdd` functions (`self` implies `other`).

         - The operation fails if the node count of the resulting `Bdd` exceeds the specified `limit`
        (if given).
        """
        pass

    def l_iff(self, other: Bdd, limit: int | None = None) -> Bdd:
        """
        Compute the logical **equivalence** of two `Bdd` functions (`self` if and only if `other`).

         - The operation fails if the node count of the resulting `Bdd` exceeds the specified `limit`
        (if given).
        """
        pass

    def l_xor(self, other: Bdd, limit: int | None = None) -> Bdd:
        """
        Compute the **exclusive disjunction** of two `Bdd` functions. This operation also corresponds
        to logical **non-equivalence** or **"full outer join"** of two sets.

         - The operation fails if the node count of the resulting `Bdd` exceeds the specified `limit`
        (if given).
        """

    def l_and_not(self, other: Bdd, limit: int | None = None) -> Bdd:
        """
        Compute the logical conjunction of this `Bdd` function with the negated second argument. This operation also
        corresponds to **set difference**.

         - The operation fails if the node count of the resulting `Bdd` exceeds the specified `limit`
        (if given).
        """

    def l_not(self) -> Bdd:
        """
        Compute the logical **negation** of this `Bdd` function. This operation also corresponds to **set complement**.
        """

    def sem_eq(self, other: Bdd) -> bool:
        """
        Returns `true` if two `Bdd` objects represent the same Boolean function (**semantic equivalence**).

        Note that while this is typically the same as `==`, there are edge cases where two BDDs are structurally
        different but still semantically equivalent.
        """

    def project_exists(self, variables: BddVariable | list[BddVariable]) -> Bdd:
        """
        Computes the **existential projection** of this `Bdd` function with respect to the given variable(s).
        In the first-order logic, this operation also corresponds to the **existential quantification**.
        """

    def project_for_all(self, variables: BddVariable | list[BddVariable]) -> Bdd:
        """
        Computes the **universal projection** of this `Bdd` function with respect to the given variable(s).
        In the first-order logic, this operation also corresponds to the **universal quantification**.
        """

    def pick(self, variables: BddVariable | list[BddVariable]) -> Bdd:
        """
        Computes the "pick" operation on this `Bdd` function with respect to the given variable(s). The "pick"
        operation ensures that for every valuation of the remaining (not picked) variables that is present in the
        original `Bdd`, the result admits **exactly one** valuation of the picked variables. The operation is
        deterministic and biased towards lexicographically smaller valuations (with `0 < 1`).

        Intuitively, let us understand the `Bdd` as a relation `R ⊆ X × Y` where `X` is encoded by the picked variables,
        and `Y` is encoded by the remaining variables. Then the result `pick(R)` is the "largest" BDD where for
        every two `(x1,y1) ∈ pick(R)` and `(x2,y2) ∈ pick(R)` holds that if `y1 == y2`, then also `x1 == x2`. In other
        words, for every unique (not picked) `y ∈ Y`, there is at most one unique (picked) `x ∈ X` such that
        `(x,y) ∈ pick(R)`.

        Note that, as opposed to existential or universal projection, this operation is **neither commutative nor
        applicative** (e.g. `pick(pick(R, x), y) != pick(R, [x,y])`).
        """

    def pick_random(self, variables: BddVariable | list[BddVariable], seed: int | None = None) -> Bdd:
        """
        The same operation as `Bdd.pick`, but instead of a deterministic bias, it uses random generator to pick
        the valuations that are preserved in the result. You can optionally use a fixed `seed` to produce random but
        deterministic output.
        """

    def select(self, values: BddPartialValuation) -> Bdd:
        """
        Compute a `Bdd` where all the given `BddVariables` are **fixed** to the associated values.

        Informally, this is equivalent to constructing a "conjunctive clause" `Bdd` from the given `values` and then
        computing a conjunction with this `Bdd`.
        """

    def restrict(self, values: BddPartialValuation) -> Bdd:
        """
        The same as `Bdd.select`, but the method uses existential projection to eliminate the fixed variables
        from the resulting `Bdd`.
        """

    def valuation_iterator(self) -> BddValuationIterator:
        """
        Create an iterator that goes through all satisfying `BddValuation` objects of this `Bdd`.

        Intuitively, these are the unique **variable assignments** for which the `Bdd` function returns `true`.
        """

    def valuation_witness(self) -> BddValuation | None:
        """
        Returns a single `BddValuation` that satisfies this `Bdd`, or `None` if the `Bdd` represents a contradiction.
        """

    def valuation_random(self, seed: int | None = None) -> BddValuation | None:
        """
        Returns a single randomized `BddValuation` that satisfies this `Bdd`, or `None` if the `Bdd` represents
        a contradiction. You can use an optional fixed `seed` to produce random but deterministic output.
        """

    def clause_iterator(self) -> BddClauseIterator:
        """
        Create an iterator that goes through all satisfying `BddPartialValuation` objects of this `Bdd`.

        Intuitively, these represent individual **paths towards the `true` node** in the `Bdd` directed graph,
        or conjunctive clauses in a DNF representation of the `Bdd` function.
        """

    def clause_necessary(self) -> BddPartialValuation | None:
        """
        Compute the most restrictive conjunctive clause (partial valuation) that is still satisfied by *all*
        the satisfying valuations of this `Bdd`. In other words, this is the "tightest" over-approximation of this
        `Bdd` by a single partial valuation.

        Returns `None` when the `Bdd` is a contradiction.
        """

    def clause_witness(self) -> BddPartialValuation | None:
        """
        Returns a single `BddPartialValuation` that satisfies this `Bdd`, to `None` if the `Bdd`
        represents a contradiction.
        """

    def clause_random(self, seed: int | None = None) -> BddPartialValuation | None:
        """
        Returns a single randomized `BddPartialValuation` that satisfies this `Bdd`, or `None` if the `Bdd` represents
        a contradiction. You can use an optional fixed `seed` to produce random but deterministic output.
        """

    def to_dot(self, support_set: BddVariableSet | None = None, zero_pruned: bool = True) -> str:
        """
        Export the underlying directed graph of this `Bdd` as a **Graphviz-compatible `.dot` file**.

         - If `zero_pruned=True`, the zero node and all edges leading to it are removed
         (the result is a more concise graph).
         - If `support_set=None`, the graph will use "anonymous" variable names (`x_1`, `x_2`, ...).
         Otherwise, names provided by the `support_set` are used.
        """

    def to_raw_string(self) -> str:
        """
        Export the underlying directed graph into a "raw string" representation that can be later
        imported using `Bdd.from_raw_string`.

        Details of this format are described in the documentation of the original Rust library.
        """

    @staticmethod
    def from_raw_string() -> Bdd:
        """
        Import a `Bdd` object from a string representation previously created using `Bdd.to_raw_string`.

        **WARNING:** There are only minimal safety checks on the integrity of the imported `Bdd`. If the
        file string is corrupted, it is possible to create a `Bdd` that admits undefined behaviour.
        """

    def is_valuation(self) -> bool:
        """
        Returns true if this `Bdd` object represents a single `BddValuation`, meaning it fixes the value of
        every `BddVariable` exactly.
        """

    def is_conjunctive_clause(self) -> bool:
        """
        Returns true if this `Bdd` object represents a single conjunctive clause (i.e. a `BddPartialValuation`),
        meaning it fixed the values of some BDD variables exactly while keeps the remaining variables free.
        """

    def node_count(self) -> int:
        """
        Returns the number of nodes in the underlying graph representation of this `Bdd`. This is the "size of
        the symbolic representation" of this `Bdd` function.
        """

    def var_count(self) -> int:
        """
        Returns the number of BDD variables that are tracked within this `Bdd`.
        """

    def is_true(self) -> bool:
        """
        Returns true if this is a tautology `Bdd`.
        """

    def is_false(self) -> bool:
        """
        Returns true if this is a contradiction `Bdd`.
        """

    def cardinality(self) -> float:
        """
        Returns the number of satisfying valuations of this `Bdd` function.

        **WARNING:** Uses floating point approximation, so for very large values, the results may not be precise, or
        may even be infinite.
        """

    def to_boolean_expression(self, support_set: BddVariableSet | None = None) -> BooleanExpression:
        """
        Convert this `Bdd` function to a `BooleanExpression`. Note that the method is not running any minimization
        algorithms and as such the result can be in some cases very large.

         - If `support_set` is given, the conversion uses variable names from this set. Otherwise, default variable
         names (`x_0`, `x_1`, ...) are used.
        """

    def support_set(self) -> set[BddVariable]:
        """
        Compute the set of BDD variables that are actually used in conditions within this `Bdd`.
        """

    def size_per_variable(self) -> dict[BddVariable, int]:
        """
        Compute the contributions of individual BDD variables towards the size (node count) of this `Bdd`.
        """


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

    def feedback_vertex_set(self, parity: str | None = None, restriction: list[VariableType] | None = None) -> set[
        VariableId]:
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

    def independent_cycles(self, parity: str | None = None, restriction: list[VariableType] | None = None) -> list[
        list[VariableId]]:
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
    def __init__(self, rg: RegulatoryGraph):
        """
        Creates a new `BooleanNetwork` whose internally structure is based on the provided `RegulatoryGraph`.

        Initially, all update functions are implicit parameters (i.e. they are unknown).
        """

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
    in the original Rust library.
    """

    def __init__(self):
        """
        Create a new empty annotation tree.
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