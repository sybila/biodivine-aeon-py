from biodivine_aeon import *
import pickle
import os


def test_class():
    c = Class(["c", "a"])
    assert str(c) == '["a", "c"]'
    assert eval(repr(c)) == c
    assert len(c) == 2
    assert {c: 5}[c] == 5
    assert pickle.loads(pickle.dumps(c)) == c

    assert c.ensure(["a", "b"]) == Class(["a", "b", "c"])
    assert c.append(["a", "b"]) == Class(["a", "a", "b", "c"])

    assert c.append("c").erase("c") == Class(["a"])
    assert c.append("c").minus("c") == c

    assert c == Class({"a", "a", "c"})
    assert c == Class("a").ensure(Class("c"))


def test_annotation_manipulation():
    ann = ModelAnnotation()

    Classification.write_dynamic_assertions(ann, ["assertion_1", "assertion_2"])
    Classification.write_dynamic_properties(ann, [("p_1", "property_1")])
    Classification.write_dynamic_properties(ann, [("p_2", "property_2")])

    print(ann)

    assert Classification.read_dynamic_assertions(ann) == ["assertion_1", "assertion_2"]
    assert Classification.read_dynamic_properties(ann) == [
        ("p_1", "property_1"),
        ("p_2", "property_2"),
    ]


def test_archive_input_output():
    bn = BooleanNetwork.from_aeon("""
        a -> b
        b -|? c
        c -?? b
        c -| a
        $b: a & f(c)    
    """)

    graph = AsynchronousGraph(bn)

    all_colors = graph.mk_unit_colors()
    c1 = graph.mk_function_colors("f", "!x_0").intersect(all_colors)
    c2 = graph.mk_function_colors("f", "true").intersect(all_colors)

    assert not c1.is_empty() and not c2.is_empty()
    cls = {Class(["not", "a"]): c1, Class(["tt", "b"]): c2}

    # Technically, using "custom" classes with property annotations is not supported by the BN Classifier.
    # Furthermore, the data we are writing isn't even a valid HCTL.
    # However, we can still write this as an output.
    ann = ModelAnnotation()
    Classification.write_dynamic_assertions(ann, ["assertion_1"])
    Classification.write_dynamic_properties(ann, [("p_1", "property_1")])
    ann["description"].value = "My model description."

    Classification.save_classification("classification.test.zip", bn, cls, ann)
    (l_bn, l_cls, l_ann) = Classification.load_classification("classification.test.zip")
    assert l_bn == bn
    assert l_cls == cls
    assert str(l_ann) == str(ann)

    os.remove("classification.test.zip")


def test_attractor_classification():
    bn = BooleanNetwork.from_file("./tests/model-2.aeon")
    stg = AsynchronousGraph(bn)

    attractors = Attractors.attractors(stg)

    # This test works because the model does not have colors.
    for a in attractors:
        mapping = Classification.classify_long_term_behavior(stg, a)
        if a.is_singleton():
            assert len(mapping) == 1 and mapping[Class("stability")] == a.colors()
        else:
            assert len(mapping) == 1 and mapping[Class("disorder")] == a.colors()

    mapping = Classification.classify_attractor_bifurcation(stg)
    assert len(mapping) == 1 and mapping[Class(["disorder", "stability", "stability"])] == stg.mk_unit_colors()


def test_property_classification():
    path = "./tests/model-with-properties.aeon"

    annotations = ModelAnnotation.from_file(path)
    assertions_str = Classification.read_dynamic_assertions(annotations)
    assert len(assertions_str) == 1
    properties_str = Classification.read_dynamic_properties(annotations)
    assert len(properties_str) == 4

    assertions = [HctlFormula(x) for x in assertions_str]
    properties = {k: HctlFormula(v) for k, v in properties_str}

    bn = BooleanNetwork.from_file(path)
    # The original network has some ambiguity in operator precedence and the "canonical" output will print the
    # functions slightly differently compared to the original representation, but they are semantically equal.
    # We do this to ensure that all ambiguity is structured in a way that is repeatable in the aeon parser.
    bn = BooleanNetwork.from_aeon(bn.to_aeon())

    graph = AsynchronousGraph(bn)
    mapping = Classification.classify_dynamic_properties(graph, properties, assertions)

    assert len(mapping) == 3
    assert mapping[Class("p2")].cardinality() == 3
    assert mapping[Class("p4")].cardinality() == 3
    assert mapping[Class(["p1", "p2"])].cardinality() == 9
    # Test that the result is compatible with the original STG, not the one that is extended for model checking.
    assert mapping[Class("p2")].intersect(graph.mk_unit_colors()) == mapping[Class("p2")]

    Classification.save_classification("classification.test.2.zip", bn, mapping, annotations)
    (l_bn, l_cls, l_ann) = Classification.load_classification("classification.test.2.zip")
    assert l_bn == bn
    assert l_cls == mapping
    assert str(l_ann) == str(annotations)

    os.remove("classification.test.2.zip")
