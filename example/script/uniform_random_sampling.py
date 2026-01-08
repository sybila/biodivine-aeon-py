from biodivine_aeon import *

# This script showcases the ability to randomly uniformly sample
# from symbolic sets. This property then allows building more complex
# statistics workflows on top of it.

bn = BooleanNetwork.from_file('../case-study/butanol-production/butanol-pathway-f1-f2-f3.aeon')
graph = AsynchronousGraph(bn)

print("Total number of interpretations:", graph.mk_unit_colors().cardinality())

# The network has three unknown functions: f_1(2), f_2(2) and f_3(3).
# First, let us find all colors that exhibit the `butanol` phenotype:

butanol = graph.mk_subspace({ 'butanol': 1 })
print("Colors where butanol=1: ", butanol.colors())
butanol_phenotype = Reachability.forward_subset(graph, butanol)
print("Colors where exists trap set with butanol=1: ", butanol_phenotype.colors())

# Now, we want to test a hypothesis that f_1 and f_2 are independent of each other.
# In particular, both f_1 and f_2 can be either conjunction or disjunction. We thus
# want to test whether the choice in f_1 influences the choice in f_2.

# (Note that we could also test this particular hypothesis exactly through
# careful manipulation of the symbolic set, but it is a useful motivating example)

color_sampler = butanol_phenotype.colors().sample_items(retained=['f_1', 'f_2'], seed=1)
samples = 0
same = 0
different = 0
while samples < 10000:
    samples += 1
    sample = next(color_sampler)
    f_1_expr = sample['f_1']
    f_2_expr = sample['f_2']
    if f_1_expr.is_and() == f_2_expr.is_and():
        same += 1
    else:
        different += 1
    if samples % 1000 == 0:
        print(f"{same/samples:.2f} / {different/samples:.2f}")

# If all went according to plan, we should get a roughly 50/50 distribution in the end.

# Note that similar random sampling functions are available on all symbolic sets.