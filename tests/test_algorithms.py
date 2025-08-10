from biodivine_aeon import *


def test_algorithms():
    bn = BooleanNetwork.from_file("./example/workflow/data/g2a_p1026.aeon")
    ctx = SymbolicSpaceContext(bn)
    graph = AsynchronousGraph(bn, ctx)

    unit = graph.mk_unit_colored_vertices()
    space_unit = ctx.mk_unit_colored_spaces(graph)

    fixed_points = FixedPoints.symbolic(graph, unit)
    fixed_point_vertices = FixedPoints.symbolic_vertices(graph, unit)
    fixed_point_colors = FixedPoints.symbolic_colors(graph, unit)

    assert fixed_point_vertices == fixed_points.vertices()
    assert fixed_point_colors == fixed_points.colors()
    assert fixed_points == FixedPoints.symbolic(graph)
    assert fixed_point_vertices == FixedPoints.symbolic_vertices(graph)
    assert fixed_point_colors == FixedPoints.symbolic_colors(graph)

    minimal_traps = TrapSpaces.minimal_symbolic(ctx, graph, space_unit)
    essential_traps = TrapSpaces.essential_symbolic(ctx, graph, space_unit)
    minimal_trap_states = minimal_traps.to_colored_vertices(ctx)
    maximal_traps = TrapSpaces.maximize(ctx, essential_traps)

    assert minimal_traps.is_subset(essential_traps)
    assert maximal_traps.is_subset(essential_traps)
    assert minimal_traps == TrapSpaces.minimize(ctx, essential_traps)
    assert minimal_traps == TrapSpaces.minimal_symbolic(ctx, graph)
    assert essential_traps == TrapSpaces.essential_symbolic(ctx, graph)

    print(unit)
    assert Attractors.transition_guided_reduction(graph, unit.minus(minimal_trap_states)).is_empty()

    tgr = Attractors.transition_guided_reduction(graph, unit)
    attractors = Attractors.attractors(graph, unit)
    attractors2 = Attractors.xie_beerel(graph, unit)

    assert len(attractors) == 2
    assert len(attractors2) == 5
    attractor_states = attractors[0].union(attractors[1])
    attractor_states2 = graph.mk_empty_colored_vertices()
    for x in attractors2:
        attractor_states2 = attractor_states2.union(x)

    assert attractor_states2 == attractor_states

    assert tgr != unit and tgr.is_subset(unit)
    assert attractor_states.is_subset(tgr)
    assert fixed_points.is_subset(attractor_states)
    assert attractor_states.is_subset(minimal_trap_states)

    assert Reachability.reach_bwd(graph, attractor_states) == unit

    for a in attractors:
        assert Reachability.reach_fwd(graph, a) == a

        pivot = a.pick_vertex()
        fwd = Reachability.reach_fwd(graph, pivot)
        bwd = Reachability.reach_bwd(graph, pivot)

        assert fwd.intersect(bwd) == a

def test_percolation_case_1():
    bn = BooleanNetwork.from_file("./tests/model-3.aeon")
    stg = AsynchronousGraph(bn)

    subspace = { 'APC': 1 }
    percolated = Percolation.percolate_subspace(stg, subspace)
    print(percolated)

    # Percolating one input should not affect other outputs!
    inputs = bn.inputs(infer=True)
    for v in inputs:
        if bn.get_variable_name(v) == 'APC':
            continue
        assert v not in percolated