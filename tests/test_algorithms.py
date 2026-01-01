from biodivine_aeon import *
import pytest

def union_all(graph: AsynchronousGraph, items: list[ColoredVertexSet]) -> ColoredVertexSet:
    result = graph.mk_empty_colored_vertices()
    for item in items:
        result = result.union(item)
    return result

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

    candidates = unit.minus(minimal_trap_states)
    reduced = Attractors.transition_guided_reduction(graph, candidates)
    assert reduced.is_empty()

    tgr = Attractors.transition_guided_reduction(graph, unit)
    attractors = Attractors.attractors(graph, unit)
    attractors2 = Attractors.xie_beerel(graph, unit)

    assert len(attractors) > 0
    assert len(attractors2) > 0

    attractor_states = union_all(graph, attractors)
    attractor_states2 = union_all(graph, attractors2)

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

    for a in attractors:
        assert Reachability.forward_superset(graph, a) == a
        assert Reachability.forward_subset(graph, a) == a

        # Attractor is either fully bwd-closed or not (and is empty).
        # However, we have a colored model, so we need to account for that.
        bwd_closed = Reachability.backward_subset(graph, a)
        closed_colors = bwd_closed.colors()
        assert bwd_closed.intersect_colors(closed_colors) == a.intersect_colors(closed_colors)

        pivot = a.pick_vertex()
        fwd = Reachability.forward_superset(graph, pivot)
        bwd = Reachability.backward_superset(graph, pivot)
        assert fwd.intersect(bwd) == a

        assert Reachability.backward_subset(graph, bwd) == bwd
        assert Reachability.forward_subset(graph, fwd) == fwd

    # Test that symbolic size limits work:

    with pytest.raises(InterruptedError):
        config_reach: ReachabilityConfig = {
            'graph': graph,
            'max_symbolic_size': 10
        }
        Reachability.backward_superset(config_reach, unit.pick_vertex())

    config_reach = {
        'graph': graph,
        'max_symbolic_size': 10_000
    }

    assert not Reachability.backward_superset(config_reach, unit.pick_vertex()).is_empty()

    with pytest.raises(InterruptedError):
        config_attr: AttractorConfig = {
            'graph': graph,
            'max_symbolic_size': 10
        }
        Attractors.attractors(config_attr, unit)

    config_attr = {
        'graph': graph,
        'max_symbolic_size': 10_000
    }

    assert len(Attractors.attractors(config_attr, unit)) > 0

    scc_list = Scc.fwd_bwd(graph, unit)
    scc_states = union_all(graph, scc_list)

    # Every attractor is an SCC (or fixed-point), and we should have at least
    # one SCC in this model that isn't an attractor.
    assert attractor_states.is_subset(scc_states.union(fixed_points))
    assert not scc_states.minus(attractor_states).is_empty()

    # Chain should give us the same SCCs as fwd-bwd:
    scc_list_2 = Scc.chain(graph, unit)
    scc_states_2 = union_all(graph, scc_list_2)

    assert scc_states == scc_states_2

    # Try SCC decomposition with custom config
    scc_config: SccConfig = {
        'graph': graph,
        'should_trim': 'sources',
        'filter_long_lived': True
    }

    # Every long-lived SCC is an SCC, but the model should also have short-lived SCCs
    scc_list_3 = Scc.chain(scc_config, unit)
    scc_states_3 = union_all(graph, scc_list_3)
    assert scc_states_3.is_subset(scc_states)
    assert not scc_states.minus(scc_states_3).is_empty()

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