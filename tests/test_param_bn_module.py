import pytest

from biodivine_aeon import *
import pickle
import copy
from pathlib import Path
from functools import reduce
from typing import Literal


def test_drop_1():
    network = BooleanNetwork.from_file("./tests/model-1.aeon")
    to_remove = []
    for var in network.variable_names():
        if len(network.predecessors(var)) == 0 and len(network.successors(var)) == 0:
            to_remove.append(var)
    dropped = network.drop(to_remove)
    assert network.variable_count() == dropped.variable_count() + len(to_remove)


def test_drop_2():
    network = BooleanNetwork.from_file('./tests/model-2.aeon')
    # Test that we can safely prune the network to its top SCC.
    for scc in network.strongly_connected_components():
        print(scc)
        if network.backward_reachable(list(scc)) == scc:
            to_remove = [v for v in network.variables() if v not in scc]
            dropped = network.drop(to_remove)
            print(dropped)
            assert dropped.variable_count() == len(scc)


def test_variable_id():
    a = VariableId(0)
    b = VariableId(1)
    assert a == eval(repr(a))
    assert a != b
    assert a < b
    assert a <= a
    assert str(a) == "v_0"
    assert int(a) == 0

    d = {a: True, b: False}
    assert d[a] != d[b]

    data = pickle.dumps(a)
    assert pickle.loads(data) == a


def test_parameter_id():
    a = ParameterId(0)
    b = ParameterId(1)
    assert a == eval(repr(a))
    assert a != b
    assert a < b
    assert a <= a
    assert str(a) == "p_0"
    assert int(a) == 0

    d = {a: True, b: False}
    assert d[a] != d[b]

    data = pickle.dumps(a)
    assert pickle.loads(data) == a


def test_regulatory_graph():
    rg1 = RegulatoryGraph(["a", "b", "c"])
    rg1.add_regulation("a -> b")
    rg1.add_regulation({
        'source': 'b',
        'target': 'c',
        'sign': '-',
        'essential': False,
    })
    rg1.add_regulation({
        'source': VariableId(2),
        'target': VariableId(0),
        'essential': True,
        'sign': None,
    })
    with pytest.raises(TypeError):
        rg1.add_regulation({
            'source': 'b',
            'target': 'a',
            'sign': '-',
            'essential': False,
            'extra': 1
        })  # type: ignore
    rg2 = RegulatoryGraph(None, ["a -> b", "b -|? c", "c -? a"])

    assert rg1 == rg2

    assert str(rg1) == "RegulatoryGraph(variables=3, regulations=3)"
    assert rg1 == eval(repr(rg1))
    assert rg1 == pickle.loads(pickle.dumps(rg1))
    assert rg1 == copy.copy(rg1)

    assert rg1 == RegulatoryGraph.from_aeon(rg1.to_aeon())
    Path("tmp.aeon").write_text(rg1.to_aeon())
    assert rg1 == RegulatoryGraph.from_file("tmp.aeon")
    Path("tmp.aeon").unlink()

    assert rg1.to_dot() == rg2.to_dot()

    assert rg1.variable_count() == 3
    assert rg1.variable_names() == ["a", "b", "c"]
    assert rg1.variables() == [VariableId(0), VariableId(1), VariableId(2)]
    assert rg1.find_variable("b") == VariableId(1)
    assert rg1.find_variable("z") is None
    assert rg1.find_variable(VariableId(4)) is None
    assert rg1.get_variable_name(VariableId(2)) == "c"
    rg1.set_variable_name("c", "d")
    assert rg1.get_variable_name(VariableId(2)) == "d"
    rg1.set_variable_name("d", "c")
    assert rg1.get_variable_name(VariableId(2)) == "c"

    assert rg1.regulation_count() == 3
    assert rg1.regulations() == [
        {'source': VariableId(0), 'target': VariableId(1), 'sign': '+', 'essential': True},
        {'source': VariableId(1), 'target': VariableId(2), 'sign': '-', 'essential': False},
        {'source': VariableId(2), 'target': VariableId(0), 'sign': None, 'essential': True},
    ]
    assert rg1.regulation_strings() == ["a -> b", "b -|? c", "c -? a"]
    assert rg1.find_regulation('a', 'c') is None
    assert rg1.find_regulation('c', 'a') == {
        'source': VariableId(2),
        'target': VariableId(0),
        'essential': True,
        'sign': None,
    }
    rg1.add_regulation('a -?? c')
    assert rg1.find_regulation('a', 'c') == {
        'source': VariableId(0),
        'target': VariableId(2),
        'essential': False,
        'sign': None,
    }
    rg1.remove_regulation('a', 'c')
    assert rg1.find_regulation('a', 'c') is None
    assert rg1.ensure_regulation('a -?? c ') is None
    assert rg1.ensure_regulation('a -| c') == {
        'source': VariableId(0),
        'target': VariableId(2),
        'essential': False,
        'sign': None
    }
    assert rg1.find_regulation('a', 'c') == {
        'source': VariableId(0),
        'target': VariableId(2),
        'essential': True,
        'sign': '-',
    }

    rg1e = rg1.extend(['d', 'e'])
    rg1e.add_regulation('c -> d')
    rg1e.add_regulation('e -| b')
    assert rg1e != rg1
    assert rg1e.find_variable('e') == VariableId(4)

    assert rg1e.drop(['d', 'e']) == rg1

    # Here, the result is actually the same because the inlined variables do not interact meaningfully with the rest.
    assert rg1e.inline_variable('d').inline_variable('e') == rg1
    # Keep in mind that rg1 is now extended with 'a -| c'
    assert rg1.inline_variable('c') == RegulatoryGraph(None, ["a -> b", "b -?? a", "a -? a"])

    assert rg1.predecessors('c') == {VariableId(0), VariableId(1)}
    assert rg1.successors('a') == {VariableId(1), VariableId(2)}

    assert rg1e.backward_reachable('d') == set(rg1e.variables())
    assert rg1e.forward_reachable('d') == {VariableId(3)}
    assert rg1e.backward_reachable('e') == {VariableId(4)}
    assert rg1e.forward_reachable('e') == set(rg1e.variables())
    assert rg1e.backward_reachable(['d', 'e']) == set(rg1e.variables())
    assert rg1e.forward_reachable(['d', 'e']) == set(rg1e.variables())

    # FVS and IC are very simple, since there are effectively just two cycles of very ambiguous monotonicity.
    assert rg1.feedback_vertex_set() == {VariableId(0)}
    assert len(rg1.feedback_vertex_set()) == len(rg1e.feedback_vertex_set())
    assert rg1.feedback_vertex_set(parity='+') == rg1.feedback_vertex_set()
    assert rg1.feedback_vertex_set(parity='-') == rg1.feedback_vertex_set()
    assert rg1.feedback_vertex_set(subgraph=['a', 'b']) == set()

    assert rg1.independent_cycles() == [[VariableId(0), VariableId(2)]]
    assert rg1.independent_cycles() == rg1e.independent_cycles()
    assert rg1.independent_cycles(parity='+') == rg1.independent_cycles(parity='-')
    assert rg1.independent_cycles(subgraph=['a', 'b']) == []

    assert rg1.strongly_connected_components() == rg1e.strongly_connected_components()
    assert rg1.strongly_connected_components(subgraph=['a', 'b']) == []

    assert rg1.weakly_connected_components() == [set(rg1.variables())]
    assert rg1e.weakly_connected_components(subgraph=['d', 'e']) == [{VariableId(3)}, {VariableId(4)}]

    assert rg1.shortest_cycle('a') == [VariableId(0), VariableId(2)]
    assert rg1.shortest_cycle('b') == [VariableId(1), VariableId(2), VariableId(0)]
    assert rg1.shortest_cycle('a', length=1) is None
    assert rg1.shortest_cycle('a', subgraph=['a', 'b']) is None
    assert rg1.shortest_cycle('a', parity='+') == rg1.shortest_cycle('a', parity='-')


