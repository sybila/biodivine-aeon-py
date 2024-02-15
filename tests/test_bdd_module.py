from biodivine_aeon import *
from typing import Literal
import pytest
import pickle


def test_bdd_variable():
    a = BddVariable(0)
    b = BddVariable(1)
    assert a == eval(repr(a))
    assert a != b
    assert a < b
    assert a <= a
    assert str(a) == "x_0"
    assert int(a) == 0

    d = {a: True, b: False}
    assert d[a] != d[b]

    data = pickle.dumps(a)
    assert pickle.loads(data) == a


def test_bdd_pointer():
    x = BddPointer()  # Default should be zero.
    y = BddPointer(True)
    z = BddPointer(10)
    assert z == eval(repr(z))
    assert x != z
    assert x < y < z
    assert str(x) == "node_0"
    assert int(x) == 0
    assert x == BddPointer.zero()
    assert y == BddPointer.one()
    assert (x.as_bool() is not None) and not x.as_bool()
    assert (y.as_bool() is not None) and y.as_bool()
    assert z.as_bool() is None
    assert x.is_terminal() and x.is_zero()
    assert y.is_terminal() and y.is_one()
    assert not (z.is_terminal() or z.is_one() or z.is_zero())

    d = {x: "foo", z: "bar"}
    assert d[x] != d[z]

    data = pickle.dumps(z)
    assert pickle.loads(data) == z


def test_bdd_variable_set_builder():
    builder = BddVariableSetBuilder()
    x = builder.add("var_x")
    a, b, c = builder.add_all(["a", "b", "c"])
    assert builder == eval(repr(builder))
    assert str(builder) == 'BddVariableSetBuilder(len = 4)'
    assert len(builder) == 4
    ctx = builder.build()
    assert ctx.variable_count() == 4
    assert ctx.get_variable_name(b) == "b"
    assert ctx.find_variable("x") is None
    assert ctx.find_variable("var_x") == x

    data = pickle.dumps(builder)
    assert pickle.loads(data) == builder

    builder2 = BddVariableSetBuilder(["a", "b"])
    assert builder2 != builder
    assert builder2.build().variable_count() == 2


def test_bdd_variable_set():
    ctx = BddVariableSet(["a", "b", "c"])
    assert str(ctx) == 'BddVariableSet(len = 3)'
    assert ctx == eval(repr(ctx))
    assert len(ctx) == 3
    assert ctx.variable_count() == 3
    assert ctx.variable_names() == ["a", "b", "c"]
    assert ctx.variable_ids() == [BddVariable(i) for i in [0, 1, 2]]

    # Variable lookup.
    var_b = ctx.find_variable("b")
    assert var_b is not None
    assert ctx.find_variable("x") is None
    assert ctx.find_variable(BddVariable(1)) == BddVariable(1)
    assert ctx.find_variable(BddVariable(10)) is None
    assert ctx.get_variable_name(var_b) == "b"
    assert ctx.get_variable_name("c") == "c"
    with pytest.raises(IndexError):
        ctx.get_variable_name("x")
    with pytest.raises(IndexError):
        ctx.get_variable_name(BddVariable(5))

    # BDD transfer.
    ctx2 = BddVariableSet(["a", "c"])
    not_c_1 = ctx.mk_literal("c", False)
    not_c_2 = ctx2.transfer_from(not_c_1, ctx)
    assert not_c_2 == ctx2.mk_literal("c", False)
    with pytest.raises(RuntimeError):
        ctx2.transfer_from(ctx.mk_literal("b", True), ctx)

    # Pickle
    data = pickle.dumps(ctx)
    assert pickle.loads(data) == ctx

    # Test various internal type conversions.
    assert ctx.mk_false() == ctx.mk_const(0)
    assert ctx.mk_true() == ctx.mk_const(1)
    assert ctx.mk_literal("a", 1) == ctx.mk_literal(BddVariable(0), True)
    assert ctx.mk_literal("a", 0) == ctx.mk_literal(BddVariable(0), False)

    clause_1 = {'a': True, 'b': False}
    clause_2: dict[BddVariable, Literal[0, 1]] = {BddVariable(0): 1, BddVariable(1): 0}
    assert ctx.mk_conjunctive_clause(clause_1) == ctx.mk_conjunctive_clause(clause_2)
    assert ctx.mk_disjunctive_clause(clause_1) == ctx.mk_disjunctive_clause(clause_2)
    assert ctx.mk_conjunctive_clause(clause_1) != ctx.mk_disjunctive_clause(clause_1)
    assert ctx.mk_conjunctive_clause(clause_1) == ctx.mk_dnf([clause_1, clause_2])
    assert ctx.mk_disjunctive_clause(clause_1) == ctx.mk_cnf([clause_1, clause_2])


