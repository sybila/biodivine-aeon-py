from biodivine_aeon import *
import sys

# This file provides functionality that is equivalent to the `stability_analysis` feature in the AEON
# online interface

def checkStability(graph: AsynchronousGraph, attractor: ColoredVertexSet, variable: VariableIdType):

    # Colors where variable oscillates
    oscillation = graph.var_can_post_within(variable, attractor).colors()
    attractor = attractor.minus_colors(oscillation)

    # In the remaining colors, the variable is always stable; we just need
    # to check which stable value it takes...
    var_true = graph.mk_subspace({ variable: True })
    var_false = graph.mk_subspace({ variable: False })
    stable_true = attractor.intersect(var_true).colors()
    stable_false = attractor.intersect(var_false).colors()

    return {
        Class('oscillation'): oscillation,
        Class('true'): stable_true,
        Class('false'): stable_false
    }

bn = BooleanNetwork.from_file(sys.argv[1])
bn = bn.inline_inputs(infer_inputs=True, repair_graph=True)

graph = AsynchronousGraph(bn)
print(f"Model color count: {graph.mk_unit_colors().cardinality()}")
attractors = Attractors.attractors(graph)
print(f"Found {len(attractors)} attractor set(s).")

for var in bn.variables():
    # Initially, all colors belong to an empty class, but as we go through the attractors,
    # they should be re-assigned to the correct class.
    classification = { Class([]) : graph.mk_unit_colors() }
    for attractor in attractors:
        stability = checkStability(graph, attractor, var)
        for cls, colors in stability.items():
            classification = Classification.ensure(classification, cls, colors)

    print(f"\tVariable: {bn.get_variable_name(var)}")
    for cls, colors in classification.items():
        print(f"\t\t{cls}: {colors.cardinality()}")