def test_boolean_network_inheritance():
    """
    This is *almost* the same test as for RegulatoryGraph, but it performs the same operations on a BooleanNetwork
    to check that they still work the same.
    """
    bn1 = BooleanNetwork(["a", "b", "c"])
    bn1.add_regulation("a -> b")
    bn1.add_regulation({
        'source': 'b',
        'target': 'c',
        'sign': '-',
        'essential': False,
    })
    bn1.add_regulation({
        'source': VariableId(2),
        'target': VariableId(0),
        'essential': True,
        'sign': None,
    })
    with pytest.raises(TypeError):
        bn1.add_regulation({
            'source': 'b',
            'target': 'a',
            'sign': '-',
            'essential': False,
            'extra': 1
        })  # type: ignore
    bn2 = BooleanNetwork(None, ["a -> b", "b -|? c", "c -? a"])

    assert bn1 == bn2

    assert str(bn1) == "BooleanNetwork(variables=3, regulations=3, explicit_parameters=0, implicit_parameters=3)"
    assert bn1 == eval(repr(bn1))
    assert bn1 == pickle.loads(pickle.dumps(bn1))
    assert bn1 == copy.copy(bn1)

    assert bn1 == BooleanNetwork.from_aeon(bn1.to_aeon())
    Path("tmp.aeon").write_text(bn1.to_aeon())
    assert bn1 == BooleanNetwork.from_file("tmp.aeon")
    Path("tmp.aeon").unlink()

    assert bn1.to_dot() == bn2.to_dot()

    assert bn1.variable_count() == 3
    assert bn1.variable_names() == ["a", "b", "c"]
    assert bn1.variables() == [VariableId(0), VariableId(1), VariableId(2)]
    assert bn1.find_variable("b") == VariableId(1)
    assert bn1.find_variable("z") is None
    assert bn1.find_variable(VariableId(4)) is None
    assert bn1.get_variable_name(VariableId(2)) == "c"
    bn1.set_variable_name("c", "d")
    assert bn1.get_variable_name(VariableId(2)) == "d"
    bn1.set_variable_name("d", "c")
    assert bn1.get_variable_name(VariableId(2)) == "c"

    assert bn1.regulation_count() == 3
    assert bn1.regulations() == [
        {'source': VariableId(0), 'target': VariableId(1), 'sign': '+', 'essential': True},
        {'source': VariableId(1), 'target': VariableId(2), 'sign': '-', 'essential': False},
        {'source': VariableId(2), 'target': VariableId(0), 'sign': None, 'essential': True},
    ]
    assert bn1.regulation_strings() == ["a -> b", "b -|? c", "c -? a"]
    assert bn1.find_regulation('a', 'c') is None
    assert bn1.find_regulation('c', 'a') == {
        'source': VariableId(2),
        'target': VariableId(0),
        'essential': True,
        'sign': None,
    }
    bn1.add_regulation('a -?? c')
    assert bn1.find_regulation('a', 'c') == {
        'source': VariableId(0),
        'target': VariableId(2),
        'essential': False,
        'sign': None,
    }
    bn1.remove_regulation('a', 'c')
    assert bn1.find_regulation('a', 'c') is None
    assert bn1.ensure_regulation('a -?? c ') is None
    assert bn1.ensure_regulation('a -| c') == {
        'source': VariableId(0),
        'target': VariableId(2),
        'essential': False,
        'sign': None
    }
    assert bn1.find_regulation('a', 'c') == {
        'source': VariableId(0),
        'target': VariableId(2),
        'essential': True,
        'sign': '-',
    }

    bn1e = bn1.extend(['d', 'e'])
    bn1e.add_regulation('c -> d')
    bn1e.add_regulation('e -| b')
    assert bn1e != bn1
    assert bn1e.find_variable('e') == VariableId(4)

    assert bn1e.drop(['d', 'e']) == bn1

    # For inlining, the results are actually different compared to normal regulatory graph, because inlining
    # can introduce inlined variables as parameters and give names to anonymous functions. Hence, we only compare
    # the RG here.
    assert bn1e.inline_variable('d').inline_variable('e').to_graph() == bn1.to_graph()
    # Keep in mind that rg1 is now extended with 'a -| c'
    assert bn1.inline_variable('c').to_graph() == BooleanNetwork(None, ["a -> b", "b -?? a", "a -? a"]).to_graph()

    assert bn1.predecessors('c') == {VariableId(0), VariableId(1)}
    assert bn1.successors('a') == {VariableId(1), VariableId(2)}

    assert bn1e.backward_reachable('d') == set(bn1e.variables())
    assert bn1e.forward_reachable('d') == {VariableId(3)}
    assert bn1e.backward_reachable('e') == {VariableId(4)}
    assert bn1e.forward_reachable('e') == set(bn1e.variables())
    assert bn1e.backward_reachable(['d', 'e']) == set(bn1e.variables())
    assert bn1e.forward_reachable(['d', 'e']) == set(bn1e.variables())

    # FVS and IC are very simple, since there are effectively just two cycles of very ambiguous monotonicity.
    assert bn1.feedback_vertex_set() == {VariableId(0)}
    assert len(bn1.feedback_vertex_set()) == len(bn1e.feedback_vertex_set())
    assert bn1.feedback_vertex_set(parity='+') == bn1.feedback_vertex_set()
    assert bn1.feedback_vertex_set(parity='-') == bn1.feedback_vertex_set()
    assert bn1.feedback_vertex_set(subgraph=['a', 'b']) == set()

    assert bn1.independent_cycles() == [[VariableId(0), VariableId(2)]]
    assert bn1.independent_cycles() == bn1e.independent_cycles()
    assert bn1.independent_cycles(parity='+') == bn1.independent_cycles(parity='-')
    assert bn1.independent_cycles(subgraph=['a', 'b']) == []

    assert bn1.strongly_connected_components() == bn1e.strongly_connected_components()
    assert bn1.strongly_connected_components(subgraph=['a', 'b']) == []

    assert bn1.weakly_connected_components() == [set(bn1.variables())]
    assert bn1e.weakly_connected_components(subgraph=['d', 'e']) == [{VariableId(3)}, {VariableId(4)}]

    assert bn1.shortest_cycle('a') == [VariableId(0), VariableId(2)]
    assert bn1.shortest_cycle('b') == [VariableId(1), VariableId(2), VariableId(0)]
    assert bn1.shortest_cycle('a', length=1) is None
    assert bn1.shortest_cycle('a', subgraph=['a', 'b']) is None
    assert bn1.shortest_cycle('a', parity='+') == bn1.shortest_cycle('a', parity='-')