def test_bdd_valuation():
    ctx = BddVariableSet(["a", "b", "c"])

    assert BddValuation(ctx).values() == [False, False, False]

    val_1 = BddValuation(ctx, [0, 1, 1])
    val_2 = BddValuation(ctx, [0, 0, 0])
    val_1_copy = BddValuation(val_1)

    assert val_1 == eval(repr(val_1))
    assert val_1 == BddValuation(ctx, [False, True, True])
    assert val_1 == BddValuation(BddPartialValuation(ctx, {'a': 0, 'b': 1, 'c': 1}))
    assert str(val_1) == "[0,1,1]"
    assert len(val_1) == 3
    assert "a" in val_1 and "z" not in val_1
    assert val_1["a"] == val_1_copy["a"]
    assert val_1[BddVariable(2)] == val_1_copy[BddVariable(2)]
    val_1_copy["a"] = 1
    assert val_1["a"] != val_1_copy["a"]

    valuations_as_keys = {val_1: "foo", val_1_copy: "bar"}
    assert valuations_as_keys[val_1] == "foo"

    data = pickle.dumps(val_2)
    assert pickle.loads(data) == val_2

    assert val_1.keys() == ctx.variable_ids()
    assert val_1.values() == [False, True, True]

    val_dict = dict(val_1.items())
    assert val_dict == {
        BddVariable(0): False,
        BddVariable(1): True,
        BddVariable(2): True
    }

    p_val_1 = BddPartialValuation(ctx, {'a': 0, 'c': 1})
    assert val_1.extends(p_val_1)
    assert not val_2.extends(p_val_1)


def test_bdd_partial_valuation():
    ctx = BddVariableSet(["a", "b", "c"])

    assert len(BddPartialValuation(ctx, {})) == 0

    val_1 = BddPartialValuation(ctx, {'a': 0, 'b': 1})
    val_2 = BddPartialValuation(BddValuation(ctx, [0, 1, 0]))
    val_3 = BddPartialValuation(val_1)

    assert val_1 == eval(repr(val_1))
    assert str(val_1) == "{'a': 0, 'b': 1}"
    assert len(val_1) == 2
    assert "a" in val_1 and "z" not in val_1
    assert (val_1['a'] is not None) and (not val_1['a'])
    assert (val_1['b'] is not None) and (val_1['b'])
    # For "valid" variables, we return `None`, but fail for invalid variables.
    assert val_1['c'] is None
    with pytest.raises(IndexError):
        assert val_1['z']
    assert val_1["a"] == val_3["a"]
    assert val_1[BddVariable(2)] == val_3[BddVariable(2)]
    val_3["a"] = 1
    assert val_1["a"] != val_3["a"]

    valuations_as_keys = {val_1: "foo", val_3: "bar"}
    assert valuations_as_keys[val_1] == "foo"

    data = pickle.dumps(val_2)
    assert pickle.loads(data) == val_2

    assert val_1.keys() == [BddVariable(0), BddVariable(1)]
    assert val_1.support_set() == {BddVariable(0), BddVariable(1)}
    assert val_1.values() == [False, True]
    val_dict = dict(val_1.items())
    assert val_dict == {
        BddVariable(0): False,
        BddVariable(1): True
    }
    assert val_dict == val_1.to_dict()

    assert val_2.extends(val_1)


