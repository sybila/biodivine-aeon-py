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
    pass