def test_boolean_network():
    """
    This second test actually updates things that are "new" in a `BooleanNetwork` compared to a `RegulatoryGraph`.
    """
    bn1 = BooleanNetwork(variables=["a", "b", "c"])
    bn1.add_regulation("a -> b")
    bn1.add_regulation("b -?? c")
    bn1.add_regulation("c -|? b")
    bn1.add_explicit_parameter("f", 1)
    bn1.set_update_function("b", "a | f(c)")

    bn2 = BooleanNetwork(
        regulations=["a -> b", "b -?? c", "c -|? b"],
        parameters=[("f", 1)],
        functions={"b": "a | f(c)"}
    )

    rg = RegulatoryGraph(
        variables=["a", "b", "c"],
        regulations=["a -> b", "b -?? c", "c -|? b"]
    )

    assert bn1 == bn2
    assert bn1.to_graph() == BooleanNetwork(rg).to_graph()

    assert str(bn1) == "BooleanNetwork(variables=3, regulations=3, explicit_parameters=1, implicit_parameters=2)"
    assert bn1 == eval(repr(bn1))
    assert bn1 == pickle.loads(pickle.dumps(bn1))
    assert bn1 == copy.copy(bn1)

    assert bn1 == BooleanNetwork.from_aeon(bn1.to_aeon())
    Path("tmp.aeon").write_text(bn1.to_aeon())
    assert bn1 == BooleanNetwork.from_file("tmp.aeon")
    Path("tmp.aeon").unlink()
    assert bn1 == BooleanNetwork.from_sbml(bn1.to_sbml())
    Path("tmp.sbml").write_text(bn1.to_sbml())
    assert bn1 == BooleanNetwork.from_file("tmp.sbml")
    Path("tmp.sbml").unlink()
    with pytest.raises(RuntimeError):
        # Cannot export due to parameters.
        bn1.to_bnet()

    bn3 = BooleanNetwork(
        regulations=["a -> b", "b -| a", "b -> b"],
        functions=["!b", "a & b"]
    )

    assert bn3 == BooleanNetwork.from_bnet(bn3.to_bnet(), repair_graph=True)
    Path("tmp.bnet").write_text(bn3.to_bnet())
    assert bn3 == BooleanNetwork.from_file("tmp.bnet", repair_graph=True)
    Path("tmp.bnet").unlink()

    bn1i = bn1.inline_variable("b")
    assert bn1i == BooleanNetwork(
        ["a", "c"],
        ["a -?? c", "c -?? c"],
        [("f", 1), ("f_c", 1)],
        [None, "f_c(a | f(c))"]
    )

    assert bn1.to_graph() == RegulatoryGraph(regulations=["a -> b", "b -?? c", "c -|? b"])

    assert bn1.explicit_parameter_count() == 1
    assert bn1.implicit_parameter_count() == 2
    assert bn1.explicit_parameters() == {ParameterId(0): 1}
    assert bn1.implicit_parameters() == {VariableId(0): 0, VariableId(2): 1}
    assert bn1.explicit_parameter_names() == ["f"]
    assert bn1.get_explicit_parameter_name(ParameterId(0)) == "f"
    assert bn1.get_explicit_parameter_arity("f") == 1
    assert bn1.find_explicit_parameter("f") == ParameterId(0)
    assert bn1.find_explicit_parameter(ParameterId(0)) == ParameterId(0)
    assert bn1.find_explicit_parameter("g") is None
    assert bn1.find_explicit_parameter(ParameterId(2)) is None

    bn1.add_explicit_parameter("g", 2)
    assert bn1.find_explicit_parameter("g") == ParameterId(1)
    bn1 = bn1.prune_unused_parameters()
    assert bn1.find_explicit_parameter("g") is None

    assert str(bn1.get_update_function("b")) == "a | f(c)"
    assert bn1.get_update_function("c") is None

    bn1.set_update_function("c", "true")
    assert str(bn1.get_update_function("c")) == "true"
    bn1.set_update_function("c", None)
    assert bn1.get_update_function("c") is None

    bn1.set_update_function("c", "b")
    bn1x = bn1.infer_valid_graph()
    assert bn1x.find_regulation("b", "c") == {
        'source': VariableId(1),
        'target': VariableId(2),
        'sign': '+',
        'essential': True,
    }

    for reg in bn1x.remove_regulation_constraints().regulations():
        assert not reg['essential']
        assert reg['sign'] is None

    assert bn1.is_variable_input("a")
    assert not bn1.is_variable_input("b")
    assert bn1.inputs() == [VariableId(0)]
    assert bn1.input_names() == ["a"]
    bn1x = bn1.inline_inputs()
    assert bn1x.explicit_parameter_names() == ["f", "a"]
    assert bn1x.find_explicit_parameter("a") == ParameterId(1)

    bn1.set_update_function("a", "false")
    assert bn1.is_variable_constant("a") == False
    assert bn1.is_variable_constant("b") is None
    assert bn1.constants() == { VariableId(0): False }
    assert bn1.constant_names() == { "a": False }
    bn1x = bn1.inline_constants()
    assert bn1x.variable_names() == ["b", "c"]
    assert str(bn1x.get_update_function("b")) == "f(c)"

    bn1.set_update_function("a", None)
    bn1.set_update_function("c", None)
    bn1named = bn1.name_implicit_parameters()
    assert bn1named.implicit_parameter_count() == 0
    assert bn1named.explicit_parameter_names() == ["f", "f_a", "f_c"]

    bn1.assign_parameter_name("a")
    bn1.assign_parameter_name("c", "foo_c")
    assert bn1.implicit_parameter_count() == 0
    assert bn1.explicit_parameter_names() == ["f", "f_a", "foo_c"]

    with pytest.raises(RuntimeError):
        bn1.assign_parameter_name("a") # Variable has no implicit parameter.