def test_bdd():
    ctx = BddVariableSet(["a", "b", "c"])

    bdd_true = ctx.mk_true()
    bdd_false = ctx.mk_false()
    bdd_val = Bdd(BddValuation(ctx, [0, 1, 0]))
    bdd_clause = Bdd(BddPartialValuation(ctx, {'a': False, 'b': True}))
    bdd_x = ctx.eval_expression("(a & b) | (b & !c)")

    # Basic properties
    assert bdd_x == eval(repr(bdd_x))
    assert str(bdd_x) == "Bdd(vars = 3, len = 6, cardinality = 3)"
    assert bdd_true.is_true()
    assert not bdd_x.is_true()
    assert bdd_false.is_false()
    assert not bdd_x.is_false()
    assert bdd_val.is_valuation()
    assert not bdd_x.is_valuation()
    assert bdd_clause.is_clause()
    assert not bdd_x.is_clause()
    assert {bdd_x: True}[bdd_x]
    assert bdd_val.implies(bdd_clause)
    assert not bdd_val.semantic_eq(bdd_clause)
    assert not bdd_val.structural_eq(bdd_clause)
    assert bdd_val < bdd_clause
    assert bdd_val <= bdd_clause
    assert bdd_x(BddValuation(ctx, [1, 1, 0]))
    assert not bdd_x(BddValuation(ctx, [0, 0, 1]))
    assert len(bdd_x) == 6
    assert bdd_x.cardinality() == 3
    assert bdd_x.variable_count() == 3
    assert bdd_x.node_count() == 6
    assert bdd_x.node_count_per_variable() == {
        BddVariable(0): 1,
        BddVariable(1): 2,
        BddVariable(2): 1
    }
    assert bdd_x.support_set() == {BddVariable(0), BddVariable(1), BddVariable(2)}
    assert bdd_clause.support_set() == {BddVariable(0), BddVariable(1)}

    # Graph queries
    root = bdd_x.root()
    n_1, n_2 = bdd_x.node_links(root)
    assert bdd_x.node_variable(root) == BddVariable(0)
    assert bdd_x.node_variable(n_1) == BddVariable(1)
    assert bdd_x.node_variable(n_2) == BddVariable(1)

    # Serialization and conversions
    assert Bdd(ctx, bdd_x.data_string()) == bdd_x
    assert Bdd(ctx, bdd_x.data_bytes()) == bdd_x
    assert pickle.loads(pickle.dumps(bdd_x)) == bdd_x

    assert bdd_x.to_dot() != bdd_x.to_dot(zero_pruned=False)
    assert str(bdd_x.to_expression()) == "((a & b) | (!a & (b & !c)))"
    assert BddPartialValuation(ctx, {'a': True, 'b': True}) == BddPartialValuation(ctx, {'a': 1, 'b': 1})
    # (!a & b & c) | (a & b)
    dnf = bdd_x.to_dnf()
    dnf_basic = bdd_x.to_dnf(optimize=False)
    # For a very simple BDD like this, the optimization does not really do much.
    assert len(dnf) == len(dnf_basic)
    assert len(dnf_basic) == bdd_x.clause_cardinality()
    assert len(dnf) == 2
    assert dnf[0] == BddPartialValuation(ctx, {'a': True, 'b': True})
    assert dnf[1] == BddPartialValuation(ctx, {'a': False, 'b': True, 'c': False})
    # (a | b) & (a | !b | !c) & (!a | b)
    cnf = bdd_x.to_cnf()
    assert len(cnf) == 3
    assert cnf[0] == BddPartialValuation(ctx, {'a': True, 'b': True})
    assert cnf[1] == BddPartialValuation(ctx, {'a': True, 'b': False, 'c': False})
    assert cnf[2] == BddPartialValuation(ctx, {'a': False, 'b': True})
    assert ctx.mk_cnf(cnf) == ctx.mk_dnf(dnf)

    # Iterators
    bdd_a = ctx.mk_literal("a", True)
    assert len(list(bdd_a.valuation_iterator())) == 4
    assert len(list(bdd_a.clause_iterator())) == 1

    assert len(list(bdd_x.valuation_iterator())) == bdd_x.cardinality()
    assert len(list(bdd_x.clause_iterator())) == 2

    # Basic logic
    assert bdd_x == bdd_x.l_not().l_not()
    assert bdd_x == bdd_x.l_and(bdd_x)
    assert bdd_x.l_and(bdd_x.l_not()).is_false()
    assert bdd_x == bdd_x.l_or(bdd_x)
    assert bdd_x.l_or(bdd_x.l_not()).is_true()
    assert bdd_val.l_imp(bdd_clause).is_true()
    assert bdd_x.l_iff(bdd_x.l_not()).is_false()
    assert bdd_x.l_xor(bdd_x.l_not()).is_true()
    assert bdd_x.l_and_not(bdd_x).is_false()

    var_c = ctx.mk_literal('c', True)
    var_a = ctx.mk_literal('a', True)
    ite = Bdd.if_then_else(var_c, bdd_clause, var_a)
    assert ite == ctx.eval_expression("(c => (!a & b)) & (!c => a)")

    def my_xor(a, b):
        if a is None or b is None:
            return None
        return a != b

    with pytest.raises(InterruptedError):
        Bdd.apply2(
            bdd_val,
            bdd_clause,
            function=my_xor,
            flip_output='c',
            limit=4
        )

    result = Bdd.apply2(
        bdd_val,
        bdd_clause,
        function=my_xor,
        flip_output='c',
        limit=8
    )

    assert result == Bdd(BddValuation(ctx, [0, 1, 0]))

    def my_and_3(a, b, c):
        if a is None or b is None or c is None:
            return None
        return (a and b) and c

    result = Bdd.apply3(
        ctx.mk_literal('a', True),
        ctx.mk_literal('b', False),
        ctx.mk_literal('c', True),
        function=my_and_3,
        flip_a='a',
        flip_b='b',
        flip_c='c',
    )

    assert result == bdd_val

    result_nested = Bdd.apply_nested(
        bdd_val,
        bdd_x,
        outer_function=my_xor,
        inner_function=lambda x, y: (x or y) if (x is not None and y is not None) else None,
        variables=['b']
    )

    result_exists = Bdd.apply_with_exists(bdd_val, bdd_x, variables=['b'], function=my_xor)
    result_for_all = Bdd.apply_with_for_all(bdd_val, bdd_x, variables=['b'], function=my_xor)

    assert result_nested == result_exists
    assert result_nested != result_for_all

    # Should replace 'a' and 'b' with witnesses.
    assert bdd_true.r_pick(['a', 'b']).cardinality() == 2

    assert bdd_true.r_pick_random('a', seed=1) != bdd_true.r_pick_random('a', seed=2)
    assert bdd_true.r_pick_random('a', seed=1) == bdd_true.r_pick_random('a', seed=1)

    assert bdd_x.r_exists('b') != bdd_x.r_for_all('b')
    assert bdd_clause.r_for_all('c') == bdd_clause
    assert bdd_val.r_exists('c') == bdd_clause
    assert bdd_val.r_for_all('c').is_false()
    assert bdd_x.r_select({'b': True}).r_exists('b') == bdd_x.r_restrict({'b': True})

    assert bdd_false.valuation_first() is None
    assert bdd_x.witness() == BddValuation(ctx, [1, 1, 0])
    assert bdd_x.valuation_first() == BddValuation(ctx, [0, 1, 0])
    assert bdd_x.valuation_last() == BddValuation(ctx, [1, 1, 1])
    assert bdd_x.valuation_random(seed=1) != bdd_x.valuation_random(seed=2)
    assert bdd_x.valuation_most_positive() == BddValuation(ctx, [1, 1, 1])
    assert bdd_x.valuation_most_negative() == BddValuation(ctx, [0, 1, 0])

    assert bdd_false.clause_first() is None
    assert bdd_x.clause_first() == dnf_basic[0]
    assert bdd_x.clause_last() == dnf_basic[1]
    assert bdd_x.clause_random(seed=1) != bdd_x.clause_random(seed=2)
    assert bdd_clause.clause_necessary() == BddPartialValuation(ctx, {'a': False, 'b': True})

    expected = Bdd(BddPartialValuation(ctx, {'a': False, 'c': True}))
    assert bdd_clause.substitute('b', var_c) == expected
    assert bdd_clause.rename([('b', 'c')]) == expected


