from biodivine_aeon import BddVariable, BddPointer, BddVariableSet, BddVariableSetBuilder
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
    with pytest.raises(RuntimeError):
        ctx.get_variable_name("x")
    with pytest.raises(RuntimeError):
        ctx.get_variable_name(BddVariable(5))

    # BDD transfer.
    ctx2 = BddVariableSet(["a", "c"])
    not_c_1 = ctx.mk_literal("c", False)
    not_c_2 = ctx2.transfer_from(not_c_1, ctx)
    assert not_c_2 == ctx2.mk_literal("c", False)
    assert ctx2.transfer_from(ctx.mk_literal("b", True), ctx) is None

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
