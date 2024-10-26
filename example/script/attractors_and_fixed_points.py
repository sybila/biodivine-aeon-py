from biodivine_aeon import *
import sys

# This script computes the attractors of a single,
# fully specified Boolean network. This includes both fixed points
# and complex attractors. In this script, we first compute all fixed
# points and then search the remaining state space for complex
# attractors.
#
# The script either prints the number of solutions (first fixed points,
# then complex attractors), or the fist N solutions, assuming N is given
# as a second argument.
#
# In such case, it prints whether the attractor is fixed point
# or complex, and its enclosing subspace (for fixed points, this
# is exactly the single state).
#
# Note that if the network has constant nodes, we can automatically
# percolate them without changing the outcome. However, this is not
# enabled by default to ensure all nodes are present in the result.
# You can uncomment this modification below.
#
# Also note that computing only first X attractors is not faster
# than computing the total result. I.e. the "time to first" and
# "time to all" is the same for this implementation.
#
# You can use `.aeon`, `.bnet`, or `.sbml` as input model formats.
#
# Print the attractor:
# ```
# python3 attractors_and_fixed_points.py ./path/to/network.aeon
# ```
#
# Print first 1000 attractors:
# ```
# python3 attractors_and_fixed_points.py ./path/to/network.aeon 1000
# ```

bn = BooleanNetwork.from_file(sys.argv[1])
bn = bn.infer_valid_graph()

# If you want to inline constant input nodes, uncomment this line:
#bn = bn.inline_constants(infer_constants=True, repair_graph=True)

limit = None
if len(sys.argv) == 3:
    limit = int(sys.argv[2])

stg = AsynchronousGraph(bn)

# Assert that the network is fully specified.
assert stg.mk_unit_colors().cardinality() == 1

fixed_points = FixedPoints.symbolic(stg)

count = 0

if limit is not None:
    for vertex in fixed_points.vertices():
        if count >= limit:
            break
        print(f"fixed-point\t{vertex.to_named_dict()}")
        count += 1
else:
    print(fixed_points.cardinality())


fixed_point_basin = Reachability.reach_bwd(stg, fixed_points)
attractor_candidates = stg.mk_unit_colored_vertices().minus(fixed_point_basin)

attractors = Attractors.attractors(stg, attractor_candidates)

if limit is not None:
    for attractor in attractors:
        if count >= limit:
            break
        print(f"complex\t{attractor.vertices().enclosing_named_subspace()}")
        count += 1
else:
    print(len(attractors))

