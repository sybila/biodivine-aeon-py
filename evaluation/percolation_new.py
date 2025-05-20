import sys

from biodivine_aeon import *

# A simple demo script showcasing the percolation functionality.
# Right now, this isn't really doing anything useful, but it is
#  a reasonable demonstration of the percolation feature.
# Right now, it is theoretically possible to miss some errors
#  because of the summing of the lengths of the dictionaries.
#  If the dictionaries differ between implementations but their
#  lengths add up to the same number, we will not notice.
#  However, this is unlikely to happen in practice.

bn = BooleanNetwork.from_file(sys.argv[1])
bn = bn.infer_valid_graph()

ctx = SymbolicSpaceContext(bn)
graph = AsynchronousGraph(bn, ctx)

space_dict_len_sum, p_space_len_sum = 0, 0
limit = 1000
percolation = PercolationComp.create_from(graph)
for space in ctx.mk_unit_spaces():
    space_dict = {k: v for k, v in space.items() if v is not None}
    p_space = percolation.percolate_subspace(space_dict)

    space_dict_len_sum += len(space_dict)
    p_space_len_sum += len(p_space)

    limit -= 1
    if limit == 0:
        break

print(f"{space_dict_len_sum} -> {p_space_len_sum}")
