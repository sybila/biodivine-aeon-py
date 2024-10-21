from biodivine_aeon import *
import sys

# This script computes the fixed points of a single, 
# fully specified Boolean network.
# 
# It either prints the number of solutions, or the
# fist N solutions, assuming N is given as a second argument.
#
# Note that if the network has constant nodes, we can automatically
# percolate them without changing the outcome. However, this is not
# enabled by default to ensure all nodes are present in the result.
# You can uncomment this modification below. 
# 
# Also note that computing only first X fixed points is not faster
# than computing the total cardinality of the set. I.e. the 
# "time to first" and "time to all" is the same for this implementation.
#
# You can use `.aeon`, `.bnet`, or `.sbml` as input model formats.
#
# Print the fixed-point count:
# ```
# python3 fixed_points.py ./path/to/network.aeon 
# ```
#
# Print first 1000 fixed-points:
# ```
# python3 fixed_points.py ./path/to/network.aeon 1000
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

if limit is None:
	print(f"{fixed_points.cardinality()}")
else:
	count = 0
	for vertex in fixed_points.vertices():
		print(vertex.to_named_dict())
		count += 1
		if count >= limit:
			break



