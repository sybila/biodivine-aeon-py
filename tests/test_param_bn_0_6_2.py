"""
Tests for new features added in lib-param-bn 0.6.2 update.

This module tests the new functionality that was added to the Python bindings
to reflect the changes in lib-param-bn from version 0.5.13 to 0.6.2.
"""

import pytest
import biodivine_aeon as aeon


def test_symbolic_space_context_new_methods():
    """Test the new methods added to SymbolicSpaceContext."""
    # Create a simple Boolean network
    network = aeon.BooleanNetwork.from_aeon("""
    a -> a
    b -> b
    """)
    
    # Create a symbolic space context
    ctx = aeon.SymbolicSpaceContext(network)
    
    # Test get_dual_variable_pair
    var_a = network.find_variable("a")
    pos_var, neg_var = ctx.get_dual_variable_pair(var_a)
    assert isinstance(pos_var, aeon.BddVariable)
    assert isinstance(neg_var, aeon.BddVariable)
    assert pos_var != neg_var
    
    # Test get_dual_variables
    dual_vars = ctx.get_dual_variables()
    assert len(dual_vars) == 2  # Two variables in the network
    for pos_var, neg_var in dual_vars:
        assert isinstance(pos_var, aeon.BddVariable)
        assert isinstance(neg_var, aeon.BddVariable)
    
    # Test mk_exactly_k_free_spaces
    spaces_0 = ctx.mk_exactly_k_free_spaces(0)
    spaces_1 = ctx.mk_exactly_k_free_spaces(1)
    spaces_2 = ctx.mk_exactly_k_free_spaces(2)
    
    assert isinstance(spaces_0, aeon.SpaceSet)
    assert isinstance(spaces_1, aeon.SpaceSet)
    assert isinstance(spaces_2, aeon.SpaceSet)
    
    # Test mk_can_go_to_false
    # Create a simple BDD function
    graph = aeon.AsynchronousGraph(network, ctx)
    function = graph.mk_update_function("a")
    can_go_false = ctx.mk_can_go_to_false(function)
    assert isinstance(can_go_false, aeon.Bdd)
    
    # Test mk_has_down_transition and mk_has_up_transition
    var_bdd = ctx.find_network_bdd_variable("a")
    has_down = ctx.mk_has_down_transition(var_bdd, function)
    has_up = ctx.mk_has_up_transition(var_bdd, function)
    
    assert isinstance(has_down, aeon.Bdd)
    assert has_down.as_bool() is None
    assert isinstance(has_up, aeon.Bdd)
    assert has_up.as_bool() is None


def test_asynchronous_graph_logically_unique_colors():
    """Test the new logically_unique_subset method on AsynchronousGraph."""
    # Create a simple Boolean network
    network = aeon.BooleanNetwork.from_aeon("""
    a -> a
    b -> a
    c -> a
    $a: c | f(a, b, c)
    """)
    
    graph = aeon.AsynchronousGraph(network)
    
    # Get some colors
    colors = graph.mk_unit_colors()
    
    # Test logically_unique_subset
    unique_colors = graph.logically_unique_colors(colors)
    assert unique_colors.cardinality() < colors.cardinality()

    # The result should be a subset of the original colors
    assert unique_colors.is_subset(colors)


def test_trap_spaces_new_methods():
    """Test the new methods and updated signatures in TrapSpaces."""
    # Create a simple Boolean network
    network = aeon.BooleanNetwork.from_aeon("""
    a -> a
    b -> b
    """)
    
    # Create context and graph
    ctx = aeon.SymbolicSpaceContext(network)
    graph = aeon.AsynchronousGraph(network, ctx)
    
    # Test minimal_symbolic with new exclude_fixed_points parameter
    minimal_spaces = aeon.TrapSpaces.minimal_symbolic(ctx, graph)
    assert isinstance(minimal_spaces, aeon.ColoredSpaceSet)
    
    # Test with exclude_fixed_points parameter
    fixed_points = graph.mk_unit_colored_vertices()
    minimal_spaces_excluded = aeon.TrapSpaces.minimal_symbolic(
        ctx, graph, exclude_fixed_points=fixed_points
    )
    assert minimal_spaces_excluded.is_empty()
    
    # Test long_lived_symbolic
    long_lived_spaces = aeon.TrapSpaces.long_lived_symbolic(ctx, graph)
    assert not long_lived_spaces.is_empty()
    
    # Test with restriction
    restriction = ctx.mk_empty_colored_spaces()
    long_lived_restricted = aeon.TrapSpaces.long_lived_symbolic(
        ctx, graph, restriction=restriction
    )
    assert long_lived_restricted.is_empty()


def test_comprehensive_workflow():
    """Test a comprehensive workflow using the new features."""
    # Create a more complex Boolean network
    network = aeon.BooleanNetwork.from_aeon("""
    a -> a
    b -> b
    c -> c
    """)
    
    # Create context and graph
    ctx = aeon.SymbolicSpaceContext(network)
    graph = aeon.AsynchronousGraph(network, ctx)
    
    # Test dual variables
    dual_vars = ctx.get_dual_variables()
    assert len(dual_vars) == 3  # Three variables
    
    # Test space operations
    spaces_1 = ctx.mk_exactly_k_free_spaces(1)
    spaces_2 = ctx.mk_exactly_k_free_spaces(2)
    
    # Test transition analysis
    var_a = network.find_variable("a")
    bdd_a = ctx.find_network_bdd_variable(var_a)
    
    function_a = graph.mk_update_function("a")
    has_down = ctx.mk_has_down_transition(bdd_a, function_a)
    has_up = ctx.mk_has_up_transition(bdd_a, function_a)
    
    # Test trap space analysis with new features
    minimal_spaces = aeon.TrapSpaces.minimal_symbolic(ctx, graph)
    long_lived_spaces = aeon.TrapSpaces.long_lived_symbolic(ctx, graph)
    
    # Test logical uniqueness
    colors = graph.mk_unit_colors()
    unique_colors = graph.logically_unique_colors(colors)
    
    # All operations should complete without errors
    assert isinstance(minimal_spaces, aeon.ColoredSpaceSet)
    assert isinstance(long_lived_spaces, aeon.ColoredSpaceSet)
    assert isinstance(unique_colors, aeon.ColorSet)
    assert isinstance(spaces_1, aeon.SpaceSet)
    assert isinstance(spaces_2, aeon.SpaceSet)
    assert isinstance(has_down, aeon.Bdd)
    assert isinstance(has_up, aeon.Bdd)


if __name__ == "__main__":
    pytest.main([__file__])
