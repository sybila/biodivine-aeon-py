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

def test_attractor_bifurcation_with_perturbations():
    path = './tests/model-myeloid-witness.aeon'
    model_unknown = BooleanNetwork.from_file(path)
    model_unknown.set_update_function("GATA1", None)
    model_unknown.set_update_function("CEBPa", None)
    model_unknown.set_update_function("PU1", None)
    model_unknown = model_unknown.name_implicit_parameters()

    pstg = AsynchronousPerturbationGraph(model_unknown)
    stg = AsynchronousGraph(model_unknown)
    mapping_pstg = Classification.classify_attractor_bifurcation(pstg)
    mapping_stg = Classification.classify_attractor_bifurcation(stg)

    assert len(mapping_stg) == len(mapping_pstg)

    for (cls, colors) in mapping_stg.items():
        assert pstg.transfer_from(colors, stg) == mapping_pstg[cls]
    for (cls, colors) in mapping_pstg.items():
        assert stg.transfer_from(colors, pstg) == mapping_stg[cls]

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

    Classification.save_classification("classification.test.2.zip", bn, mapping)
    (l_bn, l_cls, l_ann) = Classification.load_classification("classification.test.2.zip")
    assert l_bn == bn
    assert l_cls == mapping
    assert str(l_ann) == str(annotations)

    os.remove("classification.test.2.zip")

def test_phenotype_classification():
    path = "./tests/model-myeloid-3-unknown.aeon"

    bn = BooleanNetwork.from_file(path)

    graph = AsynchronousGraph(bn)
    phenotypes = {
        Class('erythrocyte'): graph.mk_subspace_vertices({"EKLF": True}),
        Class('megakaryocyte'): graph.mk_subspace_vertices({"Fli1": True}),
        Class('monocyte'): graph.mk_subspace_vertices({"cJun": True}),
        Class('granulocyte'): graph.mk_subspace_vertices({"Gfi1": True})
    }
    phenotype_mapping = Classification.classify_phenotypes(graph, phenotypes)
    phenotype_attractor_mapping = Classification.classify_attractor_phenotypes(graph, phenotypes, count_multiplicity=False)

    # An example of how to nicely print the mappings:
    for (cls, colors) in phenotype_mapping.items():
        print(cls, colors)
    # Expected output:
    # ["granulocyte", "megakaryocyte", "monocyte"] ColorSet(cardinality=1423062, symbolic_size=543)
    # ["erythrocyte", "granulocyte", "megakaryocyte"] ColorSet(cardinality=4617, symbolic_size=197)
    # ["granulocyte", "monocyte"] ColorSet(cardinality=58482, symbolic_size=295)
    # ["granulocyte", "megakaryocyte"] ColorSet(cardinality=4617, symbolic_size=197)
    # ["erythrocyte", "megakaryocyte"] ColorSet(cardinality=53865, symbolic_size=279)
    # ["megakaryocyte"] ColorSet(cardinality=53865, symbolic_size=279)
    # ["erythrocyte", "granulocyte", "megakaryocyte", "monocyte"] ColorSet(cardinality=1364580, symbolic_size=561)

    for (cls, colors) in phenotype_attractor_mapping.items():
        print(cls)
        print([eval(x).feature_list() for x in cls.feature_list()], colors)

    # Expected output:
    # [['granulocyte', 'megakaryocyte'], ['megakaryocyte', 'monocyte'], ['megakaryocyte']] ColorSet(cardinality=1364976, symbolic_size=675)
    # [['granulocyte', 'megakaryocyte'], ['megakaryocyte', 'monocyte']] ColorSet(cardinality=58086, symbolic_size=408)
    # [['granulocyte', 'megakaryocyte'], ['megakaryocyte']] ColorSet(cardinality=4617, symbolic_size=197)
    # [['erythrocyte', 'granulocyte'], ['erythrocyte'], ['granulocyte', 'megakaryocyte'], ['megakaryocyte']] ColorSet(cardinality=4221, symbolic_size=213)
    # [['megakaryocyte']] ColorSet(cardinality=53865, symbolic_size=279)
    # [['granulocyte'], ['monocyte']] ColorSet(cardinality=58086, symbolic_size=408)
    # [['granulocyte'], ['monocyte'], []] ColorSet(cardinality=396, symbolic_size=141)
    # [['erythrocyte'], ['granulocyte'], ['megakaryocyte'], ['monocyte'], []] ColorSet(cardinality=4617, symbolic_size=165)
    # [['erythrocyte'], ['granulocyte'], ['megakaryocyte'], ['monocyte']] ColorSet(cardinality=1281078, symbolic_size=858)
    # [['erythrocyte', 'granulocyte'], ['erythrocyte'], ['granulocyte', 'megakaryocyte'], ['granulocyte'], ['megakaryocyte'], ['monocyte']] ColorSet(cardinality=78885, symbolic_size=453)
    # [['erythrocyte'], ['megakaryocyte']] ColorSet(cardinality=53865, symbolic_size=279)
    # [['erythrocyte'], ['granulocyte'], ['megakaryocyte']] ColorSet(cardinality=396, symbolic_size=141)

    # Test that every attractor class is a subset of the corresponding phenotype class:
    for (cls, colors) in phenotype_attractor_mapping.items():
        phenotype_class = Class([])
        for inner in cls.feature_list():
            inner_cls: Class = eval(inner)
            for phenotype in inner_cls.feature_list():
                phenotype_class = phenotype_class.ensure(phenotype)
        assert colors.is_subset(phenotype_mapping[phenotype_class])



test_phenotype_classification()
