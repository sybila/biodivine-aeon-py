import sys

from biodivine_aeon import *

bn = BooleanNetwork.from_file(sys.argv[1])
bn = bn.infer_valid_graph()

stg = AsynchronousGraph(bn)

singleton = stg.mk_unit_colored_vertices().pick_singleton()

forward_closed_superset = ReachabilityComp.create_from(stg).forward_closed_superset(
    singleton
)

print(
    f"{forward_closed_superset.cardinality()} ({forward_closed_superset.vertices().cardinality()} | {forward_closed_superset.colors().cardinality()})"
)
