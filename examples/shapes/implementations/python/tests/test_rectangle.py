import pytest
from .. import Rectangle

# Polytest Suite: rectangle

@pytest.fixture
def rectangle():
    return Rectangle(10, 4)

# Polytest Group: rectangle

@pytest.mark.group_rectangle
def test_is_square():
    """A rectangle should be able to accurately determine if it is a square"""
    square = Rectangle(10, 10)
    rect = Rectangle(10, 4)
    
    assert square.is_square() is True
    assert rect.is_square() is False

# Polytest Group: polygon

@pytest.mark.group_polygon
def test_edge_count(rectangle):
    """A polygon should accurately count the number of edges it contains"""
    assert rectangle.edge_count() == 4

@pytest.mark.group_polygon
def test_vertex_count(rectangle):
    """A polygon should accurately count the number of verticies it contains"""
    assert rectangle.vertex_count() == 4

# Polytest Group: shape

@pytest.mark.group_shape
def test_perimeter(rectangle):
    """A shape should be able to accurately calculate its perimeter (or circumference)"""
    expected = 2 * (rectangle.width + rectangle.height)
    assert rectangle.perimeter() == expected

@pytest.mark.group_shape
def test_area(rectangle):
    """A shape should be able to accurately calculate its area"""
    assert rectangle.area() == rectangle.width * rectangle.height