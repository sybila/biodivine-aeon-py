import sys

from biodivine_aeon import (
    AsynchronousGraph,
    BooleanNetwork,
    SymbolicSpaceContext,
    TrapSpaces,
)

bn = BooleanNetwork.from_file(sys.argv[1])
bn = bn.infer_valid_graph()

ctx = SymbolicSpaceContext(bn)
stg = AsynchronousGraph(bn, ctx)

traps = TrapSpaces.minimal_symbolic(ctx, stg)

print(
    f"{traps.cardinality()} ({traps.spaces().cardinality()} | {traps.colors().cardinality()})"
)
