import sys

from biodivine_aeon import AsynchronousGraph, BooleanNetwork, Reachability

bn = BooleanNetwork.from_file(sys.argv[1])
bn = bn.infer_valid_graph()

stg = AsynchronousGraph(bn)

singleton = stg.mk_unit_colored_vertices().pick_singleton()

backward_closed_superset = Reachability.reach_bwd(stg, singleton)

print(
    f"{backward_closed_superset.cardinality()} ({backward_closed_superset.vertices().cardinality()} | {backward_closed_superset.colors().cardinality()})"
)
