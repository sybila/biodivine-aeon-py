from biodivine_aeon import *
import biodivine_aeon
import sys

biodivine_aeon.LOG_LEVEL = biodivine_aeon.LOG_ESSENTIAL

# This script computes the minimal trap spaces of a single, 
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
# Also note that computing only first X trap spaces is not faster
# than computing the total cardinality of the set. I.e. the 
# "time to first" and "time to all" is the same for this implementation.
#
# You can use `.aeon`, `.bnet`, or `.sbml` as input model formats.
#
# Print the fixed-point count:
# ```
# python3 minimal_trap_spaces.py ./path/to/network.aeon 
# ```
#
# Print first 1000 minimal trap spaces:
# ```
# python3 minimal_trap_spaces.py ./path/to/network.aeon 1000
# ```

bn = BooleanNetwork.from_file(sys.argv[1])
bn = bn.infer_valid_graph()

# If you want to inline constant input nodes, uncomment this line:
#bn = bn.inline_constants(infer_constants=True, repair_graph=True)

limit = None
if len(sys.argv) == 3:
	limit = int(sys.argv[2])

ctx = SymbolicSpaceContext(bn)
stg = AsynchronousGraph(bn, ctx)

# Assert that the network is fully specified.
assert stg.mk_unit_colors().cardinality() == 1

essential = TrapSpaces.essential_symbolic(ctx, stg)
print(f"Complete: {essential.cardinality()}")
traps = TrapSpaces.minimize(ctx, essential)

if limit is None:
	print(f"Minimal: {traps.cardinality()}")
else:
	count = 0
	for space in traps.spaces():
		print(space.to_named_dict())
		count += 1
		if count >= limit:
			break



