from biodivine_aeon import *

def test_bma_model_loading():
    model = BooleanNetwork.from_file("./tests/small-bma-model.json", repair_graph=True, binarize=True)

    assert model.variable_count() == 3 + 4 + 1
    assert model.explicit_parameter_count() == 0
    assert model.implicit_parameter_count() == 0
    for var in model.variables():
        assert model.get_update_function(var) is not None

def test_aeon_to_bma_round_trip():
    model = BooleanNetwork.from_file("./tests/model-myeloid-witness.aeon")

    print(model.to_bma_xml())

    model_json = BooleanNetwork.from_bma_json(model.to_bma_json())
    model_xml = BooleanNetwork.from_bma_xml(model.to_bma_xml())

    assert model_json.variable_count() == model.variable_count()
    assert model_json.regulation_count() == model.regulation_count()

    assert model_xml.variable_count() == model.variable_count()
    assert model_xml.regulation_count() == model.regulation_count()

    assert model_xml == model_json
