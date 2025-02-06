from biodivine_aeon import *
import sys

print("Hello from algorithms.py")

bn = BooleanNetwork.from_file(sys.argv[1])
bn = bn.infer_valid_graph()

stg = AsynchronousGraph(bn)

config = ReachabilityConfig(stg)

print(config.get_graph())
print(config.sorted_variables())
