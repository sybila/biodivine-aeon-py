import sys

from biodivine_aeon import *

bn = BooleanNetwork.from_file(sys.argv[1])
bn = bn.infer_valid_graph()
print("Boolean network loaded.\n")

stg = AsynchronousGraph(bn)
config = ReachabilityConfig.with_graph(stg).cancel_after(10)
print("Reachability config created.\n")

singleton = stg.mk_unit_colored_vertices().pick_singleton()
print("Initial state:")
print(singleton)
print()

reach = Reachability.with_config(config)
print("Reachability running forward_closed_superset.")
result = reach.forward_closed_superset(singleton)

print("Result state:")
print(result)
print()

print("Reachability running backward_closed_superset.")
result = reach.backward_closed_superset(singleton)

print("Result state:")
print(result)
print()

reach_with_graph = Reachability.with_graph(stg)
print("Reachability running backward_closed_superset.")
result = reach_with_graph.backward_closed_superset(singleton)

print("Result state:")
print(result)
