from biodivine_aeon import *
import sys

# A simple demo script showcasing the percolation functionality.
# Right now, this isn't really doing anything useful, but it is
#  a reasonable demonstration of the percolation feature.

bn = BooleanNetwork.from_file(sys.argv[1])
bn = bn.infer_valid_graph()

ctx = SymbolicSpaceContext(bn)
graph = AsynchronousGraph(bn, ctx)

limit = 1000
for space in ctx.mk_unit_spaces():
    space_dict = { k: v for k, v in space.items() if v is not None }
    p_space = Percolation.percolate_subspace(graph, space_dict)
    print(f"{len(space_dict)} -> {len(p_space)}")
    limit -= 1
    if limit == 0:
        break
