import pytest
import math
from .. import Circle

# Polytest Suite: circle

RADIUS = 7
circle = Circle(RADIUS)

# Polytest Group: circle

@pytest.mark.group_circle
def test_diameter():
    """A circle should be able to accurately calculate its diameter"""
    assert circle.diameter() == 14

@pytest.mark.group_circle
def test_radius():
    """A circle should be able to accurately calculate its radius"""
    assert circle.radius == 7

# Polytest Group: shape

@pytest.mark.group_shape
def test_perimeter():
    """A shape should be able to accurately calculate its perimeter (or circumference)"""
    assert circle.perimeter() == math.pi * RADIUS * 2

@pytest.mark.group_shape
def test_area():
    """A shape should be able to accurately calculate its area"""
    assert circle.area() == math.pi * RADIUS ** 2