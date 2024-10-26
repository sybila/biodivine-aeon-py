from biodivine_aeon import *
import pickle
import pytest


def test_hctl_formula():
    f = HctlFormula("V{a}: (EF {a}) EU (AF var_1)")

    assert f.children() == [HctlFormula("(EF {a}) EU (AF var_1)")]
    assert f.operator() == "forall"
    assert f.used_extended_properties() == set()
    assert f.used_state_variables() == { "a" }

    with pytest.raises(RuntimeError):
        # Garbage syntax
        HctlFormula("V{x}. foo")

    assert pickle.loads(pickle.dumps(f)) == f

    # Test builders.

    assert HctlFormula.mk_hybrid("exists", "v", f) == HctlFormula.mk_exists("v", f)
    assert HctlFormula.mk_hybrid("forall", "v", f) == HctlFormula.mk_forall("v", f)
    assert HctlFormula.mk_hybrid("bind", "v", f) == HctlFormula.mk_bind("v", f)
    assert HctlFormula.mk_hybrid("jump", "v", f) == HctlFormula.mk_jump("v", f)
    assert HctlFormula.mk_hybrid("exists", "v", f, "dom") == HctlFormula.mk_exists("v", f, "dom")
    assert HctlFormula.mk_hybrid("forall", "v", f, "dom") == HctlFormula.mk_forall("v", f, "dom")
    assert HctlFormula.mk_hybrid("bind", "v", f, "dom") == HctlFormula.mk_bind("v", f, "dom")

    assert HctlFormula.mk_temporal("exist_next", f) == HctlFormula.mk_exist_next(f)
    assert HctlFormula.mk_temporal("all_next", f) == HctlFormula.mk_all_next(f)
    assert HctlFormula.mk_temporal("exist_future", f) == HctlFormula.mk_exist_future(f)
    assert HctlFormula.mk_temporal("all_future", f) == HctlFormula.mk_all_future(f)
    assert HctlFormula.mk_temporal("exist_global", f) == HctlFormula.mk_exist_global(f)
    assert HctlFormula.mk_temporal("all_global", f) == HctlFormula.mk_all_global(f)

    assert HctlFormula.mk_temporal("exist_until", f, f) == HctlFormula.mk_exist_until(f, f)
    assert HctlFormula.mk_temporal("all_until", f, f) == HctlFormula.mk_all_until(f, f)
    assert HctlFormula.mk_temporal("exist_weak_until", f, f) == HctlFormula.mk_exist_weak_until(f, f)
    assert HctlFormula.mk_temporal("all_weak_until", f, f) == HctlFormula.mk_all_weak_until(f, f)

    assert HctlFormula.mk_boolean("and", f, f) == HctlFormula.mk_and(f, f)
    assert HctlFormula.mk_boolean("or", f, f) == HctlFormula.mk_or(f, f)
    assert HctlFormula.mk_boolean("imp", f, f) == HctlFormula.mk_imp(f, f)
    assert HctlFormula.mk_boolean("iff", f, f) == HctlFormula.mk_iff(f, f)
    assert HctlFormula.mk_boolean("xor", f, f) == HctlFormula.mk_xor(f, f)

    assert HctlFormula.mk_not(HctlFormula("a")) == HctlFormula("~a")
    assert HctlFormula.mk_network_var("var_1") == HctlFormula("var_1")
    assert HctlFormula.mk_extended_prop("prop") == HctlFormula("%prop%")
    assert HctlFormula.mk_const(True) == HctlFormula("true")
    assert HctlFormula.mk_state_var("x_1") == HctlFormula("{x_1}")

    # Test checks and destructors.

    a = HctlFormula("a")
    b = HctlFormula("b")

    ff = HctlFormula("3{x}: a")
    assert ff.is_exists() and ff.is_hybrid()
    assert not (ff.is_exists_in() or ff.is_hybrid_in())
    assert ff.as_exists() == ("x", a) and ff.as_hybrid() == ("exists", "x", a)

    ff = HctlFormula("3{x} in %dom%: a")
    assert ff.is_exists_in() and ff.is_hybrid_in()
    assert not (ff.is_exists() or ff.is_hybrid())
    assert ff.as_exists_in() == ("x", "dom", a) and ff.as_hybrid_in() == ("exists", "x", "dom", a)

    ff = HctlFormula("V{x}: a")
    assert ff.is_forall() and ff.is_hybrid()
    assert not (ff.is_forall_in() or ff.is_hybrid_in())
    assert ff.as_forall() == ("x", a) and ff.as_hybrid() == ("forall", "x", a)

    ff = HctlFormula("V{x} in %dom%: a")
    assert ff.is_forall_in() and ff.is_hybrid_in()
    assert not (ff.is_forall() or ff.is_hybrid())
    assert ff.as_forall_in() == ("x", "dom", a) and ff.as_hybrid_in() == ("forall", "x", "dom", a)

    ff = HctlFormula("!{x}: a")
    assert ff.is_bind() and ff.is_hybrid()
    assert not (ff.is_bind_in() or ff.is_hybrid_in())
    assert ff.as_bind() == ("x", a) and ff.as_hybrid() == ("bind", "x", a)

    ff = HctlFormula("!{x} in %dom%: a")
    assert ff.is_bind_in() and ff.is_hybrid_in()
    assert not (ff.is_bind() or ff.is_hybrid())
    assert ff.as_bind_in() == ("x", "dom", a) and ff.as_hybrid_in() == ("bind", "x", "dom", a)

    ff = HctlFormula("@{x}: a")
    assert ff.is_jump() and ff.is_hybrid()
    assert ff.as_jump() == ("x", a) and ff.as_hybrid() == ("jump", "x", a)

    ff = HctlFormula("EX a")
    assert ff.is_exist_next() and ff.is_temporal_unary() and ff.is_temporal()
    assert ff.as_exist_next() == a and ff.as_temporal_unary() == ("exist_next", a)

    ff = HctlFormula("AX a")
    assert ff.is_all_next() and ff.is_temporal_unary() and ff.is_temporal()
    assert ff.as_all_next() == a and ff.as_temporal_unary() == ("all_next", a)

    ff = HctlFormula("EF a")
    assert ff.is_exist_future() and ff.is_temporal_unary() and ff.is_temporal()
    assert ff.as_exist_future() == a and ff.as_temporal_unary() == ("exist_future", a)

    ff = HctlFormula("AF a")
    assert ff.is_all_future() and ff.is_temporal_unary() and ff.is_temporal()
    assert ff.as_all_future() == a and ff.as_temporal_unary() == ("all_future", a)

    ff = HctlFormula("EG a")
    assert ff.is_exist_global() and ff.is_temporal_unary() and ff.is_temporal()
    assert ff.as_exist_global() == a and ff.as_temporal_unary() == ("exist_global", a)

    ff = HctlFormula("AG a")
    assert ff.is_all_global() and ff.is_temporal_unary() and ff.is_temporal()
    assert ff.as_all_global() == a and ff.as_temporal_unary() == ("all_global", a)

    ff = HctlFormula("a EU b")
    assert ff.is_exist_until() and ff.is_temporal_binary() and ff.is_temporal()
    assert ff.as_exist_until() == (a, b) and ff.as_temporal_binary() == ("exist_until", a, b)

    ff = HctlFormula("a AU b")
    assert ff.is_all_until() and ff.is_temporal_binary() and ff.is_temporal()
    assert ff.as_all_until() == (a, b) and ff.as_temporal_binary() == ("all_until", a, b)

    ff = HctlFormula("a EW b")
    assert ff.is_exist_weak_until() and ff.is_temporal_binary() and ff.is_temporal()
    assert ff.as_exist_weak_until() == (a, b) and ff.as_temporal_binary() == ("exist_weak_until", a, b)

    ff = HctlFormula("a AW b")
    assert ff.is_all_weak_until() and ff.is_temporal_binary() and ff.is_temporal()
    assert ff.as_all_weak_until() == (a, b) and ff.as_temporal_binary() == ("all_weak_until", a, b)

    ff = HctlFormula("a & b")
    assert ff.is_and() and ff.is_boolean()
    assert ff.as_and() == (a, b) and ff.as_boolean() == ("and", a, b)

    ff = HctlFormula("a | b")
    assert ff.is_or() and ff.is_boolean()
    assert ff.as_or() == (a, b) and ff.as_boolean() == ("or", a, b)

    ff = HctlFormula("a => b")
    assert ff.is_imp() and ff.is_boolean()
    assert ff.as_imp() == (a, b) and ff.as_boolean() == ("imp", a, b)

    ff = HctlFormula("a <=> b")
    assert ff.is_iff() and ff.is_boolean()
    assert ff.as_iff() == (a, b) and ff.as_boolean() == ("iff", a, b)

    ff = HctlFormula("a ^ b")
    assert ff.is_xor() and ff.is_boolean()
    assert ff.as_xor() == (a, b) and ff.as_boolean() == ("xor", a, b)

    ff = HctlFormula("var_1")
    assert ff.is_network_var() and ff.as_network_var() == "var_1"

    ff = HctlFormula("{x_1}")
    assert ff.is_state_var() and ff.as_state_var() == "x_1"

    ff = HctlFormula("%p_1%")
    assert ff.is_extended_prop() and ff.as_extended_prop() == "p_1"

    ff = HctlFormula("true")
    assert ff.is_const() and ff.as_const()


def test_model_checker():
    network = BooleanNetwork.from_file("./tests/model-2.aeon")

    # There should be two fixed-points.
    fixed_points = HctlFormula("!{x}: AX {x}")
    # There should be a complex attractor with these variables set.
    phenotype = HctlFormula("AG ~n1 & ~n2 & ~n3")

    # We'll use this to test the extended propositions.
    # In theory, everything should either reach a fixed-point, or the phenotype attractor.
    basin = HctlFormula("EF (%fix% | %phenotype%)")

    stg = AsynchronousGraph.mk_for_model_checking(network, 1)

    assert fixed_points.is_compatible_with(stg)
    assert phenotype.is_compatible_with(stg)
    assert basin.is_compatible_with(stg)

    r = ModelChecking.verify(stg, [fixed_points, phenotype])
    f = r[0]
    p = r[1]

    assert f.cardinality() == 2
    assert p.cardinality() >= 16

    b = ModelChecking.verify(stg, basin, {"fix": f, "phenotype": p})

    assert b == stg.mk_unit_colored_vertices()
