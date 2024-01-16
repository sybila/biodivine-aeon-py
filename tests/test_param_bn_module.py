import pytest

from biodivine_aeon import *
import pickle
import copy
from pathlib import Path


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
    })
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
        {'source': VariableId(0), 'target': VariableId(1), 'sign': '+'},
        {'source': VariableId(1), 'target': VariableId(2), 'sign': '-', 'essential': False},
        {'source': VariableId(2), 'target': VariableId(0)},
    ]
    assert rg1.regulation_strings() == ["a -> b", "b -|? c", "c -? a"]
    assert rg1.find_regulation('a', 'c') is None
    assert rg1.find_regulation('c', 'a') == {'source': VariableId(2), 'target': VariableId(0)}
    rg1.add_regulation('a -?? c')
    assert rg1.find_regulation('a', 'c') == {'source': VariableId(0), 'target': VariableId(2), 'essential': False}
    rg1.remove_regulation('a', 'c')
    assert rg1.find_regulation('a', 'c') is None
    assert rg1.ensure_regulation('a -?? c ') is None
    assert rg1.ensure_regulation('a -| c') == {'source': VariableId(0), 'target': VariableId(2), 'essential': False}
    assert rg1.find_regulation('a', 'c') == {'source': VariableId(0), 'target': VariableId(2), 'sign': '-'}

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
    assert rg1.feedback_vertex_set() == rg1e.feedback_vertex_set()
    assert rg1.feedback_vertex_set(parity='+') == rg1.feedback_vertex_set()
    assert rg1.feedback_vertex_set(parity='-') == rg1.feedback_vertex_set()
    assert rg1.feedback_vertex_set(subgraph=['a', 'b']) == set()

    # TODO: Re-enable this once independent cycles are properly deterministic.
    # assert rg1.independent_cycles() == [[VariableId(0), VariableId(2)]]
    # assert rg1.independent_cycles() == rg1e.independent_cycles()
    assert rg1.independent_cycles(parity='+') == rg1.independent_cycles(parity='-')
    assert rg1.independent_cycles(subgraph=['a', 'b']) == []

    assert rg1.strongly_connected_components() == rg1e.strongly_connected_components()
    assert rg1.strongly_connected_components(subgraph=['a', 'b']) == []

    assert rg1.weakly_connected_components() == [set(rg1.variables())]
    # TODO: Re-enable this once restricted weakly connected components work.
    # assert rg1e.weakly_connected_components(subgraph=['d', 'e']) == [{VariableId(3)},{VariableId(4)}]

    assert rg1.shortest_cycle('a') == [VariableId(0), VariableId(2)]
    assert rg1.shortest_cycle('b') == [VariableId(1), VariableId(2), VariableId(0)]
    assert rg1.shortest_cycle('a', length=1) is None
    assert rg1.shortest_cycle('a', subgraph=['a', 'b']) is None
    assert rg1.shortest_cycle('a', parity='+') == rg1.shortest_cycle('a', parity='-')


