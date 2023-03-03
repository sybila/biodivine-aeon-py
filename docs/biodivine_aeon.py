from __future__ import annotations
from typing import TypeAlias, Callable


###
# [Section 1] lib-bdd
#
# Use `api-coverage/lib-bdd.md` to map individual items to their Rust counterparts.
###

###
# [Section 1.1] Classes
###

class BooleanExpression:
    """
    An abstract syntax tree of a Boolean expression.

     - Boolean variables are referenced using raw strings. - Supported Boolean operators are negation `!`,
     conjunction `&`, disjunction `|`, implication `=>`, equivalence `<=>` and xor `^`.

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


class Bdd:
    """
    Binary decision diagram (BDD) represents a Boolean function through the means of a directed acyclic graph. To
    learn more about BDDs in general, you can read the [tutorial](
    https://docs.rs/biodivine-lib-bdd/0.4.2/biodivine_lib_bdd/tutorial/index.html) of the underlying Rust BDD library.

    ## Manipulation

    First, note that to perform an operation on two BDDs, they need to be based on the same set of underlying
    Boolean variables. This typically means that they were created using the same `BddVariableSet`, but technically
    it is sufficient for their `Bdd.var_count` to be equal (since a `Bdd` does not actually know the names of its
    underlying variables).

    As such, under normal circumstances, you should create new `Bdd` objects using the constructor methods provided
    by a `BddVariableSet` (to ensure the BDDs are compatible). However, if you need to serialize/deserialize BDDs,
    you can use the `Bdd.from_raw_string` / `Bdd.from_bytes` methods (and their `Bdd.to_raw_string` / `Bdd.to_bytes`
    counterparts).

    To combine multiple BDDs, you can use the logical operations (see `l_*` methods) and relational
    operations (see `r_*` methods). But you can also use our special "infix" syntax (and `~` negation):

    ```python
    vars = BddVariableSet(["a", "b"])
    a = VAR(vars, "a")
    b = VAR(vars, "b")

    bdd = (a /AND/ b) /IFF/ ~a
    bdd = (EXISTS | a | (~bdd /OR/ b))
    ```

    ### BDD Equality

    There are two notions of BDD equality. For the most part, two BDDs represent an equivalent
    Boolean function if their underlying graphs are equal, because BDD operations produce *canonical* BDDs. However,
    there are ways to create non-canonical BDDs (most notably by importing a "raw" BDD that wasn't created using this
    library). In such case, you can still test for semantic equality using a different method though.

    As such, the `Bdd` class overrides `__eq__` using the (slower) true semantic equivalence, but you can also use
    `Bdd.graph_eq` to use the (faster) graph equivalence. In general, you can always safely use `==` and only replace
    it with `graph_eq` in cases where you find it to be a significant performance bottleneck, and you are sure both
    arguments are always canonical.

    """

    def __str__(self):
        """
        Returns `Bdd(var_count=X, node_count=Y)` placeholder string for this `Bdd`.
        """

    def __repr__(self):
        """
        A `Bdd` has no canonical string representation, as it has to be created in conjunction with a `BddVariableSet`.
        As such, we only return a `<Bdd(var_count=X, node_count=Y)>` placeholder string.
        """

    def __eq__(self, other):
        """
        Implements a true *semantic* equality between `Bdd` functions.
        """

    def graph_eq(self, other: Bdd) -> bool:
        """
        Returns `true` if two `Bdd` objects share the same underlying acyclic graph. On canonical BDDs, this
        is the same as semantic equivalence.
        """

    def __hash__(self):
        """
        Each `Bdd` is hashable.
        """

    def l_not(self) -> Bdd:
        """
        Compute the logical **negation** of this `Bdd` function. This operation also corresponds to **set complement**.
        """

    def l_and(self, other: Bdd, limit: int | None = None) -> Bdd:
        """
        Compute the logical **conjunction** of two `Bdd` functions. This operation also represents **set intersection**.

         - The operation fails if the node count of the resulting `Bdd` exceeds the specified `limit` (if given).
        """

    def l_or(self, other: Bdd, limit: int | None = None) -> Bdd:
        """
        Compute the logical **disjunction** of two `Bdd` functions. This operation also represents **set union**.

         - The operation fails if the node count of the resulting `Bdd` exceeds the specified `limit` (if given).
        """

    def l_and_not(self, other: Bdd, limit: int | None = None) -> Bdd:
        """
        Compute the logical conjunction of this `Bdd` function with the negated second argument. This operation also
        corresponds to **set difference**.

         - The operation fails if the node count of the resulting `Bdd` exceeds the specified `limit` (if given).
        """

    def l_imp(self, other: Bdd, limit: int | None = None) -> Bdd:
        """
        Compute the logical **implication** of two `Bdd` functions (`self` implies `other`).

         - The operation fails if the node count of the resulting `Bdd` exceeds the specified `limit` (if given).
        """
        pass

    def l_iff(self, other: Bdd, limit: int | None = None) -> Bdd:
        """
        Compute the logical **equivalence** of two `Bdd` functions (`self` if and only if `other`).

         - The operation fails if the node count of the resulting `Bdd` exceeds the specified `limit` (if given).
        """
        pass

    def l_xor(self, other: Bdd, limit: int | None = None) -> Bdd:
        """
        Compute the **exclusive disjunction** of two `Bdd` functions. This operation also corresponds
        to logical **non-equivalence** or **"full outer join"** of two sets.

         - The operation fails if the node count of the resulting `Bdd` exceeds the specified `limit` (if given).
        """

    @staticmethod
    def apply2(
            left: Bdd,
            right: Bdd,
            function: str | OpFunction2,
            flip_left: BddVariable | None = None,
            flip_right: BddVariable | None = None,
            flip_output: BddVariable | None = None,
            limit: int | None = None,
    ) -> Bdd:
        """
        Compute a custom logical operation on two `Bdd` arguments.

         - Argument `function` is a custom function that will be applied to BDD leaf nodes (see `OpFunction2`
         for the requirements that such function must satisfy). Alternatively, you can also supply one of `and`, `or`,
         `and_not`, `imp`, `iff` and `xor`, in which case the corresponding predefined function is applied.
         - If you specify `flip_left`, `flip_right`, or `flip_output`, the validity of the given variable will be
         negated in the left, right, or output BDD respectively.
         - The `limit` argument works the same way as in the other logical operations: If the resulting `Bdd` exceeds
         this number of nodes, the function raises a runtime error.
        """

    @staticmethod
    def apply3(
            arg1: Bdd,
            arg2: Bdd,
            arg3: Bdd,
            function: OpFunction3,
            flip1: BddVariable | None = None,
            flip2: BddVariable | None = None,
            flip3: BddVariable | None = None,
    ) -> Bdd:
        """
        The same as `Bdd.apply2`, but implements a ternary instead of a binary operator.

        Note that a single `Bdd` can appear as multiple arguments with different flipped variables.
        """

    @staticmethod
    def check2(
            left: Bdd,
            right: Bdd,
            function: str | OpFunction2,
            flip_left: BddVariable | None = None,
            flip_right: BddVariable | None = None,
            flip_output: BddVariable | None = None,
    ) -> (bool, int):
        """
        Compute the number of "low level" tasks that are necessary to complete the specified BDD operation, plus
        a Boolean value indicating if the result is non-trivial (i.e. not a contradiction).

        This is generally faster than computing the actual operation, since simpler algorithm can be used. For example,
        this method can be used in greedy algorithms to estimate the complexity of competing operations when picking
        the "easiest" greedy option.

        The arguments have the same function as in the `Bdd.apply2` function.
        """

    def r_pick(self, variables: BddVariable | list[BddVariable]) -> Bdd:
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

    def r_pick_random(self, variables: BddVariable | list[BddVariable], seed: int | None = None) -> Bdd:
        """
        The same operation as `Bdd.r_pick`, but instead of a deterministic bias, it uses random generator to pick
        the valuations that are preserved in the result. You can optionally use a fixed `seed` to produce random but
        deterministic output.
        """

    def r_project_exists(self, variables: BddVariable | list[BddVariable]) -> Bdd:
        """
        Computes the **existential projection** of this `Bdd` function with respect to the given variable(s).
        In the first-order logic, this operation also corresponds to the **existential quantification**.
        """

    def r_project_for_all(self, variables: BddVariable | list[BddVariable]) -> Bdd:
        """
        Computes the **universal projection** of this `Bdd` function with respect to the given variable(s).
        In the first-order logic, this operation also corresponds to the **universal quantification**.
        """

    def r_restrict(self, values: BddPartialValuationType) -> Bdd:
        """
        The same as `Bdd.r_select`, but the method uses existential projection to eliminate the fixed variables
        from the resulting `Bdd`.
        """

    def r_select(self, values: BddPartialValuationType) -> Bdd:
        """
        Compute a `Bdd` where all the given `BddVariables` are **fixed** to the associated values.

        Informally, this is equivalent to constructing a "conjunctive clause" `Bdd` from the given `values` and then
        computing a conjunction with this `Bdd`.
        """

    def var_count(self) -> int:
        """
        Returns the number of BDD variables that are tracked within this `Bdd`.
        """

    def set_var_count(self, new_count: int):
        """
        Force update the number of variables tracked within this `Bdd`. Note that this is a very low-level operation
        that should not be used unless absolutely necessary, as it can easily create inconsistencies between BDDs.
        """

    def support_set(self) -> set[BddVariable]:
        """
        Compute the set of BDD variables that are actually used in conditions within this `Bdd`. These are variables
        that *syntactically* appear within the directed graph of this BDD. However, due to the properties of BDDs, this
        also implies that the variables have impact on the output of the Boolean function represented by this BDD.
        """

    def is_true(self) -> bool:
        """
        Returns `True` if this is a tautology `Bdd`.
        """

    def is_false(self) -> bool:
        """
        Returns `True` if this is a contradiction `Bdd`.
        """

    def is_clause(self) -> bool:
        """
        Returns `True` if this `Bdd` object represents a single **conjunctive** clause (i.e. a `BddPartialValuation`),
        meaning it fixes the values of some BDD variables exactly, while it keeps the remaining variables free.
        """

    def is_valuation(self) -> bool:
        """
        Returns `True` if this `Bdd` object represents a single `BddValuation`, meaning it fixes the value of
        every `BddVariable` exactly.
        """

    def cardinality(self, exact: bool = False) -> int:
        """
        Returns the number of satisfying valuations of this `Bdd` function.

        Note that the values resulting from this operation often substantially exceed the bounds of a 64-bit integer.
        This requires the use of arbitrary-precision integers which aren't exactly fast.

        As such, the default implementation uses an approximate method based on floating point numbers which is
        significantly faster. If the result exceeds the available value range,
        an exact method is used instead. However, note that even if the result fits within the 64-bit domain,
        it may not be exact due to the use of the floating point abstraction.

        When the exact result is strictly required, you can override this behaviour and force the method to use the
        exact algorithm.
        """

    def node_count(self) -> int:
        """
        Returns the number of nodes in the underlying graph representation of this `Bdd`. This is the "size of
        the symbolic representation" of this `Bdd` function.
        """

    def node_count_per_variable(self) -> dict[BddVariable, int]:
        """
        Compute the contributions of individual BDD variables towards the node count of this `Bdd`.

        Note that this does not count the two terminal nodes, because they do not correspond to any variable. As such,
        the sum of the results is always smaller than `Bdd.node_count`.
        """

    def witness(self) -> BddValuation | None:
        """
        A single `BddValuation` that satisfies this `Bdd`, or `None` if the `Bdd` is a contradiction.
        """

    def valuation_first(self) -> BddValuation | None:
        """
        Lexicographically the smallest valuation that satisfies this `Bdd` (or `None` if contradiction).
        """

    def valuation_last(self) -> BddValuation | None:
        """
        Lexicographically the largest valuation that satisfies this `Bdd` (or `None` if contradiction).
        """

    def valuation_random(self, seed: int | None = None) -> BddValuation | None:
        """
        Returns a single randomized `BddValuation` that satisfies this `Bdd`, or `None` if the `Bdd` represents
        a contradiction. You can use an optional fixed `seed` to produce random but deterministic output.
        """

    def valuation_most_positive(self) -> BddValuation | None:
        """
        A satisfying valuation of this `Bdd` with the most positive literals (or `None` if contradiction).

        If there are multiple such valuations, the method picks an arbitrary but deterministic valuation.
        """

    def valuation_most_negative(self) -> BddValuation | None:
        """
        A satisfying valuation of this `Bdd` with the most negative literals (or `None` if contradiction).

        If there are multiple such valuations, the method picks an arbitrary but deterministic valuation.
        """

    def valuation_iterator(self) -> BddValuationIterator:
        """
        Create an iterator that goes through all satisfying `BddValuation` objects of this `Bdd`.

        Intuitively, these are the unique **variable assignments** for which the `Bdd` function returns `true`.
        """

    def clause_first(self) -> BddPartialValuation | None:
        """
        Lexicographically the smallest conjunctive clause that satisfies this `Bdd` (or `None` if contradiction).
        """

    def clause_last(self) -> BddPartialValuation | None:
        """
        Lexicographically the largest conjunctive clause that satisfies this `Bdd` (or `None` if contradiction).
        """

    def clause_random(self, seed: int | None = None) -> BddPartialValuation | None:
        """
        Returns a single randomized `BddPartialValuation` that satisfies this `Bdd`, or `None` if the `Bdd` represents
        a contradiction. You can use an optional fixed `seed` to produce random but deterministic output.
        """

    def clause_necessary(self) -> BddPartialValuation | None:
        """
        Compute the most restrictive conjunctive clause (partial valuation) that is still satisfied by *all*
        the satisfying valuations of this `Bdd`. In other words, this is the "tightest" over-approximation of this
        `Bdd` by a single conjunctive clause.

        Returns `None` when the `Bdd` is a contradiction.
        """

    def clause_iterator(self) -> BddClauseIterator:
        """
        Create an iterator that goes through all satisfying `BddPartialValuation` objects of this `Bdd`.

        Intuitively, these represent individual **paths towards the `True` node** in the `Bdd` directed graph,
        or conjunctive clauses in a DNF representation of the `Bdd` function.
        """

    def to_expression(self, var_set: BddVariableSet | None = None) -> BooleanExpression:
        """
        Convert this `Bdd` function to a `BooleanExpression`. Note that the method is not running any minimization
        algorithms and as such the result can be in some cases very large.

         - If `var_set` is given, the conversion uses variable names from this set. Otherwise, default variable
         names (`x_0`, `x_1`, ...) are used.
        """

    @staticmethod
    def from_valuation(valuation: BddValuation) -> Bdd:
        """
        Create a singleton `Bdd` from the provided `BddValuation`.
        """

    @staticmethod
    def from_expression(expression: str | BooleanExpression, var_set: BddVariableSet) -> Bdd:
        """
        Create a `Bdd` corresponding to the given `BooleanExpression`. The `var_set` argument is mandatory, as it
        not only tracks the variable names used within the given `expression`, but also the total number of
        symbolic variables admissible in the resulting `Bdd`.
        """

    def to_raw_string(self) -> str:
        """
        Export the underlying directed graph into a "raw string" representation that can be later
        imported using `Bdd.from_raw_string`.

        Details of this format are described in the documentation of the original Rust library.
        """

    @staticmethod
    def from_raw_string(data: str) -> Bdd:
        """
        Import a `Bdd` object from a string representation previously created using `Bdd.to_raw_string`.

        **WARNING:** There are only minimal safety checks on the integrity of the imported `Bdd`. If the
        data is corrupted, it is possible to create a `Bdd` that results in undefined behaviour.
        """

    def to_bytes(self) -> bytes:
        """
        Export the underlying directed graph into "raw bytes" representation that can be later imported using
        `Bdd.from_bytes`.

        Details of this format are described in the documentation of the original Rust library.
        """

    @staticmethod
    def from_bytes(data: bytes) -> Bdd:
        """
        Import a `Bdd` object from a byte encoding previously created using `Bdd.to_bytes`.

        **WARNING:** There are only minimal safety checks on the integrity of the imported `Bdd`. If the
        data is corrupted, it is possible to create a `Bdd` that results in undefined behaviour.
        """

    def to_dot_string(self, var_set: BddVariableSet | None = None, zero_pruned: bool = True) -> str:
        """
        Export the underlying directed graph of this `Bdd` as a **Graphviz-compatible `.dot` file**.

         - If `zero_pruned=True`, the zero node and all edges leading to it are removed
         (the result is a more concise graph).
         - If `var_set=None`, the graph will use "anonymous" variable names (`x_1`, `x_2`, ...).
         Otherwise, names provided by the `support_set` are used.
        """


class BddPartialValuation:
    """
    Maps `BddVariable` keys to `bool` values just like a dictionary (meaning you can use standard indexing operations
    through `__getitem__`, `__setitem__`, `__delitem__`, `__contains__` and `__iter__`). It also implements standard
    hashing and equality functionality. However, compared to the standard dictionary, it admits missing elements,
    where it returns `None` instead of throwing an exception.

    By default, the constructor creates an empty map, but you can supply it with list/dictionary data.
    """

    # noinspection PyUnusedLocal
    def __init__(self, data: BddPartialValuationType | None = None):
        pass

    def __len__(self) -> int:
        """
        The number of BDD variables that are fixed in this valuation.
        """

    def __getitem__(self, item: BddVariable) -> bool | None:
        pass

    def __setitem__(self, key: BddVariable, value: bool):
        pass

    def __delitem__(self, key: BddVariable):
        pass

    def __contains__(self, item: BddVariable) -> bool:
        pass

    def __iter__(self) -> list[BddVariable]:
        """
        Iterate through all BDD variables that have a fixed value in this valuation.
        """

    def __hash__(self):
        pass

    def __eq__(self, other: BddPartialValuation):
        pass

    def is_empty(self) -> bool:
        """
        Returns `True` if there are no variables fixed in this valuation.
        """

    def extends(self, partial_valuation: BddPartialValuationType) -> bool:
        """
        Check if this partial valuation agrees in all variables with the given partial valuation. In other words,
        if this valuation implies the given valuation.
        """

    def support_set(self) -> set[BddVariable]:
        """
        Return the set of BDD variables that are fixed in this valuation.
        """

    def to_dict(self) -> dict[BddVariable, bool]:
        """
        Export the values stored in this `BddPartialValuation` as a dictionary.
        """

    def to_list(self) -> list[tuple[BddVariable, bool]]:
        """
        Export the values stored in this `BddPartialValuation` as a list.
        """


class BddValuation:
    """
    Maps fixed sequence of `BddVariable` keys to `bool` values. Supports normal dictionary-like access through
    indexing, plus the `len` method. Always assumes a fixed number of variables and defaults to `False`
    for unknown values.

    A valuation can be created by giving a variable count (in which case the values default to `False`),
    an explicit list of Boolean values, or a `BddPartialValuation` (in such case, the partial valuation must
    fixe all BDD variables in a continuous range).
    """

    # noinspection PyUnusedLocal
    def __init__(self, data: int | list[bool] | BddPartialValuation):
        pass

    def __getitem__(self, item: BddVariable) -> bool:
        pass

    def __setitem__(self, key: BddVariable, value: bool):
        pass

    def __len__(self) -> int:
        pass

    def __eq__(self, other: BddValuation) -> bool:
        pass

    def __hash__(self):
        pass

    def __str__(self):
        """
        Represents the valuation as a list of Boolean values.
        """

    def __repr__(self):
        """
        Represents the valuation as `BddValuation(<str>)`, where `<str>` is the vector string produced by the `__str__`
        method.
        """

    def to_list(self) -> list[bool]:
        """
        Convert this `BddValuation` to a list of "raw" Boolean values.
        """

    def extends(self, partial_valuation: BddPartialValuationType) -> bool:
        """
        Check if this valuation agrees in all variables with the given partial valuation. In other words, if this
        valuation implies the given partial valuation.
        """


class BddClauseIterator:
    """
    Iterates through the `BddPartialValuation` objects representing individual satisfying conjunctive clauses
    of the provided `Bdd`. The iterator is lazy.
    """

    # noinspection PyUnusedLocal
    def __init__(self, bdd: Bdd):
        pass

    def __str__(self):
        pass

    def __repr__(self):
        pass

    def __iter__(self) -> BddClauseIterator:
        pass

    def __next__(self) -> BddPartialValuation:
        pass


class BddValuationIterator:
    """
    Iterates through the `BddValuation` objects representing individual satisfying valuations
    of the provided `Bdd`. The iterator is lazy.
    """

    # noinspection PyUnusedLocal
    def __init__(self, bdd: Bdd):
        pass

    def __str__(self):
        pass

    def __repr__(self):
        pass

    def __iter__(self) -> BddValuationIterator:
        pass

    def __next__(self) -> BddValuation:
        pass


class BddVariable:
    """
    References a single BDD variable that is managed by some `BddVariableSet`. Variables are sorted based on the
    order in which they appear in a BDD.
    """

    def __hash__(self):
        pass

    def __eq__(self, other: BddVariable) -> bool:
        pass

    def __str__(self):
        pass

    def __repr__(self):
        pass

    def __lt__(self, other: BddVariable) -> bool:
        pass

    def __gt__(self, other: BddVariable) -> bool:
        pass

    def __le__(self, other: BddVariable) -> bool:
        pass

    def __ge__(self, other: BddVariable) -> bool:
        pass


class BddVariableSet:
    """
    A "variable manager" that keeps track of how individual `BddVariable` objects are named and where they appear in the
    variable ordering.

    It can be created by providing a list of variable names or the variable count, in which case it uses default names
    (`x_0`, `x_1`, ...). It can be also created using a `BddVariableSetBuilder`.
    """

    # noinspection PyUnusedLocal
    def __init__(self, variables: int | list[str]):
        pass

    def __str__(self):
        """
        The same as `__repr__`.
        """

    def __repr__(self):
        """
        Returns a `BddVariableSet(<variables>)` string, where `<variables>` is the list of variable names.
        """

    def eval_expression(self, expression: str | BooleanExpression) -> Bdd:
        """
        Create a `Bdd` function representation based on a `BooleanExpression`
        (or a string that is parsed as an expression).
        """

    def mk_const(self, value: bool) -> Bdd:
        """
        Create a constant `Bdd` function.
        """

    def mk_true(self) -> Bdd:
        """
        A tautology.
        """

    def mk_false(self) -> Bdd:
        """
        A contradiction.
        """

    def mk_literal(self, variable: BddVariableType, value: bool) -> Bdd:
        """
        Create a `Bdd` corresponding to the positive or negative literal of the given `variable` (so either `var` or
        `!var`).
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

    def var_count(self) -> int:
        """
        Return the number of variables managed by this `BddVariableSet`.
        """

    def find_variable(self, variable: BddVariableType) -> BddVariable | None:
        """
        Find a `BddVariable` corresponding to the given name, or `None` if the variable does not exist.
        """

    def get_variable_name(self, variable: BddVariable) -> str:
        """
        Return the name of the given `BddVariable`.
        """
    def all_variables(self) -> list[BddVariable]:
        """
        Return the list of all `BddVariable` objects managed by this `BddVariableSet`.
        """


class BddVariableSetBuilder:
    """
    A simpler "builder" object that can be used to gradually construct a `BddVariableSet`.

    You can optionally supply the constructor with a list of initial variables.
    """

    # noinspection PyUnusedLocal
    def __init__(self, variables: list[str] | None = None):
        pass

    def __str__(self):
        pass

    def __repr__(self):
        pass

    def make(self, variable: str) -> BddVariable:
        """
        Declare a single new `BddVariable`.
        """

    def make_all(self, variables: list[str]) -> list[BddVariable]:
        """
        Declare all variables in the given list.
        """

    def build(self) -> BddVariableSet:
        """
        Turn the builder object into the actual `BddVariableSet`.
        """


###
# [Section 1.2] Type aliases
###

OpFunction2: TypeAlias = Callable[[bool | None, bool | None], bool | None]
"""
A function type used when implementing arbitrary logical operations on BDDs. The function takes two optional 
Boolean arguments and returns an optional Boolean value that is expected as a result of the logical operation. The 
function must satisfy the following properties:

 - The function is pure. I.e. it has no side effects and given the same input, it always produces the same output. In 
 fact, the library will *not* call this function for every test within the BDD algorithm. It will just use it to 
 populate a lookup table that is then used within the algorithm.
 - If both arguments are concrete Boolean values, the output must also be a concrete Boolean value. If this is not
 satisfied, the BDD algorithm will fail.
 - When both arguments are `None`, the result must also be `None`. In theory, this behaviour would not result in
 a runtime error, but the corresponding operation is constant for any non-trivial `Bdd` (and thus this using such
 function is almost certainly an error).
 
For example, the "reverse implication" operator (`left | ~right`) would be implemented by the following function:

```
def rev_imp(left, right):
    if left == True:
        return True
    if right == False:
        return True
    if left == False and right == True:
        return False
    return None
```

"""

OpFunction3: TypeAlias = Callable[[bool | None, bool | None, bool | None], bool | None]
"""
The same as `OpFunction2`, but with three arguments.
"""

BddPartialValuationType: TypeAlias = BddPartialValuation | tuple[BddVariable, bool] | list[tuple[BddVariable, bool]] | \
                                     dict[BddVariable, bool]
"""
Either a `BddPartialValuation`, or a `dict[BddVariable, bool]`, `list[tuple[BddVariable, bool]]`,
or a single `tuple[BddVariable, bool]`.
"""

BddVariableType: TypeAlias = BddVariable | str
"""
A BDD variable type used by the `BddVariableSet` can be either a `BddVariable`, or a `str` representation of its name.

Note that `Bdd` objects cannot use this type, because they do not have information about variable names. 
"""

BddValuationType: TypeAlias = BddValuation | list[bool]
"""
Either a `BddValuation`, or a `list[bool]`.
"""
