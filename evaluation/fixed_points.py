import sys

from biodivine_aeon import *

bn = BooleanNetwork.from_file(sys.argv[1])
bn = bn.infer_valid_graph()

stg = AsynchronousGraph(bn)

fixed_points = FixedPoints.symbolic(stg)

print(
    f"{fixed_points.cardinality()} ({fixed_points.vertices().cardinality()} | {fixed_points.colors().cardinality()})"
)
