import pytest
from .. import Triangle

# Polytest Suite: triangle

@pytest.fixture
def triangle():
    return Triangle(3, 4, 5)

# Polytest Group: polygon

@pytest.mark.group_polygon
def test_edge_count(triangle):
    """A polygon should accurately count the number of edges it contains"""
    assert triangle.edge_count() == 3

@pytest.mark.group_polygon
def test_vertex_count(triangle):
    """A polygon should accurately count the number of verticies it contains"""
    assert triangle.vertex_count() == 3

# Polytest Group: shape

@pytest.mark.group_shape
def test_perimeter(triangle):
    """A shape should be able to accurately calculate its perimeter (or circumference)"""
    assert triangle.perimeter() == triangle.side_a + triangle.side_b + triangle.side_c

@pytest.mark.group_shape
def test_area(triangle):
    """A shape should be able to accurately calculate its area"""
    s = (triangle.side_a + triangle.side_b + triangle.side_c) / 2
    expected_area = (s * (s - triangle.side_a) * 
                    (s - triangle.side_b) * 
                    (s - triangle.side_c)) ** 0.5
    assert triangle.area() == expected_area