import pytest

# Polytest Suite: car

# Polytest Group: broken vehicle

@pytest.mark.group_broken_vehicle
def test_flat_tire_is_caught():
    """The function for checking tires should return false if the vehicle has a flat tire"""
    raise Exception("TEST NOT IMPLEMENTED")

@pytest.mark.group_broken_vehicle
def test_broken_headlight_is_caught():
    """The function for checking headlights should return false if the vehicle has a broken headlight"""
    raise Exception("TEST NOT IMPLEMENTED")

# Polytest Group: invalid vehicle

@pytest.mark.group_invalid_vehicle
def test_extra_tire_throws_error():
    """Having an extra tire is invalid and should throw an error"""
    raise Exception("TEST NOT IMPLEMENTED")

@pytest.mark.group_invalid_vehicle
def test_extra_headlight_throws_error():
    """Having an extra headlight is invalid and should throw an error"""
    raise Exception("TEST NOT IMPLEMENTED")

# Polytest Group: valid vehicle

@pytest.mark.group_valid_vehicle
def test_check_tires():
    """The function for checking tires should return true if the vehicle has the correct number of inflated tires"""
    raise Exception("TEST NOT IMPLEMENTED")

@pytest.mark.group_valid_vehicle
def test_check_headlights():
    """The function for checking headlights should return true if the vehicle has the correct number of headlights"""
    raise Exception("TEST NOT IMPLEMENTED")