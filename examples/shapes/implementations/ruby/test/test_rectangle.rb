# frozen_string_literal: true

require 'test_helper'

# rectangle
class TestRectangle < Minitest::Test
  def setup
    @rectangle = Rectangle.new(10, 4)
  end

  # Polytest Suite: rectangle

  # Polytest Group: rectangle

  def test_is_square
    square = Rectangle.new(10, 10)
    assert square.square?
    refute @rectangle.square?
  end

  # Polytest Group: polygon

  def test_edge_count
    assert_equal 4, @rectangle.edge_count
  end

  def test_vertex_count
    assert_equal 4, @rectangle.vertex_count
  end

  # Polytest Group: shape

  def test_perimeter
    assert_equal 2 * (@rectangle.width + @rectangle.height), @rectangle.perimeter
  end

  def test_area
    assert_equal @rectangle.width * @rectangle.height, @rectangle.area
  end
end
