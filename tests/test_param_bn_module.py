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
