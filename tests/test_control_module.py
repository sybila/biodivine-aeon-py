from biodivine_aeon import *
from typing import Literal
from functools import reduce


def test_witness():
    # This test is based on the case study notebook.

    bn = BooleanNetwork.from_file("./tests/model-myeloid-witness.aeon")
    pstg = AsynchronousPerturbationGraph(bn)

    attractors = Attractors.attractors(pstg)
    attractor_states = [a.vertices() for a in attractors]

    erythrocyte = pstg.mk_subspace_vertices({"EKLF": True})
    erythrocyte_att = [a for a in attractor_states if not a.intersect(erythrocyte).is_empty()][0]

    megakaryocyte = pstg.mk_subspace_vertices({"Fli1": True})
    megakaryocyte_att = [a for a in attractor_states if not a.intersect(megakaryocyte).is_empty()][0]

    # monocyte = pstg.mk_subspace_vertices({"cJun": True})
    # monocyte_att = [a for a in attractor_states if not a.intersect(monocyte).is_empty()][0]

    # granulocyte = pstg.mk_subspace_vertices({"Gfi1": True})
    # granulocyte_att = [a for a in attractor_states if not a.intersect(granulocyte).is_empty()][0]

    sym_results = Control.attractor_one_step(pstg, erythrocyte_att, megakaryocyte_att)

    assert sym_results.select_perturbation({"EKLF": True}).is_empty()
    assert sym_results.select_perturbation({"EKLF": None}).is_empty()
    assert sym_results.select_perturbation({"EKLF": False, "Fli1": True}).is_singleton()

    # There should be exactly one perturbation of size 2 that works.
    good = sym_results.select_by_size(size=2, up_to=True).select_by_robustness(0.99, result_limit=10)
    assert len(good) == 1
    assert good[0][0]["EKLF"] is False
    assert good[0][0]["Fli1"] is True

    phen_sym_results = Control.phenotype_permanent(pstg,
                                                   megakaryocyte_att,
                                                   oscillation_type="forbidden",
                                                   stop_when_found=False,
                                                   size_limit=10)

    # Similarly, only one perturbation of size 2 actually works for phenotype control
    good = phen_sym_results.select_by_size(size=2, up_to=True).select_by_robustness(threshold=0.99, result_limit=10)
    assert len(good) == 1
    assert good[0][0].perturbation_size() == 2
    assert good[0][0]["Fli1"] is True
    assert good[0][0]["PU1"] is False


def test_asynchronous_graph_inheritance():
    # This test roughly follows what is used in the base AsynchronousGraph test, but some aspects had to be
    # updated because the perturbed graph does not support custom context, unit BDD set, and it automatically
    # transforms implicit parameters into explicit ones.

    bn = BooleanNetwork.from_aeon("""
    a -> b
    b -|? c
    c -?? b
    c -| a
    $b: a & f(c)    
    """)

    graph = AsynchronousPerturbationGraph(bn, perturb=["a", "c"])
    ctx = graph.symbolic_context()

    assert str(graph) == f"AsynchronousPerturbationGraph({ctx})"

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

    assert unit_vertices.cardinality() == 8
    assert unit_colors.cardinality() == 9
    assert unit_set.cardinality() == 72

    # Compared to the original, we have to use an explicit parameter name, because implicit parameters are erased
    # by the graph constructor.

    assert graph.mk_function_colors("f", "true").is_subspace()
    assert graph.mk_function_colors("f_a", "x_0").intersect(graph.mk_function_colors("f_a", "!x_0")).is_empty()

    assert graph.mk_function_colors("f", "true").is_subset(graph.mk_function_row_colors("f", [0], True))
    assert graph.mk_function_colors("f_a", "x_0").is_subset(graph.mk_function_row_colors("f_a", [1], True))

    assert graph.transfer_from(unit_set, graph) == unit_set
    assert graph.transfer_from(unit_colors, graph) == unit_colors
    assert graph.transfer_from(unit_vertices, graph) == unit_vertices

    assert graph.mk_update_function("a") == ctx.mk_function("f_a", ["c"])

    space_arg: dict[str, Literal[0, 1]] = {"a": 0, "b": 1, "c": 1}
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
    stg = AsynchronousPerturbationGraph(bn)
    assert stg.reconstruct_network() == bn

    assert stg.mk_unit_vertices().extend_with_colors(stg.mk_unit_colors()) == stg.mk_unit_colored_vertices()
    assert stg.mk_unit_colors().extend_with_vertices(stg.mk_unit_vertices()) == stg.mk_unit_colored_vertices()


