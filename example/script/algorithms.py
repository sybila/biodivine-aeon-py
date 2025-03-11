import sys

from biodivine_aeon import *

bn = BooleanNetwork.from_file(sys.argv[1])
bn = bn.infer_valid_graph()
print("Boolean network loaded.\n")

stg = AsynchronousGraph(bn)
config = ReachabilityConfig.with_graph(stg).with_time_limit(10_000)
print("Reachability config created.\n")

test_set = {VariableId(1), VariableId(2)}
test_config = ReachabilityConfig(stg, variables=test_set, time_limit_millis=5_000)
test_config = test_config.with_variables(test_set)

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
