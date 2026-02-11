import pytest

from biodivine_aeon import BiodivineBooleanModels, BbmModel


def test_fetch_model_by_numeric_id():
    """Test fetching models using numeric IDs (as strings)."""
    # Test with various numeric IDs
    model_1 = BiodivineBooleanModels.fetch_model("1")
    model_5 = BiodivineBooleanModels.fetch_model("5")
    model_50 = BiodivineBooleanModels.fetch_model("50")
    model_100 = BiodivineBooleanModels.fetch_model("100")
    
    # Verify they are BbmModel instances
    assert isinstance(model_1, BbmModel)
    assert isinstance(model_5, BbmModel)
    assert isinstance(model_50, BbmModel)
    assert isinstance(model_100, BbmModel)
    
    # Verify they have required attributes
    assert hasattr(model_1, 'id')
    assert hasattr(model_1, 'name')
    assert hasattr(model_1, 'variables')
    assert hasattr(model_1, 'inputs')
    assert hasattr(model_1, 'regulations')
    assert hasattr(model_1, 'keywords')


def test_fetch_model_properties():
    """Test that fetched models have valid properties."""
    model = BiodivineBooleanModels.fetch_model("10")
    
    # Verify basic properties are present and valid
    assert isinstance(model.id, str)
    assert len(model.id) > 0
    assert isinstance(model.name, str)
    assert len(model.name) > 0
    assert isinstance(model.variables, int)
    assert model.variables >= 0
    assert isinstance(model.inputs, int)
    assert model.inputs >= 0
    assert isinstance(model.regulations, int)
    assert model.regulations >= 0
    assert isinstance(model.keywords, list)


def test_fetch_model_consistency():
    """Test that fetching the same model ID multiple times returns consistent results."""
    model1 = BiodivineBooleanModels.fetch_model("25")
    model2 = BiodivineBooleanModels.fetch_model("25")
    
    # The same ID should return models with the same ID
    assert model1.id == model2.id
    assert model1.name == model2.name
    assert model1.variables == model2.variables
    assert model1.inputs == model2.inputs
    assert model1.regulations == model2.regulations


def test_fetch_different_models():
    """Test that different numeric IDs return different models."""
    model_1 = BiodivineBooleanModels.fetch_model("1")
    model_2 = BiodivineBooleanModels.fetch_model("2")
    
    # Different IDs should generally return different models
    # (They might have different names, variables, etc.)
    # At minimum, they should have different IDs or be distinct objects
    assert model_1.id != model_2.id or model_1.name != model_2.name


def test_fetch_model_high_id():
    """Test fetching a model with a higher numeric ID (close to 200)."""
    # Test with IDs closer to 200
    model_150 = BiodivineBooleanModels.fetch_model("150")
    model_200 = BiodivineBooleanModels.fetch_model("200")
    
    assert isinstance(model_150, BbmModel)
    assert isinstance(model_200, BbmModel)


def test_fetch_model_invalid_id():
    """Test that fetching with an invalid ID raises an error."""
    # Test with a very high ID that likely doesn't exist
    with pytest.raises(Exception):  # Should raise an error for a non-existent model
        BiodivineBooleanModels.fetch_model("99999")
    
    # Test with an empty string
    with pytest.raises(Exception):
        BiodivineBooleanModels.fetch_model("")


def test_fetch_model_string_representation():
    """Test that BbmModel has proper string representation."""
    model = BiodivineBooleanModels.fetch_model("15")
    
    # Test __str__ method
    str_repr = str(model)
    assert isinstance(str_repr, str)
    assert len(str_repr) > 0
    assert "BbmModel" in str_repr or model.id in str_repr
    
    # Test __repr__ method
    repr_str = repr(model)
    assert isinstance(repr_str, str)
    assert len(repr_str) > 0