def test_boolean_expression():
    a = BooleanExpression("a")
    b = BooleanExpression("b")
    expr = BooleanExpression("(a & b) | (b & !c)")

    assert str(expr) == "((a & b) | (b & !c))"
    assert expr == eval(repr(expr))
    assert expr({'a': 1, 'b': 1, 'c': 0})
    assert not expr(a=0, b=0, c=0)
    with pytest.raises(RuntimeError):
        expr()
    assert BooleanExpression("true")()

    d = {a: "foo", b: "bar"}
    assert d[a] == "foo"
    assert d[a] != d[b]

    assert expr == pickle.loads(pickle.dumps(expr))

    expr_inner = expr.as_binary()
    assert expr_inner is not None
    op, l, r = expr_inner
    assert op == "or"
    assert l.__root__() == r.__root__() == expr

    assert BooleanExpression.mk_const(0) == BooleanExpression.mk_const(False)
    assert BooleanExpression.mk_const(0) != BooleanExpression.mk_const(True)
    assert BooleanExpression("a") == BooleanExpression.mk_var("a")
    assert BooleanExpression("!a") == BooleanExpression.mk_not(a)
    assert BooleanExpression("a & b") == BooleanExpression.mk_and(a, b)
    assert BooleanExpression("a | b") == BooleanExpression.mk_or(a, b)
    assert BooleanExpression("a => b") == BooleanExpression.mk_imp(a, b)
    assert BooleanExpression("a <=> b") == BooleanExpression.mk_iff(a, b)
    assert BooleanExpression("a ^ b") == BooleanExpression.mk_xor(a, b)

    assert BooleanExpression("true").is_const() and not BooleanExpression("a").is_const()
    assert BooleanExpression("a").is_var() and not BooleanExpression("true").is_var()
    assert BooleanExpression("!a").is_not() and not BooleanExpression("a").is_not()
    assert BooleanExpression("a & b").is_and() and not BooleanExpression("a | b").is_and()
    assert BooleanExpression("a | b").is_or() and not BooleanExpression("a & b").is_or()
    assert BooleanExpression("a => b").is_imp() and not BooleanExpression("a & b").is_imp()
    assert BooleanExpression("a <=> b").is_iff() and not BooleanExpression("a & b").is_iff()
    assert BooleanExpression("a ^ b").is_xor() and not BooleanExpression("a & b").is_xor()
    assert BooleanExpression("a").is_literal() and BooleanExpression("!a").is_literal()
    assert BooleanExpression("a & b").is_binary() and not BooleanExpression("!a").is_binary()

    assert BooleanExpression("true").as_const()
    assert BooleanExpression("a").as_var() == "a"
    assert BooleanExpression("!a").as_var() is None
    assert BooleanExpression("!a").as_not() == a
    assert BooleanExpression("a").as_not() is None
    assert BooleanExpression("a & b").as_and() == (a, b)
    assert BooleanExpression("a | b").as_and() is None
    assert BooleanExpression("a | b").as_or() == (a, b)
    assert BooleanExpression("a & b").as_or() is None
    assert BooleanExpression("a => b").as_imp() == (a, b)
    assert BooleanExpression("a & b").as_imp() is None
    assert BooleanExpression("a <=> b").as_iff() == (a, b)
    assert BooleanExpression("a & b").as_iff() is None
    assert BooleanExpression("a ^ b").as_xor() == (a, b)
    assert BooleanExpression("a & b").as_xor() is None
    assert BooleanExpression("a").as_literal() == ("a", True)
    assert BooleanExpression("!a").as_literal() == ("a", False)
    assert BooleanExpression("!!a").as_literal() is None
    assert BooleanExpression("a & b").as_binary() == ("and", a, b)
    assert BooleanExpression("a").as_binary() is None

    assert expr.support_set() == {"a", "b", "c"}