def test_update_function():
    bn1 = BooleanNetwork(variables=["a", "b", "c"], parameters=[("f", 1), ("g", 2)])
    bn2 = BooleanNetwork(variables=["b", "c", "d"], parameters=[("g", 1), ("h", 2)])

    a = UpdateFunction(bn1, "a")
    b = UpdateFunction(bn1, "b")
    expr = UpdateFunction(bn1, "(a & b) | g(b, !c)")

    assert str(expr) == "(a & b) | g(b, !c)"
    assert expr == eval(repr(expr))

    assert "a" in expr and "b" in expr and "c" in expr
    assert "f" not in expr and "g" in expr

    d = {a: "foo", b: "bar"}
    assert d[a] == "foo"
    assert d[a] != d[b]

    assert expr == pickle.loads(pickle.dumps(expr))

    expr_inner = expr.as_binary()
    assert expr_inner is not None
    op, l, r = expr_inner
    assert op == "or"
    assert l.__root__() == r.__root__() == expr
    assert l.__ctx__() == r.__ctx__() == expr.__ctx__()

    assert UpdateFunction.mk_const(bn1, 0) == UpdateFunction.mk_const(bn2, False)
    assert UpdateFunction.mk_const(bn1, 0) != UpdateFunction.mk_const(bn2, True)
    assert UpdateFunction(bn1, "a") == UpdateFunction.mk_var(bn1, "a")
    assert UpdateFunction(bn1, "g(a, b)") == UpdateFunction.mk_param(bn1, "g", ["a", "b"])
    assert UpdateFunction(bn1, "g(a, b)") == UpdateFunction.mk_param(bn1, ParameterId(1),
                                                                     [VariableId(0), VariableId(1)])
    assert UpdateFunction(bn1, "!a") == UpdateFunction.mk_not(a)
    assert UpdateFunction(bn1, "a & b") == UpdateFunction.mk_and(a, b)
    assert UpdateFunction(bn1, "a | b") == UpdateFunction.mk_or(a, b)
    assert UpdateFunction(bn1, "a => b") == UpdateFunction.mk_imp(a, b)
    assert UpdateFunction(bn1, "a <=> b") == UpdateFunction.mk_iff(a, b)
    assert UpdateFunction(bn1, "a ^ b") == UpdateFunction.mk_xor(a, b)
    assert UpdateFunction(bn1, "a ^ b") == UpdateFunction.mk_binary("xor", a, b)
    assert UpdateFunction(bn1, "a & b & c") == UpdateFunction.mk_conjunction(bn1, ["a", "b", "c"])
    assert UpdateFunction(bn1, "a | b | c") == UpdateFunction.mk_disjunction(bn1, ["a", "b", "c"])

    assert UpdateFunction(bn1, "true").is_const() and not UpdateFunction(bn1, "a").is_const()
    assert UpdateFunction(bn1, "a").is_var() and not UpdateFunction(bn1, "true").is_var()
    assert UpdateFunction(bn1, "g(a, b)").is_param() and not UpdateFunction(bn1, "a").is_param()
    assert UpdateFunction(bn1, "!a").is_not() and not UpdateFunction(bn1, "a").is_not()
    assert UpdateFunction(bn1, "a & b").is_and() and not UpdateFunction(bn1, "a | b").is_and()
    assert UpdateFunction(bn1, "a | b").is_or() and not UpdateFunction(bn1, "a & b").is_or()
    assert UpdateFunction(bn1, "a => b").is_imp() and not UpdateFunction(bn1, "a & b").is_imp()
    assert UpdateFunction(bn1, "a <=> b").is_iff() and not UpdateFunction(bn1, "a & b").is_iff()
    assert UpdateFunction(bn1, "a ^ b").is_xor() and not UpdateFunction(bn1, "a & b").is_xor()
    assert UpdateFunction(bn1, "a").is_literal() and UpdateFunction(bn1, "!a").is_literal()
    assert UpdateFunction(bn1, "a & b").is_binary() and not UpdateFunction(bn1, "!a").is_binary()

    assert UpdateFunction(bn1, "true").as_const()
    assert UpdateFunction(bn1, "a").as_var() == VariableId(0)
    assert UpdateFunction(bn1, "!a").as_var() is None
    assert UpdateFunction(bn1, "g(a, b)").as_param() == (ParameterId(1), [a, b])
    assert UpdateFunction(bn1, "!a").as_not() == a
    assert UpdateFunction(bn1, "a").as_not() is None
    assert UpdateFunction(bn1, "a & b").as_and() == (a, b)
    assert UpdateFunction(bn1, "a | b").as_and() is None
    assert UpdateFunction(bn1, "a | b").as_or() == (a, b)
    assert UpdateFunction(bn1, "a & b").as_or() is None
    assert UpdateFunction(bn1, "a => b").as_imp() == (a, b)
    assert UpdateFunction(bn1, "a & b").as_imp() is None
    assert UpdateFunction(bn1, "a <=> b").as_iff() == (a, b)
    assert UpdateFunction(bn1, "a & b").as_iff() is None
    assert UpdateFunction(bn1, "a ^ b").as_xor() == (a, b)
    assert UpdateFunction(bn1, "a & b").as_xor() is None
    assert UpdateFunction(bn1, "a").as_literal() == (VariableId(0), True)
    assert UpdateFunction(bn1, "!a").as_literal() == (VariableId(0), False)
    assert UpdateFunction(bn1, "!!a").as_literal() is None
    assert UpdateFunction(bn1, "a & b").as_binary() == ("and", a, b)
    assert UpdateFunction(bn1, "a").as_binary() is None

    assert expr.support_variables() == {VariableId(0), VariableId(1), VariableId(2)}
    assert expr.support_parameters() == {ParameterId(1)}

    assert expr.substitute_all({"b": "f(b & a)"}) == UpdateFunction(bn1, "(a & f(b & a)) | g(f(b & a), !c)")
    assert expr.rename_all(bn2, {'a': 'd', 'b': 'c', 'c': 'b'}, {'g': 'h'}) == UpdateFunction(bn2, "(d & c) | h(c, !b)")
    assert expr.substitute_all({"b": "true"}).simplify_constants() == UpdateFunction(bn1, "a | g(true, !c)")
    assert UpdateFunction(bn1, "!(a & b)").distribute_negation() == UpdateFunction(bn1, "!a | !b")
    assert UpdateFunction(bn1, "a ^ b").to_and_or_normal_form() == UpdateFunction(bn1, "(a | b) & !(a & b)")
    assert UpdateFunction(bn1, "a <=> b").to_and_or_normal_form() == UpdateFunction(bn1, "(a & b) | (!a & !b)")


def test_model_annotation():
    ann = ModelAnnotation()
    desc = ann['description']
    assert desc.value is None
    desc.value = "Multiline\n"
    desc.value += "Test description"
    desc['x'].value = "Variable X"
    desc['y'].value = "Variable Y"

    assert len(desc) == 2
    assert ann['description']['x'].value == "Variable X"

    properties = ModelAnnotation('Required model properties.')
    properties['p_1'].value = "Property:1"
    properties['p_2'].value = "Property:2"
    ann['properties'] = properties

    assert ann['properties'].value == "Required model properties."
    assert ann['properties']['p_1'].value == "Property:1"
    assert ann['properties']['p_2'].value == "Property:2"

    del ann['properties']['p_1']
    assert 'p_1' not in ann['properties']

    assert ann == copy.copy(ann)
    assert str(ann) == str(eval(repr(ann)))
    assert str(ann) == str(ModelAnnotation.from_aeon(str(ann)))
    assert copy.deepcopy(ann) != ann

    assert str(ann).strip() == """
#!description:Multiline
#!description:Test description
#!description:x:Variable X
#!description:y:Variable Y
#!properties:Required model properties.
#!properties:p_2:#`Property:2`#
    """.strip()

    Path('tmp.aeon').write_text(str(ann))
    assert str(ann) == str(ModelAnnotation.from_file('tmp.aeon'))
    Path('tmp.aeon').unlink()

    assert ann['description'].values() == [ann['description']['x'], ann['description']['y']]
    assert ann['description'].keys() == ['x', 'y']
    assert ann['description'].items() == [('x', ann['description']['x']), ('y', ann['description']['y'])]


