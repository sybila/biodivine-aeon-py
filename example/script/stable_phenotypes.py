from biodivine_aeon import *
import sys

from typing import Optional, Sequence

# Compute the stable phenotypes of a Boolean network, and list the number of interpretations (colors) that
# allow said phenotype.
#
# A stable phenotype is represented by a collection of variables that are invariant in at least one attractor
# of the network. If a variable is not listed in a phenotype, it is oscillating.
#
# Since the number of phenotypes is often large, you can specify relevant variables and the results are then
# aggregated to unique combinations of these variables, not all variables. For example, if there are two phenotypes,
# X=1,Y=0 with 5 colors and X=1,Y=1 with 10 colors, selecting only the X variable will give you a X=1 phenotype with
# 15 colors.
#
# Example:
# ```
# python3 ./stable_phenotypes.py ../case-study/butanol-production/butanol-pathway-relaxed.aeon "acetone" "butanol" "lactate" "pyruvate" "ethanol" "sporulation"
# ```
#
# This should identify 19 phenotypes of various sizes. Note that the order in which they are printed is not
# deterministic because the result is a dictionary.
#

bn = BooleanNetwork.from_file(sys.argv[1])
bn = bn.infer_valid_graph()

variables: Optional[Sequence[str]]
if len(sys.argv) > 2:
    variables = sys.argv[2:]
    print("Variable restriction:", variables)
else:
    variables = None

ctx = SymbolicSpaceContext(bn)
stg = AsynchronousGraph(bn, ctx)

classification = Classification.classify_stable_phenotypes(ctx, stg, variables)

print(f"Found {len(classification)} phenotypes.")
for k, v in classification.items():
    print(f"{k}: {v.cardinality()}")