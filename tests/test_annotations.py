from biodivine_aeon import *

def test_regulatory_graph_annotations():
    bn = BooleanNetwork.from_file("./tests/model-with-annotations.aeon")
    ann = bn.annotation()

    assert ann.name == "My Fancy Model"
    assert ann.description == "Description of my fancy model."
    assert ann.taxon == "mus musculus"

    assert ann.variable("a").gene_names == ["CtrA"]
    assert ann.variable("a").layout.position == (1.1, 21.3)
    assert ann.variable("b").gene_names == ["StlB"]
    assert ann.variable("b").layout.position == (3.14, 25.1234)

    assert ann.variable("a").references == ["https://some-doi.org", "https://some-dataset.com"]
    assert ann.regulation("a", "b").references == ["https://some-doi.org"]

    assert ann.variable("a").ids.uniprot == ["ID1-uni", "ID2-uni"]
    assert ann.variable("a").ids.geo_cc == ["ID1-cc", "ID2-cc"]
    assert ann.variable("a").ids.geo_mf == ["ID1-mf", "ID2-mf"]
    assert ann.variable("a").ids.geo_bp == ["ID1-bp", "ID2-bp"]
    assert ann.variable("a").ids.ncbi == ["ID1-ncbi", "ID2-ncbi"]

    ann.name = "Changed name"
    ann.description = "Changed description"
    ann.taxon = "Changed taxon"

    ann.variable("a").gene_names = ["Foo1", "Foo2"]
    ann.variable("a").layout.position = (0.0, 0.0)
    ann.variable("b").gene_names = ["Foo3"]
    ann.variable("b").layout.position = (1.0, 2.0)

    ann.variable("b").ids.uniprot = ["foo1", "foo2", "foo3"]
    ann.variable("b").ids.geo_cc = ["foo1"]
    ann.variable("b").ids.geo_mf = ["foo2"]
    ann.variable("b").ids.geo_bp = ["foo3"]
    ann.variable("b").ids.ncbi = ["foo4"]

    ann.regulation("b", "c").references = ["https://some-dataset.com"]

    bn2 = BooleanNetwork.from_aeon(bn.to_aeon())
    ann2 = bn2.annotation()

    assert str(ann) == str(ann2)

def test_drop_variable():
    bn = BooleanNetwork.from_aeon("""
        #! variable: a: gene_name: name_1
        #! variable: b: gene_name: name_2
        #! regulation: a: b: reference: ref_1
        #! regulation: b: a: reference: ref_2
        a -> b
        b -> a
    """)

    bn2 = bn.drop("a")

    expected = "#!variable:b:gene_name:name_2"
    assert expected.strip() == str(bn2.raw_annotation()).strip()

def test_remove_regulation():
    bn = BooleanNetwork.from_aeon("""
        #! variable: a: gene_name: name_1
        #! variable: b: gene_name: name_2
        #! regulation: a: b: reference: ref_1
        #! regulation: b: a: reference: ref_2
        a -> b
        b -> a
    """)

    bn.remove_regulation("a", "b")



    expected = "#!variable:a:gene_name:name_1\n#!variable:b:gene_name:name_2\n#!regulation:b:a:reference:ref_2"

    assert set([l.strip() for l in expected.splitlines()]) == set([l.strip() for l in str(bn.raw_annotation()).splitlines()])

def test_rename_variable():
    bn = BooleanNetwork.from_aeon("""
        #! variable: a: gene_name: name_1
        #! variable: b: gene_name: name_2
        #! regulation: a: b: reference: ref_1
        #! regulation: b: a: reference: ref_2
        a -> b
        b -> a
    """)

    bn.set_variable_name("a", "c")
    bn.set_variable_name("b", "d")

    bn2 = BooleanNetwork.from_aeon("""
        #! variable: c: gene_name: name_1
        #! variable: d: gene_name: name_2
        #! regulation: c: d: reference: ref_1
        #! regulation: d: c: reference: ref_2
        c -> d
        d -> c
    """)

    print(bn.raw_annotation())

    assert set([l.strip() for l in str(bn2.raw_annotation()).splitlines()]) == set([l.strip() for l in str(bn.raw_annotation()).splitlines()])

def test_variable_inline():
    bn = BooleanNetwork.from_aeon("""
        #! variable: a: gene_name: name_1
        #! variable: b: gene_name: name_2
        #! variable: c: gene_name: name_3
        #! variable: d: gene_name: name_4
        #! variable: e: gene_name: name_5
        #! regulation: a: c: reference: ref_1
        #! regulation: b: c: reference: ref_2
        #! regulation: c: d: reference: ref_3
        #! regulation: c: e: reference: ref_4
        #! regulation: d: d: reference: ref_5
        a -> c
        b -> c
        c -> d
        c -> e
        d -> d
    """)

    bn = bn.inline_variable("c")

    bn2 = BooleanNetwork.from_aeon("""
        #! variable: a: gene_name: name_1
        #! variable: b: gene_name: name_2        
        #! variable: d: gene_name: name_4
        #! variable: d: gene_name: name_3
        #! variable: e: gene_name: name_5
        #! variable: e: gene_name: name_3                
        #! regulation: b: d: reference: ref_3
        #! regulation: b: d: reference: ref_2        
        #! regulation: b: e: reference: ref_4
        #! regulation: b: e: reference: ref_2        
        #! regulation: a: d: reference: ref_3
        #! regulation: a: d: reference: ref_1        
        #! regulation: a: e: reference: ref_4
        #! regulation: a: e: reference: ref_1        
        #! regulation: d: d: reference: ref_5        
        a -> e
        a -> d
        b -> e
        b -> d
    """)

    assert set([l.strip() for l in str(bn2.raw_annotation()).splitlines()]) == set([l.strip() for l in str(bn.raw_annotation()).splitlines()])