def test_symbolic_context():
    bn = BooleanNetwork(
        variables=["a", "b", "c"],
        regulations=["a -> b", "c -| b", "a -> a", "b -| c"],
        parameters=[("f", 2)],
        functions=[None, "a | f(a, c)", None]
    )

    ctx = SymbolicContext(bn, {'a': 4, 'b': 2})
    ctx_c = ctx.to_canonical_context()
    bdd_vars = ctx.bdd_variable_set()

    assert ctx.to_canonical_context() == ctx.to_canonical_context()
    assert ctx != ctx_c

    assert str(ctx) == ("SymbolicContext(network_variables=3, extra_variables=6, explicit_functions=1, "
                        "implicit_functions=2)")

    assert ctx == copy.copy(ctx)
    assert ctx == copy.deepcopy(ctx)

    assert ctx.network_variable_count() == 3
    assert ctx.network_variable_names() == ["a", "b", "c"]
    assert ctx.network_variables() == [VariableId(x) for x in range(3)]
    # This is just hand computed.
    assert ctx.network_bdd_variables() == [BddVariable(0), BddVariable(7), BddVariable(14)]
    assert ctx.find_network_variable("b") == VariableId(1)
    assert ctx.find_network_variable(VariableId(4)) is None
    assert ctx.find_network_variable(BddVariable(14)) == VariableId(2)

    assert ctx.find_network_bdd_variable("b") == BddVariable(7)
    assert ctx.find_network_bdd_variable(VariableId(4)) is None

    with pytest.raises(IndexError):
        ctx.get_network_variable_name("x")

    assert ctx.extra_bdd_variable_count() == 6
    assert ctx.extra_bdd_variables_list() == [BddVariable(x) for x in [1, 2, 3, 4, 8, 9]]
    assert ctx.extra_bdd_variables() == {
        VariableId(0): [BddVariable(x) for x in [1, 2, 3, 4]],
        VariableId(1): [BddVariable(x) for x in [8, 9]],
    }

    assert ctx.explicit_function_count() == 1
    assert ctx.explicit_functions() == [ParameterId(0)]
    assert ctx.explicit_functions_bdd_variables_list() == [BddVariable(x) for x in [10, 11, 12, 13]]
    assert ctx.explicit_functions_bdd_variables() == {ParameterId(0): [BddVariable(x) for x in [10, 11, 12, 13]]}

    assert ctx.implicit_function_count() == 2
    assert ctx.implicit_functions() == [VariableId(0), VariableId(2)]
    assert ctx.implicit_functions_bdd_variables_list() == [BddVariable(x) for x in [5, 6, 15, 16]]
    assert ctx.implicit_functions_bdd_variables() == {
        VariableId(0): [BddVariable(x) for x in [5, 6]],
        VariableId(2): [BddVariable(x) for x in [15, 16]],
    }

    assert ctx.function_count() == 3
    assert ctx.functions() == [ParameterId(0), VariableId(0), VariableId(2)]
    assert ctx.functions_bdd_variables_list() == [BddVariable(x) for x in [5, 6, 10, 11, 12, 13, 15, 16]]
    assert ctx.functions_bdd_variables() == {
        ParameterId(0): [BddVariable(x) for x in [10, 11, 12, 13]],
        VariableId(0): [BddVariable(x) for x in [5, 6]],
        VariableId(2): [BddVariable(x) for x in [15, 16]],
    }

    assert ctx.find_function("f") == ParameterId(0)
    assert ctx.find_function("a") == VariableId(0)
    assert ctx.find_function(ParameterId(1)) is None
    assert ctx.find_function(VariableId(4)) is None
    assert ctx.find_function(BddVariable(12)) == ParameterId(0)
    assert ctx.find_function(BddVariable(15)) == VariableId(2)
    assert ctx.find_function(BddVariable(7)) is None

    assert ctx.get_function_name(VariableId(0)) == "a"
    assert ctx.get_function_name(ParameterId(0)) == "f"
    with pytest.raises(IndexError):
        ctx.get_function_name("b")

    assert ctx.get_function_arity(VariableId(0)) == 1
    assert ctx.get_function_arity("f") == 2

    assert ctx.get_function_table("f") == [
        ([False, False], BddVariable(10)),
        ([True, False], BddVariable(11)),
        ([False, True], BddVariable(12)),
        ([True, True], BddVariable(13)),
    ]
    assert ctx.get_function_table("a") == [
        ([False], BddVariable(5)),
        ([True], BddVariable(6)),
    ]

    assert ctx.mk_constant(1) == bdd_vars.mk_const(1)
    assert ctx.mk_network_variable("a") == bdd_vars.mk_literal("a", True)
    assert ctx.mk_extra_bdd_variable("a", 1) == bdd_vars.mk_literal(BddVariable(2), True)
    a = ctx.mk_network_variable("a")
    c = ctx.mk_network_variable("c")
    fn_b = bn.get_update_function("b")
    assert fn_b is not None
    assert ctx.mk_update_function(fn_b) == ctx.mk_function("f", [a, c]).l_or(a)

    fn = ctx.mk_update_function(fn_b)
    assert ctx_c.transfer_from(fn, ctx) != fn

    ctx_e = ctx.eliminate_network_variable("a")
    assert ctx_e != ctx
    assert ctx_e.bdd_variable_set() == ctx.bdd_variable_set()
    assert ctx.transfer_from(c, ctx) == ctx.mk_network_variable("c")
    assert len(ctx_e.functions_bdd_variables_list()) == len(ctx.functions_bdd_variables_list())
    # The implicit function of `a` got transformed into an explicit parameter.
    assert len(ctx_e.functions_bdd_variables()) == len(ctx.functions_bdd_variables())
    assert ctx_e.find_function('a') is not None
    assert ctx.find_function('a') is not None


