# frozen_string_literal: true

require 'test_helper'

# triangle
class TestTriangle < Minitest::Test
  def setup
    @triangle = Triangle.new(3, 4, 5)
  end

  # Polytest Suite: triangle

  # Polytest Group: polygon

  def test_edge_count
    assert_equal 3, @triangle.edge_count
  end

  def test_vertex_count
    assert_equal 3, @triangle.vertex_count
  end

  # Polytest Group: shape

  def test_perimeter
    assert_equal @triangle.side_a + @triangle.side_b + @triangle.side_c, @triangle.perimeter
  end

  def test_area
    s = (@triangle.side_a + @triangle.side_b + @triangle.side_c) / 2.0
    expected_area = Math.sqrt(
      s * (s - @triangle.side_a) * (s - @triangle.side_b) * (s - @triangle.side_c)
    )
    assert_equal expected_area, @triangle.area
  end
end
