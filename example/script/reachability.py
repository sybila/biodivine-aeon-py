import logging
import sys

from biodivine_aeon import *

logging.basicConfig(level=logging.DEBUG)

bn = BooleanNetwork.from_file(sys.argv[1])
bn = bn.infer_valid_graph()
print("Boolean network loaded.\n")

stg = AsynchronousGraph(bn)
config: ReachabilityConfig = {
    'graph': stg,
    'max_symbolic_size': 10_000
}

print("Reachability config created.\n")

singleton = stg.mk_unit_colored_vertices().pick_singleton()
print("Initial state:")
print(singleton)
print()

print("Reachability running forward_closed_superset.")
result = Reachability.forward_superset(config, singleton)

print("Result state:")
print(result)
print()

print("Reachability running backward_closed_superset.")
result = Reachability.backward_superset(config, singleton)

print("Result state:")
print(result)
print()

print("Reachability running backward_closed_superset.")
result = Reachability.backward_superset(stg, singleton)

print("Result state:")
print(result)
