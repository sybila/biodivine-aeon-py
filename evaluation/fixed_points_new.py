import sys

from biodivine_aeon import *

bn = BooleanNetwork.from_file(sys.argv[1])
bn = bn.infer_valid_graph()

fixed_points = FixedPointsComp.create_from(bn).symbolic()

print(
    f"{fixed_points.cardinality()} ({fixed_points.vertices().cardinality()} | {fixed_points.colors().cardinality()})"
)
