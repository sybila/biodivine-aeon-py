"""
Tests for new features added in lib-param-bn 0.6.2 update.

This module tests the new functionality that was added to the Python bindings
to reflect the changes in lib-param-bn from version 0.5.13 to 0.6.2.
"""

import pytest
import biodivine_aeon as aeon



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

if __name__ == "__main__":
    pytest.main([__file__])
