from biodivine_aeon import *
import sys

import biodivine_aeon

biodivine_aeon.LOG_LEVEL = biodivine_aeon.LOG_NOTHING

# This script computes the non-trivial SCCs of a single,
# fully specified Boolean network. This *does not* include fixed points,
# but it includes complex attractors. Note that it is typically faster
# to compute fixed points using the dedicated method and then check
# the remaining state space for complex SCCs.
#
# The script either prints the number of solutions, or the
# first N solutions, assuming N is given as a second argument.
# In such a case, it prints the smallest enclosing subspace of the attractor.
#
# Note that if the network has constant nodes, we can automatically
# percolate them without changing the outcome. However, this is not
# enabled by default to ensure all nodes are present in the result.
# You can uncomment this modification below.
#
# Also note that computing only the first `X` SCCs is not always significantly
# faster than computing the total result. I.e., the "time to first" and
# "time to all" can often be similar (especially if the count is small).
#
# You can use `.aeon`, `.bnet`, or `.sbml` as input model formats.
#
# Print the attractor count:
# ```
# python3 scc.py ./path/to/network.aeon
# ```
#
# Print first 1000 attractors:
# ```
# python3 scc.py ./path/to/network.aeon 1000
# ```

bn = BooleanNetwork.from_file(sys.argv[1])
bn = bn.infer_valid_graph()

# If you want to inline constant input nodes, uncomment this line:
# bn = bn.inline_constants(infer_constants=True, repair_graph=True)

limit = None
if len(sys.argv) == 3:
    limit = int(sys.argv[2])

stg = AsynchronousGraph(bn)

# Assert that the network is fully specified.
assert stg.mk_unit_colors().cardinality() == 1

config: SccConfig = {
    'graph': stg,
}

if limit is not None:
    config['solution_count'] = limit

scc_list = Scc.chain(config)

if limit is None:
    print(f"{len(scc_list)}")
else:
    count = 0
    for scc in scc_list:
        print(scc.vertices().enclosing_named_subspace())
        count += 1
        if count >= limit:
            break