def test_boolean_network_inheritence():
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
    })
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
        {'source': VariableId(0), 'target': VariableId(1), 'sign': '+'},
        {'source': VariableId(1), 'target': VariableId(2), 'sign': '-', 'essential': False},
        {'source': VariableId(2), 'target': VariableId(0)},
    ]
    assert bn1.regulation_strings() == ["a -> b", "b -|? c", "c -? a"]
    assert bn1.find_regulation('a', 'c') is None
    assert bn1.find_regulation('c', 'a') == {'source': VariableId(2), 'target': VariableId(0)}
    bn1.add_regulation('a -?? c')
    assert bn1.find_regulation('a', 'c') == {'source': VariableId(0), 'target': VariableId(2), 'essential': False}
    bn1.remove_regulation('a', 'c')
    assert bn1.find_regulation('a', 'c') is None
    assert bn1.ensure_regulation('a -?? c ') is None
    assert bn1.ensure_regulation('a -| c') == {'source': VariableId(0), 'target': VariableId(2), 'essential': False}
    assert bn1.find_regulation('a', 'c') == {'source': VariableId(0), 'target': VariableId(2), 'sign': '-'}

    bn1e = bn1.extend(['d', 'e'])
    bn1e.add_regulation('c -> d')
    bn1e.add_regulation('e -| b')
    assert bn1e != bn1
    assert bn1e.find_variable('e') == VariableId(4)

    assert bn1e.drop(['d', 'e']) == bn1

    # For inlining, the results are actually different compared to normal regulatory graph, because inlining
    # can introduce inlined variables as parameters and give names to anonymous functions. Hence we only compare
    # the RG here.
    assert bn1e.inline_variable('d').inline_variable('e').as_graph() == bn1.as_graph()
    # Keep in mind that rg1 is now extended with 'a -| c'
    assert bn1.inline_variable('c').as_graph() == BooleanNetwork(None, ["a -> b", "b -?? a", "a -? a"]).as_graph()

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
    assert bn1.feedback_vertex_set() == bn1e.feedback_vertex_set()
    assert bn1.feedback_vertex_set(parity='+') == bn1.feedback_vertex_set()
    assert bn1.feedback_vertex_set(parity='-') == bn1.feedback_vertex_set()
    assert bn1.feedback_vertex_set(subgraph=['a', 'b']) == set()

    # TODO: Re-enable this once independent cycles are properly deterministic.
    # assert rg1.independent_cycles() == [[VariableId(0), VariableId(2)]]
    # assert rg1.independent_cycles() == rg1e.independent_cycles()
    assert bn1.independent_cycles(parity='+') == bn1.independent_cycles(parity='-')
    assert bn1.independent_cycles(subgraph=['a', 'b']) == []

    assert bn1.strongly_connected_components() == bn1e.strongly_connected_components()
    assert bn1.strongly_connected_components(subgraph=['a', 'b']) == []

    assert bn1.weakly_connected_components() == [set(bn1.variables())]
    # TODO: Re-enable this once restricted weakly connected components work.
    # assert rg1e.weakly_connected_components(subgraph=['d', 'e']) == [{VariableId(3)},{VariableId(4)}]

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

    assert bn1 == bn2

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
        [("f", 1), ("fn_c", 1)],
        [None, "fn_c(a | f(c))"]
    )

    assert bn1.as_graph() == RegulatoryGraph(regulations=["a -> b", "b -?? c", "c -|? b"])

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
    }

    bn1x = bn1.inline_inputs()
    assert bn1x.explicit_parameter_names() == ["f", "a"]
    # TODO: Enable this once the name resolution bugfix is released.
    # assert bn1x.find_explicit_parameter("a") == ParameterId(1)

    bn1.set_update_function("a", "false")
    bn1x = bn1.inline_constants()
    assert bn1x.variable_names() == ["b", "c"]
    assert str(bn1x.get_update_function("b")) == "f(c)"


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
    # TODO: Uncomment once lib-param-bn is up to date.
    # assert UpdateFunction(bn1, "a & b & c") == UpdateFunction.mk_conjunction(bn1, ["a", "b", "c"])
    # assert UpdateFunction(bn1, "a | b | c") == UpdateFunction.mk_disjunction(bn1, ["a", "b", "c"])

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

    assert expr.substitute_variable("b", "f(b & a)") == UpdateFunction(bn1, "(a & f(b & a)) | g(f(b & a), !c)")
    assert expr.rename_all(bn2, {'a': 'd', 'b': 'c', 'c': 'b'}, {'g': 'h'}) == UpdateFunction(bn2, "(d & c) | h(c, !b)")
    assert expr.substitute_variable("b", "true").simplify_constants() == UpdateFunction(bn1, "a | g(true, !c)")
    assert UpdateFunction(bn1, "!(a & b)").distribute_negation() == UpdateFunction(bn1, "!a | !b")
    # TODO: Uncomment once lib-param-bn is up to date.
    # assert UpdateFunction(bn1, "a ^ b").to_and_or_normal_form() == UpdateFunction(bn1, "(a | b) & !(a & b)")
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
