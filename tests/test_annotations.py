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