def test_symbolic_representation():
    # This test should cover other basic features of AsynchronousPerturbationGraph that are not present in the
    # AsynchronousGraph.

    bn = BooleanNetwork.from_aeon("""
    a -> b
    b -|? c
    c -?? b
    c -| a
    $b: a & f(c)    
    """)

    graph = AsynchronousPerturbationGraph(bn, perturb=["a", "c"])

    normal_colors = graph.mk_unit_colors()
    perturbable_colors = graph.mk_perturbable_unit_colors()

    normal_vertices = graph.mk_unit_colored_vertices()
    perturbable_vertices = graph.mk_perturbable_unit_colored_vertices()

    assert normal_colors.is_subset(perturbable_colors)
    assert normal_vertices.is_subset(perturbable_vertices)

    assert normal_colors.cardinality() == 9
    assert perturbable_colors.cardinality() == 9 * 4

    assert normal_vertices.cardinality() == 9 * 8
    assert perturbable_vertices.cardinality() == 9 * 8 * 4

    empty_set = graph.mk_empty_colored_perturbations()
    empty_pert = graph.mk_empty_perturbations()

    unit_set = graph.mk_unit_colored_perturbations()
    unit_pert = graph.mk_unit_perturbations()

    # There are 9 perturbations and 9 "proper" colors.
    assert unit_pert.cardinality() == 9
    assert sum(1 for _ in unit_pert) == 9
    assert unit_set.cardinality() == 9 * 9
    assert sum(1 for _ in unit_set) == 9 * 9

    assert empty_set.union(unit_set) == unit_set
    assert empty_pert.union(unit_pert) == unit_pert
    assert empty_set.intersect(unit_set) == empty_set
    assert empty_pert.intersect(unit_pert) == empty_pert
    assert unit_set.minus(empty_set) == unit_set
    assert unit_pert.minus(empty_pert) == unit_pert

    assert empty_set.is_empty() and not unit_set.is_empty()
    assert empty_pert.is_empty() and not unit_pert.is_empty()

    assert empty_set.is_subset(unit_set)
    assert empty_pert.is_subset(unit_pert)

    single = graph.mk_perturbations({"a": True, "c": None})
    assert graph.mk_perturbation({"a": True}) == single
    assert not unit_pert.is_singleton()
    assert single.is_singleton()
    assert single.cardinality() == 1
    assert unit_pert.pick_singleton().is_singleton()
    assert unit_pert.pick_singleton().cardinality() == 1

    it = unit_set.__iter__()
    (m1, m2) = it.__next__()
    assert m1.to_symbolic().is_singleton()
    assert m2.to_symbolic().is_singleton()
    c1 = m1.to_symbolic().extend_with_perturbations(m2.to_symbolic())
    c2 = m2.to_symbolic().extend_with_colors(m1.to_symbolic())
    assert c1 == c2
    assert c1.is_singleton()
    assert c2.is_singleton()

    c_single = single.extend_with_colors(normal_colors.pick_singleton())

    assert not unit_set.is_singleton()
    assert c_single.is_singleton()
    assert c_single.cardinality() == 1
    assert sum(1 for _ in c_single) == 1
    assert unit_set.pick_singleton().is_singleton()
    assert unit_set.pick_singleton().cardinality() == 1
    assert sum(1 for _ in unit_set.pick_singleton()) == 1

    assert unit_set.perturbations() == unit_pert
    assert unit_set.colors() == normal_colors

    assert unit_set.intersect_colors(normal_colors.pick_singleton()).cardinality() == 9
    assert unit_set.intersect_perturbations(unit_pert.pick_singleton()).cardinality() == 9
    assert unit_set.minus_colors(normal_colors.pick_singleton()).cardinality() == 9 * 8
    assert unit_set.minus_perturbations(unit_pert.pick_singleton()).cardinality() == 9 * 8

    p_singleton = unit_pert.pick_singleton()
    assert graph.mk_perturbation({}) == p_singleton
    assert unit_set.select_perturbation({"a": None, "c": None}).cardinality() == 9
    assert unit_set.minus_perturbations(p_singleton).select_perturbation({"a": None, "c": None}).is_empty()

    # Make a set that contains all pairs except for one.
    some_colors = graph.mk_function_colors("f", "true")
    some_set = unit_set.minus(p_singleton.extend_with_colors(some_colors))

    assert some_set.select_perturbations({}) == some_set
    assert some_set.select_perturbations({"a": True}).cardinality() == 3 * 9
    assert some_set.select_perturbations({"a": None}).cardinality() == 2 * 9 + 6

    # Test that robustness is computed correctly for this set.
    assert some_set.select_perturbation({"a": True, "c": False}).cardinality() == 9
    assert some_set.select_perturbation({}).cardinality() == 6

    assert some_set.perturbation_robustness({"a": True, "c": False}) == 1.0
    assert some_set.perturbation_robustness({"a": None, "c": None}) == 0.666666  # 6 / 9 rounded to 6 places.

    some_set_zero = some_set.select_by_size(0, up_to=True)
    some_set_one = some_set.select_by_size(1, up_to=True)
    assert len(some_set.select_by_robustness(0.99)) == unit_pert.cardinality() - 1
    assert len(some_set_zero.select_by_robustness(0.99)) == 0
    assert len(some_set_zero.select_by_robustness(0.50)) == 1
    assert len(some_set_one.select_by_robustness(0.99)) == 4
    assert len(some_set_one.select_by_robustness(0.50)) == 5
    assert some_set_one.select_by_robustness(0.99, result_limit=1)[0][0].perturbation_size() == 1
    assert some_set_one.select_by_robustness(0.50, result_limit=1)[0][0].perturbation_size() == 0

# def test_base_network_compatibility():
#     bn = BooleanNetwork.from_aeon("""
#     a -> b
#     b -|? c
#     c -?? b
#     c -| a
#     $b: a & f(c)
#     """)
#
#     graph = AsynchronousPerturbationGraph(bn)
#
#     for p in graph.mk_unit_colors():
#         p.instantiate(bn)