def test_symbolic_space_context():
    # First the overloads, the new things are at the end of the method.
    bn = BooleanNetwork(
        variables=["a", "b", "c"],
        regulations=["a -> b", "c -| b", "a -> a", "b -| c"],
        parameters=[("f", 2)],
        functions=[None, "a | f(a, c)", None]
    )

    ctx = SymbolicSpaceContext(bn)
    ctx_c = ctx.to_canonical_context()
    bdd_vars = ctx.bdd_variable_set()

    assert ctx.to_canonical_context() == ctx.to_canonical_context()
    assert ctx != ctx_c

    assert str(ctx) == ("SymbolicSpaceContext(network_variables=3, space_variables=6, explicit_functions=1, "
                        "implicit_functions=2)")

    assert ctx == copy.copy(ctx)
    assert ctx == copy.deepcopy(ctx)

    assert ctx.network_variable_count() == 3
    assert ctx.network_variable_names() == ["a", "b", "c"]
    assert ctx.network_variables() == [VariableId(x) for x in range(3)]
    # This is just hand computed.
    assert ctx.network_bdd_variables() == [BddVariable(0), BddVariable(5), BddVariable(12)]
    assert ctx.find_network_variable("b") == VariableId(1)
    assert ctx.find_network_variable(VariableId(4)) is None
    assert ctx.find_network_variable(BddVariable(12)) == VariableId(2)

    assert ctx.find_network_bdd_variable("b") == BddVariable(5)
    assert ctx.find_network_bdd_variable(VariableId(4)) is None

    with pytest.raises(IndexError):
        ctx.get_network_variable_name("x")

    assert ctx.extra_bdd_variable_count() == 6
    assert ctx.extra_bdd_variables_list() == [BddVariable(x) for x in [1, 2, 6, 7, 13, 14]]
    assert ctx.extra_bdd_variables() == {
        VariableId(0): [BddVariable(x) for x in [1, 2]],
        VariableId(1): [BddVariable(x) for x in [6, 7]],
        VariableId(2): [BddVariable(x) for x in [13, 14]],
    }

    assert ctx.explicit_function_count() == 1
    assert ctx.explicit_functions() == [ParameterId(0)]
    assert ctx.explicit_functions_bdd_variables_list() == [BddVariable(x) for x in [8, 9, 10, 11]]
    assert ctx.explicit_functions_bdd_variables() == {ParameterId(0): [BddVariable(x) for x in [8, 9, 10, 11]]}

    assert ctx.implicit_function_count() == 2
    assert ctx.implicit_functions() == [VariableId(0), VariableId(2)]
    assert ctx.implicit_functions_bdd_variables_list() == [BddVariable(x) for x in [3, 4, 15, 16]]
    assert ctx.implicit_functions_bdd_variables() == {
        VariableId(0): [BddVariable(x) for x in [3, 4]],
        VariableId(2): [BddVariable(x) for x in [15, 16]],
    }

    assert ctx.function_count() == 3
    assert ctx.functions() == [ParameterId(0), VariableId(0), VariableId(2)]
    assert ctx.functions_bdd_variables_list() == [BddVariable(x) for x in [3, 4, 8, 9, 10, 11, 15, 16]]
    assert ctx.functions_bdd_variables() == {
        ParameterId(0): [BddVariable(x) for x in [8, 9, 10, 11]],
        VariableId(0): [BddVariable(x) for x in [3, 4]],
        VariableId(2): [BddVariable(x) for x in [15, 16]],
    }

    assert ctx.find_function("f") == ParameterId(0)
    assert ctx.find_function("a") == VariableId(0)
    assert ctx.find_function(ParameterId(1)) is None
    assert ctx.find_function(VariableId(4)) is None
    assert ctx.find_function(BddVariable(10)) == ParameterId(0)
    assert ctx.find_function(BddVariable(15)) == VariableId(2)
    assert ctx.find_function(BddVariable(7)) is None

    assert ctx.get_function_name(VariableId(0)) == "a"
    assert ctx.get_function_name(ParameterId(0)) == "f"
    with pytest.raises(IndexError):
        ctx.get_function_name("b")

    assert ctx.get_function_arity(VariableId(0)) == 1
    assert ctx.get_function_arity("f") == 2

    assert ctx.get_function_table("f") == [
        ([False, False], BddVariable(8)),
        ([True, False], BddVariable(9)),
        ([False, True], BddVariable(10)),
        ([True, True], BddVariable(11)),
    ]
    assert ctx.get_function_table("a") == [
        ([False], BddVariable(3)),
        ([True], BddVariable(4)),
    ]

    assert ctx.mk_constant(1) == bdd_vars.mk_const(1)
    assert ctx.mk_network_variable("a") == bdd_vars.mk_literal("a", True)
    assert ctx.mk_extra_bdd_variable("a", 1) == bdd_vars.mk_literal(BddVariable(2), True)
    a = ctx.mk_network_variable("a")
    c = ctx.mk_network_variable("c")
    fn_b = bn.get_update_function("b")
    assert fn_b is not None
    assert ctx.mk_update_function(fn_b) == ctx.mk_function("f", [a, c]).l_or(a)

    fn = ctx.mk_update_function(fn_b)
    assert ctx_c.transfer_from(fn, ctx) != fn

    ctx_e = ctx.eliminate_network_variable("a")
    assert isinstance(ctx_e, SymbolicSpaceContext)
    assert ctx_e != ctx
    assert ctx_e.bdd_variable_set() == ctx.bdd_variable_set()
    assert ctx.transfer_from(c, ctx) == ctx.mk_network_variable("c")
    assert len(ctx_e.functions_bdd_variables_list()) == len(ctx.functions_bdd_variables_list())
    # The implicit function of `a` got transformed into an explicit parameter.
    assert len(ctx_e.functions_bdd_variables()) == len(ctx.functions_bdd_variables())
    assert ctx_e.find_function('a') is not None
    assert ctx.find_function('a') is not None

    unit = ctx.mk_unit_bdd()
    unit_support = unit.support_set()
    assert len(unit_support) == ctx.network_variable_count() * 2
    for var in ctx.network_variables():
        assert ctx.get_positive_space_variable(var) in unit_support
        assert ctx.get_negative_space_variable(var) in unit_support

    stg = AsynchronousGraph(bn, ctx)
    assert ctx.mk_empty_spaces().is_empty()
    assert ctx.mk_empty_colored_spaces().is_empty()
    assert ctx.mk_unit_spaces().cardinality() == 27
    assert ctx.mk_unit_colored_spaces(stg).spaces() == ctx.mk_unit_spaces()

    space = ctx.mk_singleton({'a': True, 'c': False})
    c_space = ctx.mk_unit_colored_spaces(stg).intersect_spaces(space)
    assert ctx.mk_sub_spaces(space).cardinality() == 3
    assert ctx.mk_sub_spaces(space) == space.with_all_sub_spaces()
    assert ctx.mk_sub_spaces(c_space) == c_space.with_all_sub_spaces()
    assert ctx.mk_super_spaces(space).cardinality() == 4
    assert ctx.mk_super_spaces(space) == space.with_all_super_spaces()
    assert ctx.mk_super_spaces(c_space) == c_space.with_all_super_spaces()

    # We don't really know what the result should be here, but it should not be a constant.
    fn_b_bdd = ctx.mk_update_function(fn_b)
    assert not ctx.mk_can_go_to_true(fn_b_bdd).is_false() and not ctx.mk_can_go_to_true(fn_b_bdd).is_true()

    assert ctx.mk_unit_spaces().extend_with_colors(stg.mk_unit_colors()) == ctx.mk_unit_colored_spaces(stg)
    assert stg.mk_unit_colors().extend_with_spaces(ctx.mk_unit_spaces()) == ctx.mk_unit_colored_spaces(stg)


