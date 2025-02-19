import sys

from biodivine_aeon import *

print("Hello from algorithms.py")

bn = BooleanNetwork.from_file(sys.argv[1])
bn = bn.infer_valid_graph()

stg = AsynchronousGraph(bn)

config = ReachabilityConfig(stg)

# print(config.get_graph())
# print(config.sorted_variables())

reach = Reachability(config)
singleton = stg.mk_unit_colored_vertices().pick_singleton()
print(singleton)
print("running forward_closed_superset")
result = reach.forward_closed_superset(singleton)
print(result)
