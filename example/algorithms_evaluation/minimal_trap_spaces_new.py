import sys

from biodivine_aeon import *

bn = BooleanNetwork.from_file(sys.argv[1])
bn = bn.infer_valid_graph()

traps = TrapSpacesComp.create_from(bn).minimal_symbolic()

print(
    f"{traps.cardinality()} ({traps.spaces().cardinality()} | {traps.colors().cardinality()})"
)