def test_asynchronous_graph():
    bn = BooleanNetwork.from_aeon("""
    a -> b
    b -|? c
    c -?? b
    c -| a
    $b: a & f(c)    
    """)

    custom_ctx = SymbolicContext(bn, {"c": 3})
    custom_unit = custom_ctx.mk_network_variable("c")

    graph = AsynchronousGraph(bn, context=custom_ctx, unit_bdd=custom_unit)

    assert str(graph) == f"AsynchronousGraph({custom_ctx})"
    assert graph.symbolic_context() == custom_ctx

    assert graph.network_variable_count() == 3
    assert graph.network_variable_names() == ["a", "b", "c"]
    assert graph.network_variables() == [VariableId(x) for x in [0, 1, 2]]
    assert graph.find_network_variable("a") == VariableId(0)
    assert graph.get_network_variable_name(VariableId(1)) == "b"

    empty_set = graph.mk_empty_colored_vertices()
    empty_colors = graph.mk_empty_colors()
    empty_vertices = graph.mk_empty_vertices()

    unit_set = graph.mk_unit_colored_vertices()
    unit_colors = graph.mk_unit_colors()
    unit_vertices = graph.mk_unit_vertices()

    assert empty_set.cardinality() == 0
    assert empty_colors.cardinality() == 0
    assert empty_vertices.cardinality() == 0

    # The unit BDD is smaller than the whole state space.
    assert unit_vertices.cardinality() == 4
    assert unit_colors.cardinality() == 9
    assert unit_set.cardinality() == 36

    assert empty_set.intersect(unit_set).is_empty()
    assert empty_colors.intersect(unit_colors).is_empty()
    assert empty_vertices.intersect(unit_vertices).is_empty()

    assert empty_set.union(unit_set) == unit_set
    assert empty_colors.union(unit_colors) == unit_colors
    assert empty_vertices.union(unit_vertices) == unit_vertices

    assert unit_set.minus(empty_set) == unit_set
    assert unit_colors.minus(empty_colors) == unit_colors
    assert unit_vertices.minus(empty_vertices) == unit_vertices

    assert unit_set.minus_colors(unit_colors).is_empty()
    assert unit_set.minus_vertices(unit_vertices).is_empty()
    assert unit_set.intersect_colors(unit_colors) == unit_set
    assert unit_set.intersect_vertices(unit_vertices) == unit_set

    assert graph.mk_function_colors("f", "true").is_subspace()
    assert graph.mk_function_colors("a", "x_0").intersect(graph.mk_function_colors("a", "!x_0")).is_empty()

    assert graph.mk_function_colors("f", "true").is_subset(graph.mk_function_row_colors("f", [0], True))
    assert graph.mk_function_colors("a", "x_0").is_subset(graph.mk_function_row_colors("a", [1], True))

    assert graph.transfer_from(unit_set, graph) == unit_set
    assert graph.transfer_from(unit_colors, graph) == unit_colors
    assert graph.transfer_from(unit_vertices, graph) == unit_vertices

    assert graph.mk_update_function("a") == custom_ctx.mk_function("a", ["c"])

    space_arg: dict[str, Literal[0, 1]] = {"a": 0, "b": 1}
    space = graph.mk_subspace(space_arg)

    space_vertices_arg: dict[str, Literal[0, 1]] = {"a": 0, "b": 1, "c": 1}
    assert space.vertices() == graph.mk_subspace_vertices(space_vertices_arg)

    assert (graph.post(space) == graph.var_post("a", space)
            .union(graph.var_post("b", space))
            .union(graph.var_post("c", space)))

    assert (graph.pre(space) == graph.var_pre("a", space)
            .union(graph.var_pre("b", space))
            .union(graph.var_pre("c", space)))

    def union_all(items):
        return reduce(lambda x, y: x.union(y), items)

    assert graph.post(space) == union_all([graph.var_post(var, space) for var in graph.network_variables()])
    assert graph.pre(space) == union_all([graph.var_pre(var, space) for var in graph.network_variables()])

    for var in graph.network_variables():
        assert graph.var_post(var, space) == graph.var_post_out(var, space).union(graph.var_post_within(var, space))
        assert graph.var_pre(var, space) == graph.var_pre_out(var, space).union(graph.var_pre_within(var, space))

    assert graph.can_post(space) == union_all([graph.var_can_post(var, space) for var in graph.network_variables()])
    assert graph.can_pre(space) == union_all([graph.var_can_pre(var, space) for var in graph.network_variables()])

    for var in graph.network_variables():
        can_post = graph.var_can_post_out(var, space).union(graph.var_can_post_within(var, space))
        assert graph.var_can_post(var, space) == can_post
        can_pre = graph.var_can_pre_out(var, space).union(graph.var_can_pre_within(var, space))
        assert graph.var_can_pre(var, space) == can_pre

    bn = BooleanNetwork.from_aeon("""
        a -> b
        b -| c
        c -> b
        c -| a
        $a: !c
        $b: a & c
        $c: !b
    """)
    stg = AsynchronousGraph(bn)
    assert stg.reconstruct_network() == bn

    assert stg.mk_unit_vertices().extend_with_colors(stg.mk_unit_colors()) == stg.mk_unit_colored_vertices()
    assert stg.mk_unit_colors().extend_with_vertices(stg.mk_unit_vertices()) == stg.mk_unit_colored_vertices()


