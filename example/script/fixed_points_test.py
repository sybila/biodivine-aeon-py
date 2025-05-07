import logging
import sys

from biodivine_aeon import *

logging.basicConfig(level=logging.DEBUG)

bn = BooleanNetwork.from_file(sys.argv[1])
bn = bn.infer_valid_graph()

stg = AsynchronousGraph(bn)

# Assert that the network is fully specified.
# assert stg.mk_unit_colors().cardinality() == 1

fp_new = FixedPointsComp.with_config(
    FixedPointsConfig.create_from(stg).with_time_limit(5_000)
)
fp_new_result = fp_new.symbolic()

print(
    f"{fp_new_result.cardinality()} ({fp_new_result.vertices().cardinality()} | {fp_new_result.colors().cardinality()})"
)

fp_old_result = FixedPoints.symbolic(stg)

print(
    f"{fp_old_result.cardinality()} ({fp_old_result.vertices().cardinality()} | {fp_old_result.colors().cardinality()})"
)