def test_symbolic_iterators():
    bn = BooleanNetwork.from_aeon("""
    a -> b
    b -|? c
    c -?? b
    c -| a
    $b: a & f(c)    
    """)

    graph = AsynchronousGraph(bn)

    b_space: dict[str, Literal[0, 1]] = {'b': 1}
    c_space: dict[str, Literal[0, 1]] = {'c': 1}
    b_or_c = graph.mk_subspace_vertices(b_space).union(graph.mk_subspace_vertices(c_space))

    for vertex in b_or_c:
        assert vertex["b"] or vertex[VariableId(2)]
        assert "a" in vertex
        assert vertex.keys() == [VariableId(x) for x in range(3)]
        assert vertex["b"] == vertex.values()[1]
        assert vertex.items()[0] == (VariableId(0), vertex[VariableId(0)])
        assert vertex.to_dict() == dict(vertex.items())
        assert vertex.to_symbolic().is_singleton()
        assert vertex.to_symbolic() == graph.mk_subspace_vertices(vertex.to_dict())

        # For a single vertex, the enclosing subspace should be equal to the vertex.
        subspace = vertex.to_symbolic().enclosing_subspace()
        named_subspace = vertex.to_symbolic().enclosing_named_subspace()
        assert subspace is not None
        assert named_subspace is not None
        assert vertex.to_symbolic() == graph.mk_subspace_vertices(subspace)
        assert vertex.to_symbolic() == graph.mk_subspace_vertices(named_subspace)

    assert sum(1 for _ in b_or_c) == 6

    for vertex in b_or_c.items(["b", "c"]):
        assert vertex["b"] or vertex["c"]
        assert "a" not in vertex
        with pytest.raises(IndexError):
            assert vertex["a"]
        assert vertex.keys() == [VariableId(1), VariableId(2)]
        assert vertex["b"] == vertex.values()[0]
        assert vertex.items()[1] == (VariableId(2), vertex[VariableId(2)])
        assert vertex.to_dict() == dict(vertex.items())
        assert vertex.to_symbolic().is_subspace()
        assert vertex.to_symbolic() == graph.mk_subspace_vertices(vertex.to_dict())

    assert sum(1 for _ in b_or_c.items(["b", "c"])) == 3

    assert str(next(iter(b_or_c))) == "VertexModel({'a': 0, 'b': 0, 'c': 1})"

    unit_colors = graph.mk_unit_colors()

    f_expr = [str(i["f"]) for i in unit_colors.items(["f"])]
    assert set(f_expr) == {"true", "x_0", "!x_0"}

    a_expr = [str(i["a"]) for i in unit_colors.items(["a"])]
    assert set(a_expr) == {"!x_0"}

    c_expr = [str(i["c"]) for i in unit_colors.items(["c"])]
    assert set(c_expr) == {"true", "false", "!x_0"}

    assert sum(1 for _ in unit_colors) == unit_colors.cardinality()

    for i in unit_colors:
        i_bn = i.instantiate(bn, infer_regulations=False)
        assert i_bn.variable_count() == bn.variable_count()
        assert i_bn.regulation_count() == bn.regulation_count()
        assert i_bn.implicit_parameter_count() == 0 and i_bn.explicit_parameter_count() == 0
        assert i_bn.infer_valid_graph() != i_bn
        assert i_bn.infer_valid_graph() == i.instantiate(bn, infer_regulations=True)

        fn_b = bn.get_update_function("b")
        assert fn_b is not None
        fn_b = i.instantiate(fn_b)
        assert str(fn_b) in {"a", "a & c", "a & !c"}

        fn_c_in_a = i.instantiate("c", ["a"])
        assert str(fn_c_in_a) in {"true", "false", "!a"}
        fn_c_in_b = i.instantiate("c", ["b"])
        assert str(fn_c_in_b) in {"true", "false", "!b"}

        assert i.to_symbolic().is_singleton()
        assert i.to_symbolic().is_subset(graph.mk_function_colors("f", i["f"]))

    # Instantiation with a subset of networks:
    for i in unit_colors.items(retained=["f"]):
        i_bn = i.instantiate(bn)
        assert i_bn.explicit_parameter_count() == 0
        assert i_bn.implicit_parameter_count() == 2

        fn_b = bn.get_update_function("b")
        assert fn_b is not None
        fn_b = i.instantiate(fn_b)
        assert str(fn_b) in {"a", "a & c", "a & !c"}

    # This is basically a mix of tests for ColorSet and VertexSet

    unit_colored_set = graph.mk_unit_colored_vertices()

    f_expr = [str(i["f"]) for (i, _) in unit_colored_set.items(retained_functions=["f"])]
    assert set(f_expr) == {"true", "x_0", "!x_0"}

    a_expr = [str(i["a"]) for (i, _) in unit_colored_set.items(retained_functions=["a"])]
    assert set(a_expr) == {"!x_0"}

    c_expr = [str(i["c"]) for (i, _) in unit_colored_set.items(retained_functions=["c"])]
    assert set(c_expr) == {"true", "false", "!x_0"}

    a_vals = [v["a"] for (_, v) in unit_colored_set.items(retained_variables=["a"])]
    assert set(a_vals) == {True, False}

    assert sum(1 for _ in unit_colored_set) == unit_colored_set.cardinality()

    for (i, v) in unit_colored_set:
        assert len(v) == 3

        i_bn = i.instantiate(bn)
        assert i_bn.variable_count() == bn.variable_count()
        assert i_bn.regulation_count() == bn.regulation_count()
        assert i_bn.implicit_parameter_count() == 0 and i_bn.explicit_parameter_count() == 0

        fn_b = bn.get_update_function("b")
        assert fn_b is not None
        fn_b = i.instantiate(fn_b)
        assert str(fn_b) in {"a", "a & c", "a & !c"}

        fn_c_in_a = i.instantiate("c", ["a"])
        assert str(fn_c_in_a) in {"true", "false", "!a"}
        fn_c_in_b = i.instantiate("c", ["b"])
        assert str(fn_c_in_b) in {"true", "false", "!b"}

    b_or_c_space = graph.mk_subspace(b_space).union(graph.mk_subspace(c_space))

    for (i, v) in b_or_c_space.items(retained_variables=["b", "c"], retained_functions=["f"]):
        assert "c" not in i
        assert "a" not in i
        assert "a" not in v
        assert "b" in v
        assert "c" in v

        assert v["b"] or v["c"]
        with pytest.raises(IndexError):
            assert v["a"]

        assert v.keys() == [VariableId(1), VariableId(2)]
        assert v["b"] == v.values()[0]
        assert v.items()[1] == (VariableId(2), v[VariableId(2)])
        assert v.to_dict() == dict(v.items())

        fn_b = bn.get_update_function("b")
        assert fn_b is not None
        fn_b = i.instantiate(fn_b)
        assert str(fn_b) in {"a", "a & c", "a & !c"}

    ctx = SymbolicSpaceContext(bn)

    unit_spaces = ctx.mk_unit_spaces()

    assert sum(1 for _ in unit_spaces) == unit_spaces.cardinality()

    for s in unit_spaces.items(["b", "c"]):
        assert "a" not in s
        assert s["b"] in [True, False, None]
        assert s["c"] in [True, False, None]
        with pytest.raises(IndexError):
            assert s["a"]
        assert s.keys() == [VariableId(1), VariableId(2)]
        assert s["b"] == s.values()[0]
        assert s.items()[1] == (VariableId(2), s[VariableId(2)])
        assert s.to_dict() == dict(s.items())
        # Should always contain three spaces with different values of "a".
        assert s.to_symbolic().cardinality() == 3

    unit_colored_spaces = ctx.mk_unit_colored_spaces(AsynchronousGraph(bn, ctx))
    b = ctx.mk_singleton(b_space)
    c = ctx.mk_singleton(c_space)
    colored_spaces = unit_colored_spaces.intersect_spaces(b.union(c))

    assert sum(1 for _ in unit_colored_spaces) == unit_colored_spaces.cardinality()

    for (i, s) in colored_spaces.items(retained_variables=["b", "c"], retained_functions=["f"]):
        assert "c" not in i
        assert "a" not in i
        assert "a" not in s
        assert "b" in s
        assert "c" in s

        assert s["b"] or s["c"]
        with pytest.raises(IndexError):
            assert s["a"]

        assert s.keys() == [VariableId(1), VariableId(2)]
        assert s["b"] == s.values()[0]
        assert s.items()[1] == (VariableId(2), s[VariableId(2)])
        assert s.to_dict() == dict(s.items())

        fn_b = bn.get_update_function("b")
        assert fn_b is not None
        fn_b = i.instantiate(fn_b)
        assert str(fn_b) in {"a", "a & c", "a & !c"}

    # Combining models: (uncovered as a real bug)

    it = graph.mk_unit_colored_vertices().__iter__()
    (m1, _) = it.__next__()
    (_, m2) = it.__next__()
    combined1 = m1.to_symbolic().extend_with_vertices(m2.to_symbolic())
    assert not combined1.is_empty() and combined1.is_singleton()

    it2 = unit_colored_spaces.__iter__()
    (m3, _) = it2.__next__()
    (_, m4) = it2.__next__()
    combined2 = m3.to_symbolic().extend_with_spaces(m4.to_symbolic())
    assert not combined2.is_empty() and combined2.is_singleton()
